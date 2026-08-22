import { writable } from 'svelte/store';
import type { AutomationRuleDto, CreateAutomationRuleDto } from '$lib/ipc/automation';
import * as ipc from '$lib/ipc/automation';

export const automationRulesStore = writable<AutomationRuleDto[]>([]);

export const automationActions = {
  loadAll: async () => {
    try {
      const rules = await ipc.listAutomationRules();
      automationRulesStore.set(rules);
    } catch (e) {
      console.error('Failed to load automation rules:', e);
    }
  },
  create: async (draft: CreateAutomationRuleDto) => {
    const created = await ipc.createAutomationRule(draft);
    automationRulesStore.update(r => [...r, created]);
    return created;
  },
  delete: async (id: string) => {
    await ipc.deleteAutomationRule(id);
    automationRulesStore.update(r => r.filter(it => it.id !== id));
  },
};
