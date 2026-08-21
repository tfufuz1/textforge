export type ExportFormat = 'bundle' | 'tfbundle' | 'json' | 'json_array' | 'markdown' | 'text' | 'csv';

export interface ExportRequest {
  readonly exportType?: 'full' | 'snippets' | 'scripts' | 'pipelines';
  readonly includeSnippets: boolean;
  readonly includeScripts: boolean;
  readonly includePipelines: boolean;
  readonly includeClipboard: boolean;
  readonly format: ExportFormat;
  readonly targetPath: string;
}

export type ImportConflictPolicy = 'skip' | 'overwrite' | 'rename' | 'keep_both';

export interface ImportResult {
  readonly success: boolean;
  readonly importedSnippetsCount: number;
  readonly importedScriptsCount: number;
  readonly importedPipelinesCount: number;
  readonly importedFoldersCount: number;
  readonly skipped: number;
  readonly errors: readonly string[];
}

export function createDefaultExportRequest(targetPath: string): ExportRequest {
  return {
    exportType: 'full',
    includeSnippets: true,
    includeScripts: true,
    includePipelines: true,
    includeClipboard: false,
    format: 'tfbundle',
    targetPath,
  };
}

export function validateExportRequest(request: ExportRequest): boolean {
  if (!request.targetPath || request.targetPath.trim() === '') return false;
  const validFormats: ExportFormat[] = ['bundle', 'tfbundle', 'json', 'json_array', 'markdown', 'text', 'csv'];
  return validFormats.includes(request.format);
}
