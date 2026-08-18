import { writable, get } from 'svelte/store';
import { listScripts, createScript, deleteScript, executeScript, updateScript, type ScriptDto, type CreateScriptDto, type ScriptExecutionResultDto } from '../ipc/transform';

export const scriptsStore = writable<ScriptDto[]>([]);
export const activeScriptStore = writable<ScriptDto | null>(null);
export const executionResultStore = writable<ScriptExecutionResultDto | null>(null);

export async function loadScripts() {
  try {
    const list = await listScripts();
    scriptsStore.set(list);
    if (list.length > 0 && !get(activeScriptStore)) {
      activeScriptStore.set(list[0]);
    }
  } catch (e) {
    console.error('Failed to load scripts:', e);
  }
}

export async function handleCreateScript(draft: CreateScriptDto) {
  try {
    const created = await createScript(draft);
    await loadScripts();
    activeScriptStore.set(created);
    return created;
  } catch (e) {
    console.error('Failed to create script:', e);
    return null;
  }
}

export async function handleDeleteScript(id: string) {
  try {
    await deleteScript(id);
    if (get(activeScriptStore)?.id === id) {
      activeScriptStore.set(null);
    }
    await loadScripts();
  } catch (e) {
    console.error('Failed to delete script:', e);
  }
}

export async function handleUpdateScript(id: string, draft: Partial<ScriptDto>) {
  try {
    const updated = await updateScript(id, draft);
    await loadScripts();
    activeScriptStore.set(updated);
  } catch (e) {
    console.error('Failed to update script:', e);
  }
}

export async function handleTestScript(input: string, jsCode?: string) {
  const active = get(activeScriptStore);
  if (!active && !jsCode) return;

  try {
    const result = await executeScript({
      scriptId: active?.id,
      jsCode,
      input
    });
    executionResultStore.set(result);
    return result;
  } catch (e) {
    console.error('Failed to execute script:', e);
    return null;
  }
}
