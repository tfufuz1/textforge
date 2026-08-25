import { writable, derived, get } from 'svelte/store';
import { 
    listSnippets, 
    getSnippet, 
    createSnippet, 
    updateSnippet, 
    trashSnippet,
    duplicateSnippet,
    duplicateSnippetsBulk,
    restoreSnippet,
    deleteSnippetPermanently,
    emptyTrash,
    type SnippetFilterDto, 
    type SnippetListItemDto,
    type SnippetDto,
    type CreateSnippetDto,
    type UpdateSnippetDto
} from '../ipc/snippets';
import { suggestTags } from '../ipc/tags';
import { refreshUndoState } from './undo';
import { pushNotification } from './notifications';
import { Option, type UnixMs } from '../domain/adts';
import { executeBulkOperation, type BulkOperation } from '../ipc/bulk';

export const snippetsStore = writable<SnippetListItemDto[]>([]);
export const totalCountStore = writable<number>(0);
export const hasMoreStore = writable<boolean>(false);
export const isLoadingMoreStore = writable<boolean>(false);
export const activeSnippetStore = writable<SnippetDto | null>(null);
export const selectedTagStore = writable<string | null>(null);
export const selectedSnippetIdsStore = writable<Set<string>>(new Set());

export const allTagsStore = writable<{ tag: string; count: number }[]>([]);

function handleError(operation: string, e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    console.error(`[snippets] ${operation}:`, e);
    pushNotification({
        id: crypto.randomUUID(),
        severity: 'error',
        title: 'Fehler',
        message: Option.some(`${operation}: ${message}`),
        duration: 5000,
        action: Option.none(),
        createdAt: Date.now() as UnixMs,
    });
}

export async function loadAllTags() {
    try {
        const tagInfos = await suggestTags('', 500);
        allTagsStore.set(tagInfos.map(t => ({ tag: t.name, count: t.usageCount })));
    } catch (e) {
        handleError('Tags laden', e);
    }
}

export const tagCloud = derived(allTagsStore, $tags => $tags);

export function toggleSelectSnippetId(id: string) {
    selectedSnippetIdsStore.update(set => {
        const next = new Set(set);
        if (next.has(id)) {
            next.delete(id);
        } else {
            next.add(id);
        }
        return next;
    });
}

export function clearSnippetSelection() {
    selectedSnippetIdsStore.set(new Set());
}

export function selectAllSnippets(ids: string[]) {
    selectedSnippetIdsStore.set(new Set(ids));
}

export async function handleBulkOperation(operation: BulkOperation) {
    try {
        await executeBulkOperation(operation);
        clearSnippetSelection();
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
    } catch (e) {
        handleError('Massenoperation ausführen', e);
    }
}

export const snippetFilterStore = writable<SnippetFilterDto>({
    searchQuery: '',
    contentTypes: [],
    tags: [],
    locationType: 'all',
    folderId: null,
    isTrashed: false,
    isPinned: null,
    isFavorite: null,
    isTemplate: null,
    tagsMode: 'all',
    dateField: 'updatedAt',
    dateRange: null,
    sizeRange: null,
    sortBy: 'updatedAt',
    sortDir: 'desc'
});

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

export function setSearchQuery(query: string) {
    snippetFilterStore.update(f => ({ ...f, searchQuery: query }));

    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
        loadSnippets();
    }, 250);
}

export function setFilter(patch: Partial<SnippetFilterDto>) {
    if ('searchQuery' in patch) {
        setSearchQuery(patch.searchQuery ?? '');
        const { searchQuery: _, ...rest } = patch;
        if (Object.keys(rest).length > 0) {
            snippetFilterStore.update(f => ({ ...f, ...rest }));
            loadSnippets();
        }
    } else {
        snippetFilterStore.update(f => ({ ...f, ...patch }));
        loadSnippets();
    }
}

export async function loadSnippets() {
    const filter = get(snippetFilterStore);
    const selectedTag = get(selectedTagStore);
    
    const tags = [...(filter.tags || [])];
    if (selectedTag && !tags.includes(selectedTag)) {
        tags.push(selectedTag);
    }

    const finalFilter: SnippetFilterDto = {
        ...filter,
        tags,
        searchQuery: filter.searchQuery?.trim() ? filter.searchQuery.trim() : null,
        offset: 0,
        limit: filter.limit || 50
    };

    try {
        const res = await listSnippets(finalFilter);
        snippetsStore.set(res.items);
        totalCountStore.set(res.totalCount);
        hasMoreStore.set(res.hasMore);
    } catch (e) {
        handleError('Snippets laden', e);
    }
}

