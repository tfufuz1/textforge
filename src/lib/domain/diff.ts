export type DiffLineKind = 'equal' | 'delete' | 'insert';

export interface DiffLine {
  readonly kind: DiffLineKind;
  readonly text: string;
  readonly oldLineNumber?: number;
  readonly newLineNumber?: number;
}

export interface DiffResult {
  readonly lines: readonly DiffLine[];
  readonly additions: number;
  readonly deletions: number;
  readonly unchanged: number;
  readonly similarity: number;
  readonly isIdentical: boolean;
}

export function computeSimilarityRatio(unchanged: number, total: number): number {
  if (total <= 0) return 1.0;
  return Math.min(1.0, Math.max(0.0, unchanged / total));
}
