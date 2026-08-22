import { writable } from 'svelte/store';
import type { UnifiedItemListItemDto } from '$lib/ipc/global-search';
import * as ipc from '$lib/ipc/global-search';

export const globalSearchResultsStore = writable<UnifiedItemListItemDto[]>([]);
export const globalSearchQueryStore = writable<string>('');

export const globalSearchActions = {
  search: async (query: string, itemKinds: string[] = []) => {
    globalSearchQueryStore.set(query);
    if (!query.trim()) {
      globalSearchResultsStore.set([]);
      return;
    }
    try {
      const results = await ipc.searchAllItems({ searchQuery: query, itemKinds });
      globalSearchResultsStore.set(results);
    } catch (e) {
      console.error('Global search error:', e);
    }
  },
};
