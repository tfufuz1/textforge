import { describe, it, expect } from 'vitest';
import { computeSimilarityRatio, type DiffResult } from './diff';

describe('Diff Domain Logic', () => {
  it('computes similarity ratio correctly', () => {
    expect(computeSimilarityRatio(10, 10)).toBe(1.0);
    expect(computeSimilarityRatio(5, 10)).toBe(0.5);
    expect(computeSimilarityRatio(0, 10)).toBe(0.0);
    expect(computeSimilarityRatio(0, 0)).toBe(1.0);
  });

  it('handles DiffResult interface properly', () => {
    const res: DiffResult = {
      lines: [
        { kind: 'equal', text: 'hello' },
        { kind: 'insert', text: 'world' }
      ],
      additions: 1,
      deletions: 0,
      unchanged: 1,
      similarity: 0.5,
      isIdentical: false
    };

    expect(res.isIdentical).toBe(false);
    expect(res.additions).toBe(1);
    expect(res.similarity).toBe(0.5);
  });
});
