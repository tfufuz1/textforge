export type ExportFormat = 'json' | 'markdown' | 'txt';

export interface ExportRequest {
  readonly includeSnippets: boolean;
  readonly includeScripts: boolean;
  readonly includePipelines: boolean;
  readonly includeClipboard: boolean;
  readonly format: ExportFormat;
}

export type ImportConflictPolicy = 'skip' | 'overwrite' | 'keep_both';

export interface ImportResult {
  readonly importedSnippetsCount: number;
  readonly importedScriptsCount: number;
  readonly importedPipelinesCount: number;
  readonly errors: readonly string[];
}
