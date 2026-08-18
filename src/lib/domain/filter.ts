import type { TagName } from './adts';

export interface SnippetFilter {
  readonly search?: string;
  readonly tags: readonly TagName[];
  readonly tagsMode?: 'all' | 'any';
  readonly folderId?: string;
  readonly location?: 'inbox' | 'folder' | 'trash' | 'all';
  readonly isPinned?: boolean;
  readonly isFavorite?: boolean;
  readonly isTemplate?: boolean;
  readonly dateField?: 'createdAt' | 'updatedAt';
  readonly dateRange?: { from?: number; to?: number; preset?: string };
  readonly sizeRange?: { min?: number; max?: number };
  readonly sortBy?: string;
  readonly sortDir?: 'asc' | 'desc';
}

export const SnippetFilter = {
  default: (): SnippetFilter => ({
    tags: [],
    tagsMode: 'all',
    location: 'all',
    dateField: 'updatedAt',
    sortBy: 'updatedAt',
    sortDir: 'desc',
  }),

  merge: (base: SnippetFilter, patch: Partial<SnippetFilter>): SnippetFilter => ({
    ...base,
    ...patch,
  }),
} as const;

export interface ClipboardFilter {
  readonly search?: string;
  readonly sourceApp?: string;
  readonly isPinned?: boolean;
}
