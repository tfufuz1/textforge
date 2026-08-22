import { writable } from 'svelte/store';
import type { ClipboardIgnoreRuleDto, CreateIgnoreRuleDto } from '$lib/ipc/ignore-rules';
import * as ipc from '$lib/ipc/ignore-rules';

export const ignoreRulesStore = writable<ClipboardIgnoreRuleDto[]>([]);

export const ignoreRulesActions = {
  loadAll: async () => {
    try {
      const rules = await ipc.listIgnoreRules();
      ignoreRulesStore.set(rules);
    } catch (e) {
      console.error('Failed to load ignore rules:', e);
    }
  },
  create: async (draft: CreateIgnoreRuleDto) => {
    const created = await ipc.createIgnoreRule(draft);
    ignoreRulesStore.update(r => [created, ...r]);
    return created;
  },
  delete: async (id: string) => {
    await ipc.deleteIgnoreRule(id);
    ignoreRulesStore.update(r => r.filter(it => it.id !== id));
  },
};
