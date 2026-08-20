import { invoke } from '@tauri-apps/api/core';
import type { WorkspaceSession } from '../domain/session';

export interface DatabaseStats {
  totalSnippets?: number;
  totalClipboardEntries?: number;
  totalScripts?: number;
  totalPipelines?: number;
  dbSizeBytes?: number;
  snippetsCount?: number;
  clipboardEntriesCount?: number;
  scriptsCount?: number;
  pipelinesCount?: number;
  databaseSizeBytes?: number;
}

export async function getWorkspaceSession(): Promise<WorkspaceSession> {
  return invoke('get_workspace_session');
}

export async function saveWorkspaceSession(session: WorkspaceSession): Promise<void> {
  return invoke('save_workspace_session', { session });
}

export async function getDatabaseStats(): Promise<DatabaseStats> {
  return invoke('get_database_stats');
}

export async function getAllSettings(): Promise<Record<string, string>> {
  return invoke('get_all_settings');
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_setting', { key, value });
}
