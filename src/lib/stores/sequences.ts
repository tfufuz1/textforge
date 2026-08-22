import { writable } from 'svelte/store';
import type { SequenceDto, CreateSequenceDto } from '$lib/ipc/sequences';
import * as ipc from '$lib/ipc/sequences';

export const sequencesStore = writable<SequenceDto[]>([]);

export const sequencesActions = {
  loadAll: async () => {
    try {
      const seqs = await ipc.listSequences();
      sequencesStore.set(seqs);
    } catch (e) {
      console.error('Failed to load sequences:', e);
    }
  },
  create: async (draft: CreateSequenceDto) => {
    const created = await ipc.createSequence(draft);
    sequencesStore.update(s => [created, ...s]);
    return created;
  },
  delete: async (id: string) => {
    await ipc.deleteSequence(id);
    sequencesStore.update(s => s.filter(it => it.id !== id));
  },
};
