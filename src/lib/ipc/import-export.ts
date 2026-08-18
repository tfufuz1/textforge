import { invoke } from '@tauri-apps/api/core';

export interface ExportRequestDto {
  exportType?: string;
  format: string;
  targetPath: string;
}

export interface ExportResultDto {
  success: boolean;
  exportedCount: number;
  filePath: string;
}

export interface ImportRequestDto {
  sourcePath: string;
  conflictPolicy?: 'skip' | 'overwrite' | 'rename';
  overwrite?: boolean;
}

export interface ImportResultDto {
  success: boolean;
  snippetsImported: number;
  scriptsImported: number;
  pipelinesImported: number;
  foldersImported: number;
  skipped: number;
}

export interface ImportPreviewDto {
  snippetCount: number;
  scriptCount: number;
  pipelineCount: number;
  folderCount: number;
  createdAt: number;
}

export async function exportData(request: ExportRequestDto): Promise<ExportResultDto> {
  return invoke('export_data', { request });
}

export async function importData(request: ImportRequestDto): Promise<ImportResultDto> {
  return invoke('import_data', { request });
}

export async function previewImport(sourcePath: string): Promise<ImportPreviewDto> {
  return invoke('preview_import', { sourcePath });
}
