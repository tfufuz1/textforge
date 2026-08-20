import { writable, get } from 'svelte/store';
import { getWorkspaceSession, saveWorkspaceSession } from '../ipc/session';
import type { WorkspaceSession } from '../domain/session';

export const defaultSession: WorkspaceSession = {
  activeView: 'clipboard',
  sidebarOpen: true,
};

export const activeSessionStore = writable<WorkspaceSession>(defaultSession);

let saveTimeout: ReturnType<typeof setTimeout> | null = null;

export async function initSession(): Promise<WorkspaceSession> {
  try {
    const saved = await getWorkspaceSession();
    if (saved) {
      activeSessionStore.set(saved);
      return saved;
    }
  } catch (e) {
    console.warn('Failed to load workspace session:', e);
  }
  return defaultSession;
}

export function updateSession(patch: Partial<WorkspaceSession>) {
  activeSessionStore.update((curr) => {
    const updated = { ...curr, ...patch };

    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      saveWorkspaceSession(updated).catch((err) =>
        console.warn('Failed to save workspace session:', err)
      );
    }, 1000);

    return updated;
  });
}
