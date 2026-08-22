import { invoke } from '@tauri-apps/api/core';

export interface AutomationRuleDto {
  id: string;
  name: string;
  enabled: boolean;
  trigger: string;
  condition: string | null;
  scriptId: string;
  sortOrder: number;
  createdAt: number;
  updatedAt: number;
}

export interface CreateAutomationRuleDto {
  name: string;
  trigger: string;
  condition?: string | null;
  scriptId: string;
}

export async function listAutomationRules(): Promise<AutomationRuleDto[]> {
  return invoke('list_automation_rules');
}

export async function createAutomationRule(draft: CreateAutomationRuleDto): Promise<AutomationRuleDto> {
  return invoke('create_automation_rule', { draft });
}

export async function deleteAutomationRule(id: string): Promise<void> {
  return invoke('delete_automation_rule', { id });
}
