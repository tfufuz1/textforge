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
        const result = await listClipboardHistory(0, 50);
        clipboardStore.set(result.items);
    } catch (e) {
        console.error("Failed to load clipboard history:", e);
    }
}

export const filteredClipboard = derived(
    [clipboardStore, clipboardFilterStore],
    ([$clipboard, $filter]) => {
        let result = $clipboard;
        if ($filter.searchQuery._tag === 'Some' && $filter.searchQuery.value) {
            const query = $filter.searchQuery.value.toLowerCase();
            result = result.filter(r => r.preview.toLowerCase().includes(query));
        }
        return result;
    }
);
