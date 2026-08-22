import { invoke } from '@tauri-apps/api/core';

export interface ClipboardIgnoreRuleDto {
  id: string;
  enabled: boolean;
  matchType: string;
  pattern: string;
  createdAt: number;
}

export interface CreateIgnoreRuleDto {
  matchType: string;
  pattern: string;
}

export async function listIgnoreRules(): Promise<ClipboardIgnoreRuleDto[]> {
  return invoke('list_ignore_rules');
}

export async function createIgnoreRule(draft: CreateIgnoreRuleDto): Promise<ClipboardIgnoreRuleDto> {
  return invoke('create_ignore_rule', { draft });
}

export async function deleteIgnoreRule(id: string): Promise<void> {
  return invoke('delete_ignore_rule', { id });
}
