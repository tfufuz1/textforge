import { invoke } from '@tauri-apps/api/core';

export interface TagInfoDto {
  name: string;
  color: string | null;
  usageCount: number;
  lastUsedAt: number;
  createdAt: number;
}

export interface TagRenameResultDto {
  oldName: string;
  newName: string;
  affectedItems: number;
}

export interface TagMergeResultDto {
  sourceTags: string[];
  targetTag: string;
  affectedItems: number;
}

export async function suggestTags(prefix: string, limit: number = 20): Promise<TagInfoDto[]> {
  return invoke('suggest_tags', { prefix, limit });
}

export async function renameTag(oldName: string, newName: string): Promise<TagRenameResultDto> {
  return invoke('rename_tag', { oldName, newName });
}

export async function mergeTags(sourceTags: string[], targetTag: string): Promise<TagMergeResultDto> {
  return invoke('merge_tags', { sourceTags, targetTag });
}

export async function setTagColor(tagName: string, color: string | null): Promise<void> {
  return invoke('set_tag_color', { tagName, color });
}
