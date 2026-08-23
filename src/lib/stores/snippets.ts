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
import { refreshUndoState } from './undo';

import { executeBulkOperation, type BulkOperation } from '../ipc/bulk';

export const snippetsStore = writable<SnippetListItemDto[]>([]);
export const totalCountStore = writable<number>(0);
export const hasMoreStore = writable<boolean>(false);
export const isLoadingMoreStore = writable<boolean>(false);
export const activeSnippetStore = writable<SnippetDto | null>(null);
export const selectedTagStore = writable<string | null>(null);
export const selectedSnippetIdsStore = writable<Set<string>>(new Set());

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
        await loadSnippets();
        await refreshUndoState();
    } catch (e) {
        console.error("Bulk operation failed:", e);
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
        console.error("Failed to load snippets:", e);
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
        console.error("Failed to load more snippets:", e);
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
        console.error("Failed to load snippet details:", e);
    }
}

export async function handleCreateSnippet(draft: CreateSnippetDto): Promise<SnippetDto | null> {
    try {
        const created = await createSnippet(draft);
        await loadSnippets();
        activeSnippetStore.set(created);
        await refreshUndoState();
        return created;
    } catch (e) {
        console.error("Failed to create snippet:", e);
        return null;
    }
}

export async function handleUpdateSnippet(id: string, draft: UpdateSnippetDto) {
    try {
        const updated = await updateSnippet(id, draft);
        await loadSnippets();
        activeSnippetStore.set(updated);
        await refreshUndoState();
    } catch (e) {
        console.error("Failed to update snippet:", e);
    }
}

export async function togglePinSnippet(item: SnippetListItemDto) {
    try {
        await updateSnippet(item.id, { isPinned: !item.isPinned });
        await loadSnippets();
        if (get(activeSnippetStore)?.id === item.id) {
            const updated = await getSnippet(item.id);
            activeSnippetStore.set(updated);
        }
    } catch (e) {
        console.error("Failed to toggle pin on snippet:", e);
    }
}

export async function toggleFavoriteSnippet(item: SnippetListItemDto) {
    try {
        await updateSnippet(item.id, { isFavorite: !item.isFavorite });
        await loadSnippets();
        if (get(activeSnippetStore)?.id === item.id) {
            const updated = await getSnippet(item.id);
            activeSnippetStore.set(updated);
        }
    } catch (e) {
        console.error("Failed to toggle favorite on snippet:", e);
    }
}

export async function handleDuplicateSnippet(id: string) {
    try {
        const duplicated = await duplicateSnippet(id);
        await loadSnippets();
        activeSnippetStore.set(duplicated);
        await refreshUndoState();
    } catch (e) {
        console.error("Failed to duplicate snippet:", e);
    }
}

export async function handleDuplicateSnippetsBulk(ids: string[], targetFolderId?: string | null) {
    if (ids.length === 0) return;
    try {
        const res = await duplicateSnippetsBulk(ids, targetFolderId);
        clearSnippetSelection();
        await loadSnippets();
        if (res.succeeded.length > 0) {
            activeSnippetStore.set(res.succeeded[res.succeeded.length - 1]);
        }
        await refreshUndoState();
        return res;
    } catch (e) {
        console.error("Failed to duplicate snippets bulk:", e);
    }
}

export async function handleTrashSnippet(id: string) {
    try {
        await trashSnippet(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await loadSnippets();
        await refreshUndoState();
    } catch (e) {
        console.error("Failed to trash snippet:", e);
    }
}

export async function handleRestoreSnippet(id: string) {
    try {
        await restoreSnippet(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await loadSnippets();
        await refreshUndoState();
    } catch (e) {
        console.error("Failed to restore snippet:", e);
    }
}

export async function handleDeleteSnippetPermanently(id: string) {
    try {
        await deleteSnippetPermanently(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await loadSnippets();
        await refreshUndoState();
    } catch (e) {
        console.error("Failed to delete snippet permanently:", e);
    }
}

export async function handleEmptyTrash() {
    try {
        await emptyTrash();
        activeSnippetStore.set(null);
        await loadSnippets();
        await refreshUndoState();
    } catch (e) {
        console.error("Failed to empty trash:", e);
    }
}

export const tagCloud = derived(snippetsStore, $snippets => {
    const tagsMap = new Map<string, number>();
    for (const item of $snippets) {
        for (const tag of item.tags) {
            tagsMap.set(tag, (tagsMap.get(tag) || 0) + 1);
        }
    }
    return Array.from(tagsMap.entries()).map(([tag, count]) => ({ tag, count }));
});