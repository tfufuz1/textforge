import { invoke } from '@tauri-apps/api/core';

export interface UndoStateDto {
  canUndo: boolean;
  canRedo: boolean;
  undoCount: number;
  redoCount: number;
  topUndoDescription?: string;
  topRedoDescription?: string;
}

export async function undo(): Promise<void> {
  return invoke('undo');
}

export async function redo(): Promise<void> {
  return invoke('redo');
}

export async function getUndoState(): Promise<UndoStateDto> {
  return invoke('get_undo_state');
}
