import { writable } from 'svelte/store';
import { undo as ipcUndo, redo as ipcRedo, getUndoState as ipcGetUndoState, pushUndoEntry as ipcPushUndoEntry } from '../ipc/undo';
import type { UndoStateDto } from '../ipc/undo';

export const undoStateStore = writable<UndoStateDto>({
  canUndo: false,
  canRedo: false,
  undoCount: 0,
  redoCount: 0,
  topUndoDescription: undefined,
  topRedoDescription: undefined,
});

export async function refreshUndoState(): Promise<UndoStateDto> {
  try {
    const state = await ipcGetUndoState();
    undoStateStore.set(state);
    return state;
  } catch (e) {
    return {
      canUndo: false,
      canRedo: false,
      undoCount: 0,
      redoCount: 0,
      topUndoDescription: undefined,
      topRedoDescription: undefined,
    };
  }
}

export async function performUndo(): Promise<boolean> {
  try {
    await ipcUndo();
    await refreshUndoState();
    return true;
  } catch (e) {
    console.error('Undo failed:', e);
    return false;
  }
}

export async function performRedo(): Promise<boolean> {
  try {
    await ipcRedo();
    await refreshUndoState();
    return true;
  } catch (e) {
    console.error('Redo failed:', e);
    return false;
  }
}

export async function pushUndoAction(action: any, description: string = 'Aktion durchgeführt') {
  try {
    await ipcPushUndoEntry({
      description,
      action,
    });
    await refreshUndoState();
  } catch (e) {
    console.error('Failed to push undo action:', e);
  }
}