import { writable, derived, get } from 'svelte/store';
import { 
    listSnippets, 
    getSnippet, 
    createSnippet, 
    updateSnippet, 
    trashSnippet,
    duplicateSnippet,
    type SnippetFilterDto, 
    type SnippetListItemDto,
    type SnippetDto,
    type CreateSnippetDto,
    type UpdateSnippetDto
} from '../ipc/snippets';

export const snippetsStore = writable<SnippetListItemDto[]>([]);
export const activeSnippetStore = writable<SnippetDto | null>(null);
export const selectedTagStore = writable<string | null>(null);

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
        searchQuery: filter.searchQuery?.trim() ? filter.searchQuery.trim() : null
    };

    try {
        const items = await listSnippets(finalFilter);
        snippetsStore.set(items);
    } catch (e) {
        console.error("Failed to load snippets:", e);
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
    } catch (e) {
        console.error("Failed to duplicate snippet:", e);
    }
}

export async function handleTrashSnippet(id: string) {
    try {
        await trashSnippet(id);
        if (get(activeSnippetStore)?.id === id) {
            activeSnippetStore.set(null);
        }
        await loadSnippets();
    } catch (e) {
        console.error("Failed to trash snippet:", e);
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