import { invoke } from '@tauri-apps/api/core';

export interface SnippetFilterDto {
  searchQuery?: string | null;
  contentTypes?: string[];
  tags?: string[];
  locationType?: string | null;
  folderId?: string | null;
  isTrashed?: boolean | null;
  isPinned?: boolean | null;
  isFavorite?: boolean | null;
  isTemplate?: boolean | null;
  tagsMode?: string | null;
  dateField?: string | null;
  dateRange?: { from?: number | null; to?: number | null; preset?: string | null } | null;
  sizeRange?: { min?: number | null; max?: number | null } | null;
  sortBy?: string | null;
  sortDir?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface SnippetListResultDto {
  items: SnippetListItemDto[];
  totalCount: number;
  hasMore: boolean;
}

export interface SnippetListItemDto {
  id: string;
  title: string;
  preview: string;
  contentType: string;
  createdAt: number;
  updatedAt: number;
  isPinned: boolean;
  isFavorite: boolean;
  color: string | null;
  tags: string[];
}

export interface DuplicateSnippetsResultDto {
  succeeded: SnippetDto[];
  failed: { id: string; error: any }[];
}

export interface SnippetDto {
  id: string;
  title: string;
  content: string;
  contentType: string;
  sourceApp: string | null;
  locationType: string;
  folderId: string | null;
  createdAt: number;
  updatedAt: number;
  lastUsedAt: number | null;
  usageCount: number;
  isPinned: boolean;
  isTemplate: boolean;
  isFavorite: boolean;
  color: string | null;
  tags: string[];
}

export interface CreateSnippetDto {
  title: string;
  content: string;
  contentType?: string | null;
  tags?: string[];
  folderId?: string | null;
}

export interface UpdateSnippetDto {
  title?: string | null;
  content?: string | null;
  contentType?: string | null;
  tags?: string[];
  isPinned?: boolean | null;
  isFavorite?: boolean | null;
  color?: string | null;
  folderId?: string | null;
}

export interface FolderDto {
  id: string;
  name: string;
  parentId: string | null;
  icon: string | null;
  createdAt: number;
}

export interface TextStatsDto {
  charCount: number;
  charNoSpaceCount: number;
  wordCount: number;
  lineCount: number;
  paragraphCount: number;
  sentenceCount: number;
  estimatedTokens: number;
  uniqueWordCount: number;
  avgWordLength: number;
  longestWord: string;
  mostFrequentWords: { word: string; count: number }[];
  avgSentenceLength: number;
  fleschKincaidGrade: number | null;
  avgLineLength: number;
  longestLineLength: number;
  emptyLineCount: number;
  readingTimeMs: number;
}

export async function listSnippets(filter?: SnippetFilterDto): Promise<SnippetListResultDto> {
  return invoke('list_snippets', { filter });
}

export async function getSnippet(id: string): Promise<SnippetDto> {
  return invoke('get_snippet', { id });
}

export async function createSnippet(draft: CreateSnippetDto): Promise<SnippetDto> {
  return invoke('create_snippet', { draft });
}

export async function updateSnippet(id: string, draft: UpdateSnippetDto): Promise<SnippetDto> {
  return invoke('update_snippet', { id, draft });
}

export async function duplicateSnippet(id: string): Promise<SnippetDto> {
  return invoke('duplicate_snippet', { id });
}

export async function duplicateSnippetsBulk(ids: string[], targetFolderId?: string | null): Promise<DuplicateSnippetsResultDto> {
  return invoke('duplicate_snippets_bulk', { ids, targetFolderId });
}

export async function trashSnippet(id: string): Promise<void> {
  return invoke('trash_snippet', { id });
}

export async function restoreSnippet(id: string): Promise<void> {
  return invoke('restore_snippet', { id });
}

export async function deleteSnippetPermanently(id: string): Promise<void> {
  return invoke('delete_snippet_permanently', { id });
}

export async function emptyTrash(): Promise<number> {
  return invoke('empty_trash');
}

export async function listAllTags(): Promise<string[]> {
  return invoke('list_all_tags');
}

export async function listFolders(): Promise<FolderDto[]> {
  return invoke('list_folders');
}

export async function createFolder(name: string, parentId?: string | null, icon?: string | null): Promise<FolderDto> {
  return invoke('create_folder', { name, parentId, icon });
}

export async function renameFolder(id: string, name: string): Promise<void> {
  return invoke('rename_folder', { id, name });
}

export async function deleteFolder(id: string): Promise<void> {
  return invoke('delete_folder', { id });
}

export async function computeTextStats(content: string): Promise<TextStatsDto> {
  return invoke('compute_text_stats', { content });
}

export async function writeToClipboard(content: string, snippetId?: string | null): Promise<void> {
  return invoke('write_to_clipboard', { content, snippetId });
}

export interface ParsedTemplateDto {
  variables: {
    name: string;
    hasDefault: boolean;
    defaultVal: string | null;
    filter: string | null;
    isSpecial: boolean;
    isRequired: boolean;
    occurrences: number;
  }[];
  requiredVars: string[];
  optionalVars: string[];
  hasConditionals: boolean;
  hasLoops: boolean;
}

export interface TemplateRenderResultDto {
  output: string;
  resolvedVariables: Record<string, string>;
  unresolvedVars: string[];
  warnings: string[];
}

export async function parseTemplate(content: string): Promise<ParsedTemplateDto> {
  return invoke('parse_template', { content });
}

export async function renderTemplate(content: string, context: Record<string, string>, strict: boolean): Promise<TemplateRenderResultDto> {
  return invoke('render_template', { content, context, strict });
}