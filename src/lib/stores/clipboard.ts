import { writable, derived, get } from 'svelte/store';
import type { ClipboardEntryListItemDto } from '../ipc/clipboard';
import { listClipboardHistory } from '../ipc/clipboard';
import { Option } from '../domain/adts';

export const clipboardStore = writable<ClipboardEntryListItemDto[]>([]);

export interface ClipboardFilter {
    searchQuery: Option<string>;
    contentTypes: string[];
    sourceApps: string[];
    isPinned: Option<boolean>;
}

export const clipboardFilterStore = writable<ClipboardFilter>({
    searchQuery: Option.none(),
    contentTypes: [],
    sourceApps: [],
    isPinned: Option.none()
});

export const clipboardPageStore = writable<number>(0);
export const clipboardTotalStore = writable<number>(0);
export const clipboardHasNextStore = writable<boolean>(false);
export const clipboardHasPrevStore = writable<boolean>(false);

export async function loadClipboardHistory(page = 0) {
    try {
        const filterState = get(clipboardFilterStore);
        const filterDto = {
            searchQuery: filterState.searchQuery._tag === 'Some' ? filterState.searchQuery.value : null,
            contentTypes: filterState.contentTypes,
            sourceApps: filterState.sourceApps,
            isPinned: filterState.isPinned._tag === 'Some' ? filterState.isPinned.value : null,
        };
        const result = await listClipboardHistory(filterDto, page, 50);
        clipboardStore.set(result.items);
        clipboardPageStore.set(result.page);
        clipboardTotalStore.set(result.total);
        clipboardHasNextStore.set(result.hasNext);
        clipboardHasPrevStore.set(result.hasPrev);
    } catch (e) {
        console.error("Failed to load clipboard history:", e);
    }
}

export const filteredClipboard = derived(
    [clipboardStore],
    ([$clipboard]) => $clipboard
);

export const recentClipboardItems = derived(
    [clipboardStore],
    ([$clipboard]) => $clipboard.slice(0, 10)
);

export const pinnedClipboardItems = derived(
    [clipboardStore],
    ([$clipboard]) => $clipboard.filter(item => item.isPinned)
);
