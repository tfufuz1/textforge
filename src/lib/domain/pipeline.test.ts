import { describe, it, expect } from 'vitest';
import { Pipeline } from './pipeline';

describe('Pipeline Domain', () => {
  it('validateName rejects empty or whitespace names', () => {
    const res1 = Pipeline.validateName('');
    expect(res1._tag).toBe('Err');
    if (res1._tag === 'Err') {
      expect(res1.error.code).toBe('EMPTY_PIPELINE_NAME');
    }

    const res2 = Pipeline.validateName('   ');
    expect(res2._tag).toBe('Err');
  });

  it('validateName accepts non-empty names and trims whitespace', () => {
    const res = Pipeline.validateName('  My Transformation Pipeline  ');
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value).toBe('My Transformation Pipeline');
    }
  });
});
