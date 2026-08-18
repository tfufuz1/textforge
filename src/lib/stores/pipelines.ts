import { writable, get } from 'svelte/store';
import { listPipelines, createPipeline, deletePipeline, runPipeline, type PipelineDto, type CreatePipelineDto, type PipelineExecutionResultDto } from '../ipc/transform';

export const pipelinesStore = writable<PipelineDto[]>([]);
export const activePipelineStore = writable<PipelineDto | null>(null);
export const pipelineResultStore = writable<PipelineExecutionResultDto | null>(null);

export async function loadPipelines() {
  try {
    const list = await listPipelines();
    pipelinesStore.set(list);
    if (list.length > 0 && !get(activePipelineStore)) {
      activePipelineStore.set(list[0]);
    }
  } catch (e) {
    console.error('Failed to load pipelines:', e);
  }
}

export async function handleCreatePipeline(draft: CreatePipelineDto) {
  try {
    const created = await createPipeline(draft);
    await loadPipelines();
    activePipelineStore.set(created);
    return created;
  } catch (e) {
    console.error('Failed to create pipeline:', e);
    return null;
  }
}

export async function handleDeletePipeline(id: string) {
  try {
    await deletePipeline(id);
    if (get(activePipelineStore)?.id === id) {
      activePipelineStore.set(null);
    }
    await loadPipelines();
  } catch (e) {
    console.error('Failed to delete pipeline:', e);
  }
}

export async function handleRunPipeline(pipelineId: string, input: string) {
  try {
    const result = await runPipeline(pipelineId, input);
    pipelineResultStore.set(result);
    return result;
  } catch (e) {
    console.error('Failed to run pipeline:', e);
    return null;
  }
}
