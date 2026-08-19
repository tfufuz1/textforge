import { writable, derived, get } from 'svelte/store';
import type { ClipboardEntryListItemDto } from '../ipc/clipboard';
import { listClipboardHistory } from '../ipc/clipboard';
import { Option } from '../domain/adts';

export const clipboardStore = writable<ClipboardEntryListItemDto[]>([]);

export interface ClipboardFilter {
    searchQuery: Option<string>;
    contentTypes: string[];
    sourceApps: string[];
}

export const clipboardFilterStore = writable<ClipboardFilter>({
    searchQuery: Option.none(),
    contentTypes: [],
    sourceApps: []
});

export async function loadClipboardHistory() {
    try {
        const filterState = get(clipboardFilterStore);
        const filterDto = {
            searchQuery: filterState.searchQuery._tag === 'Some' ? filterState.searchQuery.value : null,
            contentTypes: filterState.contentTypes,
            sourceApps: filterState.sourceApps,
        };
        const result = await listClipboardHistory(filterDto, 0, 50);
        clipboardStore.set(result.items);
    } catch (e) {
        console.error("Failed to load clipboard history:", e);
    }
}

export const filteredClipboard = derived(
    [clipboardStore],
    ([$clipboard]) => $clipboard
);
