import { writable } from 'svelte/store';
import type { CollectionTabDto, CreateCollectionTabDto, ItemRefDto } from '$lib/ipc/collections';
import * as ipc from '$lib/ipc/collections';

export const collectionTabsStore = writable<CollectionTabDto[]>([]);
export const activeCollectionTabId = writable<string>('default');

export const collectionsActions = {
  loadAll: async () => {
    try {
      const tabs = await ipc.listCollectionTabs();
      collectionTabsStore.set(tabs);
    } catch (e) {
      console.error('Failed to load collection tabs:', e);
    }
  },
  create: async (draft: CreateCollectionTabDto) => {
    const created = await ipc.createCollectionTab(draft);
    collectionTabsStore.update(tabs => [...tabs, created]);
    return created;
  },
  delete: async (id: string) => {
    await ipc.deleteCollectionTab(id);
    collectionTabsStore.update(tabs => tabs.filter(t => t.id !== id));
  },
  addItem: async (tabId: string, itemRef: ItemRefDto) => {
    await ipc.addItemToTab(tabId, itemRef);
    await collectionsActions.loadAll();
  },
  removeItem: async (tabId: string, itemRef: ItemRefDto) => {
    await ipc.removeItemFromTab(tabId, itemRef);
    await collectionsActions.loadAll();
  },
};
