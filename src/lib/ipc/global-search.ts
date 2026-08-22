import { invoke } from '@tauri-apps/api/core';

export interface UnifiedItemListItemDto {
  itemKind: string;
  id: string;
  title: string;
  preview: string;
  highlightedPreview: string;
  tags: string[];
  contentType: string | null;
  updatedAt: number;
  matchScore: number | null;
}

export interface UnifiedItemFilterDto {
  searchQuery?: string | null;
  itemKinds?: string[];
  tags?: string[];
  tagsMode?: string | null;
}

export async function searchAllItems(filter: UnifiedItemFilterDto): Promise<UnifiedItemListItemDto[]> {
  return invoke('search_all_items', { filter });
}
