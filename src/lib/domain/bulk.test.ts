import { describe, it, expect } from 'vitest';
import type { BulkOperation } from '../ipc/bulk';

describe('Bulk Operations Domain Logic', () => {
  it('constructs bulk_pin operation correctly', () => {
    const op: BulkOperation = {
      _type: 'bulk_pin',
      snippetIds: ['s1', 's2'],
      pinned: true
    };
    expect(op._type).toBe('bulk_pin');
    expect(op.snippetIds).toHaveLength(2);
    expect(op.pinned).toBe(true);
  });

  it('constructs bulk_transform operation correctly', () => {
    const op: BulkOperation = {
      _type: 'bulk_transform',
      snippetIds: ['s1'],
      pipelineId: 'p1',
      saveResults: false
    };
    expect(op._type).toBe('bulk_transform');
    expect(op.pipelineId).toBe('p1');
    expect(op.saveResults).toBe(false);
  });

  it('constructs bulk_tag operation correctly', () => {
    const op: BulkOperation = {
      _type: 'bulk_tag',
      snippetIds: ['s1', 's2', 's3'],
      addTags: ['important', 'work'],
      removeTags: ['draft']
    };
    expect(op._type).toBe('bulk_tag');
    expect(op.addTags).toEqual(['important', 'work']);
    expect(op.removeTags).toEqual(['draft']);
  });
});
