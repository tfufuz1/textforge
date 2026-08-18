import { invoke } from '@tauri-apps/api/core';
import type { DiffResult } from '../domain/diff';

export async function computeDiff(original: string, modified: string): Promise<DiffResult> {
  return invoke('compute_diff', { original, modified });
}
