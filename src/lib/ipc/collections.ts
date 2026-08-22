import { invoke } from '@tauri-apps/api/core';

export interface CollectionTabDto {
  id: string;
  name: string;
  icon: string | null;
  color: string | null;
  sortOrder: number;
  kind: string;
  kindConfig: string | null;
  isPinned: boolean;
  createdAt: number;
  updatedAt: number;
  itemCount: number;
}

export interface CreateCollectionTabDto {
  name: string;
  icon?: string | null;
  color?: string | null;
  kind?: string | null;
  kindConfig?: string | null;
}

export interface ItemRefDto {
  itemKind: string;
  itemId: string;
}

export async function listCollectionTabs(): Promise<CollectionTabDto[]> {
  return invoke('list_collection_tabs');
}

export async function createCollectionTab(draft: CreateCollectionTabDto): Promise<CollectionTabDto> {
  return invoke('create_collection_tab', { draft });
}

export async function deleteCollectionTab(id: string): Promise<void> {
  return invoke('delete_collection_tab', { id });
}

export async function addItemToTab(tabId: string, itemRef: ItemRefDto): Promise<void> {
  return invoke('add_item_to_tab', { tabId, itemRef });
}

export async function removeItemFromTab(tabId: string, itemRef: ItemRefDto): Promise<void> {
  return invoke('remove_item_from_tab', { tabId, itemRef });
}
