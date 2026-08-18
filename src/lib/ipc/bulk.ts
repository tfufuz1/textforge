import { invoke } from '@tauri-apps/api/core';

export type BulkOperation =
  | { _type: 'bulk_transform'; snippetIds: string[]; pipelineId: string; saveResults: boolean }
  | { _type: 'bulk_tag'; snippetIds: string[]; addTags: string[]; removeTags: string[] }
  | { _type: 'bulk_move'; snippetIds: string[]; targetLocation: any }
  | { _type: 'bulk_delete'; snippetIds: string[]; permanent: boolean }
  | { _type: 'bulk_export'; snippetIds: string[]; format: string; outputPath: string }
  | { _type: 'bulk_pin'; snippetIds: string[]; pinned: boolean }
  | { _type: 'bulk_favorite'; snippetIds: string[]; favorite: boolean };

export interface BulkOperationResult {
  operation: BulkOperation;
  succeeded: string[];
  failed: { id: string; error: any }[];
  totalCount: number;
  durationMs: number;
  previews?: { id: string; preview: string }[];
}

export async function executeBulkOperation(
  operation: BulkOperation
): Promise<BulkOperationResult> {
  return invoke('execute_bulk_operation', { operation });
}