export async function loadMoreSnippets() {
    if (get(isLoadingMoreStore) || !get(hasMoreStore)) return;
    isLoadingMoreStore.set(true);

    const currentItems = get(snippetsStore);
    const filter = get(snippetFilterStore);
    const selectedTag = get(selectedTagStore);

    const tags = [...(filter.tags || [])];
    if (selectedTag && !tags.includes(selectedTag)) {
        tags.push(selectedTag);
    }

    const finalFilter: SnippetFilterDto = {
        ...filter,
        tags,
        searchQuery: filter.searchQuery?.trim() ? filter.searchQuery.trim() : null,
        offset: currentItems.length,
        limit: filter.limit || 50
    };

    try {
        const res = await listSnippets(finalFilter);
        snippetsStore.update(items => [...items, ...res.items]);
        totalCountStore.set(res.totalCount);
        hasMoreStore.set(res.hasMore);
    } catch (e) {
        handleError('Weitere Snippets laden', e);
    } finally {
        isLoadingMoreStore.set(false);
    }
}

export async function selectSnippet(id: string | null) {
    if (!id) {
        activeSnippetStore.set(null);
        return;
    }
    try {
        const snippet = await getSnippet(id);
        activeSnippetStore.set(snippet);
    } catch (e) {
        handleError('Snippet-Details laden', e);
    }
}

export async function handleCreateSnippet(draft: CreateSnippetDto): Promise<SnippetDto | null> {
    try {
        const created = await createSnippet(draft);
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
        activeSnippetStore.set(created);
        return created;
    } catch (e) {
        handleError('Snippet erstellen', e);
        return null;
    }
}

export async function handleUpdateSnippet(id: string, draft: UpdateSnippetDto) {
    try {
        const updated = await updateSnippet(id, draft);
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
        activeSnippetStore.set(updated);
    } catch (e) {
        handleError('Snippet aktualisieren', e);
    }
}

export async function togglePinSnippet(item: SnippetListItemDto) {
    try {
        await updateSnippet(item.id, { isPinned: !item.isPinned });
        await Promise.all([loadSnippets(), loadAllTags()]);
        if (get(activeSnippetStore)?.id === item.id) {
            const updated = await getSnippet(item.id);
            activeSnippetStore.set(updated);
        }
    } catch (e) {
        handleError('Anheften umschalten', e);
    }
}

export async function toggleFavoriteSnippet(item: SnippetListItemDto) {
    try {
        await updateSnippet(item.id, { isFavorite: !item.isFavorite });
        await Promise.all([loadSnippets(), loadAllTags()]);
        if (get(activeSnippetStore)?.id === item.id) {
            const updated = await getSnippet(item.id);
            activeSnippetStore.set(updated);
        }
    } catch (e) {
        handleError('Favorit umschalten', e);
    }
}

export async function handleDuplicateSnippet(id: string) {
    try {
        const duplicated = await duplicateSnippet(id);
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
        activeSnippetStore.set(duplicated);
    } catch (e) {
        handleError('Snippet duplizieren', e);
    }
}

export async function handleDuplicateSnippetsBulk(ids: string[], targetFolderId?: string | null) {
    if (ids.length === 0) return;
    try {
        const res = await duplicateSnippetsBulk(ids, targetFolderId);
        clearSnippetSelection();
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
        if (res.succeeded.length > 0) {
            activeSnippetStore.set(res.succeeded[res.succeeded.length - 1]);
        }
        return res;
    } catch (e) {
        handleError('Mehrere Snippets duplizieren', e);
    }
}

export async function handleTrashSnippet(id: string) {
    try {
        await trashSnippet(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
    } catch (e) {
        handleError('Snippet in Papierkorb verschieben', e);
    }
}

export async function handleRestoreSnippet(id: string) {
    try {
        await restoreSnippet(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
    } catch (e) {
        handleError('Snippet wiederherstellen', e);
    }
}

export async function handleDeleteSnippetPermanently(id: string) {
    try {
        await deleteSnippetPermanently(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
    } catch (e) {
        handleError('Snippet dauerhaft löschen', e);
    }
}

export async function handleEmptyTrash() {
    try {
        await emptyTrash();
        activeSnippetStore.set(null);
        await Promise.all([loadSnippets(), loadAllTags(), refreshUndoState()]);
    } catch (e) {
        handleError('Papierkorb leeren', e);
    }
}
