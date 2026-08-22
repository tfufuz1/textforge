import { invoke } from '@tauri-apps/api/core';

export interface SequenceItemDto {
  id: string;
  orderIndex: number;
  refType: string;
  refId: string | null;
  literalText: string | null;
  pipelineId: string | null;
  prefixOverride: string | null;
  suffixOverride: string | null;
  enabled: boolean;
}

export interface SequenceDto {
  id: string;
  name: string;
  separator: string;
  favorite: boolean;
  createdAt: number;
  updatedAt: number;
  items: SequenceItemDto[];
}

export interface CreateSequenceDto {
  name: string;
  separator?: string | null;
  items: SequenceItemDto[];
}

export async function listSequences(): Promise<SequenceDto[]> {
  return invoke('list_sequences');
}

export async function createSequence(draft: CreateSequenceDto): Promise<SequenceDto> {
  return invoke('create_sequence', { draft });
}

export async function deleteSequence(id: string): Promise<void> {
  return invoke('delete_sequence', { id });
}

export async function quickCombine(texts: string[], separator?: string | null): Promise<string> {
  return invoke('quick_combine', { texts, separator });
}
