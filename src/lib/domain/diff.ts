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
  readonly isIdentical: boolean;
}
