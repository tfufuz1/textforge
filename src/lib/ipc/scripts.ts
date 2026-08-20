import { invoke } from '@tauri-apps/api/core';
import type { Script, ScriptVersion } from '../domain/script';

export interface CreateScriptParams {
  name: string;
  description?: string;
  scriptType?: 'js' | 'regex';
  category?: string;
  jsCode?: string;
  regexPattern?: string;
  regexReplacement?: string;
  regexFlags?: string;
  color?: string;
  parametersJson?: string;
  tagsJson?: string;
}

export interface UpdateScriptParams {
  name?: string;
  description?: string;
  scriptType?: 'js' | 'regex';
  category?: string;
  jsCode?: string;
  regexPattern?: string;
  regexReplacement?: string;
  regexFlags?: string;
  isFavorite?: boolean;
  color?: string;
  parametersJson?: string;
  tagsJson?: string;
}

export async function listScripts(): Promise<Script[]> {
  return invoke('list_scripts');
}

export async function getScript(id: string): Promise<Script> {
  return invoke('get_script', { id });
}

export async function createScript(draft: CreateScriptParams): Promise<Script> {
  return invoke('create_script', { draft });
}

export async function updateScript(id: string, draft: UpdateScriptParams): Promise<Script> {
  return invoke('update_script', { id, draft });
}

export async function deleteScript(id: string): Promise<void> {
  return invoke('delete_script', { id });
}

export async function executeScript(req: { scriptId?: string; jsCode?: string; input: string; paramsJson?: string }): Promise<{
  output: string;
  executionTimeMs: number;
  consoleLogs: string[];
  error?: string;
}> {
  return invoke('execute_script', { req });
}

export async function executeBuiltin(builtinId: string, input: string, params?: Record<string, string>): Promise<string> {
  return invoke('execute_builtin', { builtinId, input, params });
}

export async function listScriptVersions(scriptId: string): Promise<ScriptVersion[]> {
  return invoke('list_script_versions', { scriptId });
}

export async function saveScriptVersion(scriptId: string, changeNote = ''): Promise<void> {
  return invoke('save_script_version', { scriptId, changeNote });
}

export async function restoreScriptVersion(scriptId: string, versionId: string | number): Promise<void> {
  return invoke('restore_script_version', { scriptId, versionId: String(versionId) });
}
