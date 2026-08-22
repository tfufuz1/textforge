import { Option } from './adts';

export type ItemKind = 'snippet' | 'clipboard' | 'script' | 'pipeline';

export interface UnifiedItemFilter {
  readonly searchQuery: Option<string>;
  readonly itemKinds: ReadonlyArray<ItemKind>;
  readonly tags: ReadonlyArray<string>;
  readonly tagsMode: 'all' | 'any' | 'none';
  readonly collectionTabId: Option<string>;
  readonly sortBy: 'relevance' | 'updatedAt' | 'title';
  readonly sortDir: 'asc' | 'desc';
}

export interface UnifiedItemListItem {
  readonly itemKind: ItemKind;
  readonly id: string;
  readonly title: string;
  readonly preview: string;
  readonly highlightedPreview: string;
  readonly tags: readonly string[];
  readonly contentType: string | null;
  readonly updatedAt: number;
  readonly matchScore: number | null;
}

export const UnifiedItemFilter = {
  default: (): UnifiedItemFilter => ({
    searchQuery: Option.none(),
    itemKinds: [],
    tags: [],
    tagsMode: 'all',
    collectionTabId: Option.none(),
    sortBy: 'relevance',
    sortDir: 'desc',
  }),
} as const;
