import { invoke } from '@tauri-apps/api/core';
import type { Pipeline, PipelineStep, PipelineExecutionResult } from '../domain/pipeline';

export interface CreatePipelineParams {
  name: string;
  description?: string;
}

export interface UpdatePipelineParams {
  name?: string;
  description?: string;
  isFavorite?: boolean;
}

export async function listPipelines(): Promise<Pipeline[]> {
  return invoke('list_pipelines');
}

export async function getPipeline(id: string): Promise<Pipeline> {
  return invoke('get_pipeline', { id });
}

export async function createPipeline(draft: CreatePipelineParams): Promise<Pipeline> {
  return invoke('create_pipeline', { draft });
}

export async function updatePipeline(id: string, draft: UpdatePipelineParams): Promise<Pipeline> {
  return invoke('update_pipeline', { id, draft });
}

export async function deletePipeline(id: string): Promise<void> {
  return invoke('delete_pipeline', { id });
}

export async function runPipeline(pipelineId: string, input: string): Promise<PipelineExecutionResult> {
  return invoke('run_pipeline', { pipelineId, input });
}

export async function addPipelineStep(
  pipelineId: string,
  scriptId?: string,
  label = 'New Step',
  order?: number
): Promise<PipelineStep> {
  return invoke('add_pipeline_step', { pipelineId, scriptId, label, order });
}

export async function removePipelineStep(stepId: string): Promise<void> {
  return invoke('remove_pipeline_step', { stepId });
}

export async function reorderPipelineSteps(pipelineId: string, stepIds: string[]): Promise<void> {
  return invoke('reorder_pipeline_steps', { pipelineId, stepIds });
}

export async function togglePipelineStep(stepId: string, enabled: boolean): Promise<void> {
  return invoke('toggle_pipeline_step', { stepId, enabled });
}
