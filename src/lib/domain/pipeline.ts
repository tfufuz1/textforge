import type { PipelineId, ScriptId } from './adts';
import { Result } from './adts';
import type { DomainError } from './errors';

export interface PipelineStep {
  readonly id: string;
  readonly pipelineId: PipelineId;
  readonly scriptId?: ScriptId;
  readonly stepOrder: number;
  readonly label: string;
  readonly enabled: boolean;
  readonly failurePolicy: 'abort' | 'warn' | 'passthrough';
  readonly conditionJson?: string;
}

export interface Pipeline {
  readonly id: PipelineId;
  readonly name: string;
  readonly description: string;
  readonly isFavorite: boolean;
  readonly createdAt: number;
  readonly updatedAt: number;
  readonly steps: readonly PipelineStep[];
}

export interface PipelineStepExecutionResult {
  readonly stepId: string;
  readonly stepLabel: string;
  readonly output: string;
  readonly executionTimeMs: number;
  readonly error?: string;
  readonly wasSkipped: boolean;
  readonly conditionResult?: boolean;
  readonly failurePolicy: string;
}

export interface PipelineExecutionResult {
  readonly finalOutput: string;
  readonly stepResults: readonly PipelineStepExecutionResult[];
  readonly totalTimeMs: number;
  readonly isSuccess: boolean;
  readonly skippedSteps: readonly string[];
}

export const Pipeline = {
  validateName: (name: string): Result<DomainError, string> => {
    const trimmed = name.trim();
    if (!trimmed) return Result.err({ code: 'EMPTY_PIPELINE_NAME' });
    return Result.ok(trimmed);
  },
} as const;
