import { writable } from 'svelte/store';
import type { TagInfoDto } from '$lib/ipc/tags';
import * as ipc from '$lib/ipc/tags';

export const tagRegistryStore = writable<TagInfoDto[]>([]);

export const tagsActions = {
  loadSuggestions: async (prefix: string = '') => {
    try {
      const tags = await ipc.suggestTags(prefix);
      tagRegistryStore.set(tags);
    } catch (e) {
      console.error('Failed to load tag suggestions:', e);
    }
  },
  rename: async (oldName: string, newName: string) => {
    const res = await ipc.renameTag(oldName, newName);
    await tagsActions.loadSuggestions('');
    return res;
  },
  merge: async (sourceTags: string[], targetTag: string) => {
    const res = await ipc.mergeTags(sourceTags, targetTag);
    await tagsActions.loadSuggestions('');
    return res;
  },
  setColor: async (tagName: string, color: string | null) => {
    await ipc.setTagColor(tagName, color);
    await tagsActions.loadSuggestions('');
  },
};
