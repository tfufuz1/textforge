import { invoke } from '@tauri-apps/api/core';

export interface ClipboardEntryListItemDto {
  id: string;
  preview: string;
  contentType: string;
  sourceApp: string | null;
  capturedAt: number;
  sizeBytes: number;
  isPinned: boolean;
  matchScore: number | null;
  promotedToSnippetId: string | null;
}

export interface ClipboardFilterDto {
  searchQuery?: string | null;
  contentTypes?: string[];
  sourceApps?: string[];
}

export interface PagedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export async function listClipboardHistory(
    filter?: ClipboardFilterDto,
    page = 0,
    pageSize = 50
): Promise<PagedResult<ClipboardEntryListItemDto>> {
    return invoke('list_clipboard_history', { filter, page, page_size: pageSize });
}

export async function pinEntry(id: string, pinned: boolean): Promise<void> {
    return invoke('pin_clipboard_entry', { id, pinned });
}

export async function deleteEntry(id: string): Promise<void> {
    return invoke('delete_clipboard_entry', { id });
}

export async function clearHistory(keepPinned = true): Promise<number> {
    return invoke('clear_clipboard_history', { keep_pinned: keepPinned });
}

export async function promoteToSnippet(entryId: string, title: string | null, location: { _type: string, folderId: string | null }): Promise<string> {
    return invoke('promote_clipboard_to_snippet', { entryId, title, location });
}

export async function getClipboardEntry(id: string): Promise<ClipboardEntryListItemDto & { content: string }> {
    return invoke('get_clipboard_entry', { id });
}

export async function readClipboardNow(): Promise<string> {
    return invoke('read_clipboard_now');
}

export async function writeToClipboard(content: string, snippetId?: string | null): Promise<void> {
    return invoke('write_to_clipboard', { content, snippet_id: snippetId ?? null });
}
