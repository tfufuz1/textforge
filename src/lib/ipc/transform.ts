import { invoke } from '@tauri-apps/api/core';

export interface ScriptDto {
  id: string;
  name: string;
  description: string;
  scriptType: string;
  category: string;
  jsCode: string | null;
  regexPattern: string | null;
  regexReplacement: string | null;
  regexFlags: string;
  isFavorite: boolean;
  usageCount: number;
  currentVersion: number;
  color: string;
  parametersJson: string;
  tagsJson: string;
  createdAt: number;
  updatedAt: number;
}

export interface CreateScriptDto {
  name: string;
  description?: string;
  scriptType?: string;
  category?: string;
  jsCode?: string;
  regexPattern?: string;
  regexReplacement?: string;
  regexFlags?: string;
  color?: string;
  parametersJson?: string;
  tagsJson?: string;
}

export interface ScriptExecutionResultDto {
  output: string;
  executionTimeMs: number;
  error: string | null;
  logs: string[];
}

export interface PipelineStepDto {
  id: string;
  pipelineId: string;
  scriptId: string | null;
  stepOrder: number;
  label: string;
  enabled: boolean;
}

export interface PipelineDto {
  id: string;
  name: string;
  description: string;
  isFavorite: boolean;
  createdAt: number;
  updatedAt: number;
  steps: PipelineStepDto[];
}

export interface CreatePipelineDto {
  name: string;
  description?: string;
}

export interface PipelineStepResultDto {
  stepId: string;
  stepLabel: string;
  output: string;
  executionTimeMs: number;
  error: string | null;
}

export interface PipelineExecutionResultDto {
  finalOutput: string;
  stepResults: PipelineStepResultDto[];
  totalTimeMs: number;
  isSuccess: boolean;
}

export async function listScripts(): Promise<ScriptDto[]> {
  return invoke('list_scripts');
}

export async function createScript(draft: CreateScriptDto): Promise<ScriptDto> {
  return invoke('create_script', { draft });
}

export async function deleteScript(id: string): Promise<void> {
  return invoke('delete_script', { id });
}

export async function listPipelines(): Promise<PipelineDto[]> {
  return invoke('list_pipelines');
}

export async function createPipeline(draft: CreatePipelineDto): Promise<PipelineDto> {
  return invoke('create_pipeline', { draft });
}

export async function deletePipeline(id: string): Promise<void> {
  return invoke('delete_pipeline', { id });
}

export interface UpdateScriptDto {
  name?: string;
  description?: string;
  scriptType?: string;
  category?: string;
  jsCode?: string | null;
  regexPattern?: string | null;
  regexReplacement?: string | null;
  regexFlags?: string;
  isFavorite?: boolean;
  color?: string;
  parametersJson?: string;
  tagsJson?: string;
}

export async function updateScript(id: string, draft: UpdateScriptDto): Promise<ScriptDto> {
  return invoke('update_script', { id, draft });
}

export async function executeScript(req: {
  scriptId?: string;
  jsCode?: string;
  regexPattern?: string;
  regexReplacement?: string;
  regexFlags?: string;
  input: string;
  paramsJson?: string;
}): Promise<ScriptExecutionResultDto> {
  return invoke('execute_script', { req });
}

export async function runPipeline(pipelineId: string, input: string): Promise<PipelineExecutionResultDto> {
  return invoke('run_pipeline', { pipelineId, input });
}
