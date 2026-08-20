import { describe, it, expect } from 'vitest';
import { Option, Result, SnippetId, ScriptId } from './adts';

describe('Option ADT', () => {
  it('some, none, map, flatMap, getOrElse, fold', () => {
    const s = Option.some(10);
    const n = Option.none<number>();

    expect(Option.map(s, x => x * 2)).toEqual(Option.some(20));
    expect(Option.map(n, x => x * 2)).toEqual(Option.none());

    expect(Option.flatMap(s, x => Option.some(x + 5))).toEqual(Option.some(15));
    expect(Option.flatMap(n, x => Option.some(x + 5))).toEqual(Option.none());

    expect(Option.getOrElse(s, 0)).toBe(10);
    expect(Option.getOrElse(n, 0)).toBe(0);

    expect(Option.fold(s, () => 'none', val => `some:${val}`)).toBe('some:10');
    expect(Option.fold(n, () => 'none', val => `some:${val}`)).toBe('none');
  });

  it('fromNullable and zip', () => {
    expect(Option.fromNullable('hello')).toEqual(Option.some('hello'));
    expect(Option.fromNullable(null)).toEqual(Option.none());
    expect(Option.fromNullable(undefined)).toEqual(Option.none());

    expect(Option.zip(Option.some(1), Option.some('a'))).toEqual(Option.some([1, 'a']));
    expect(Option.zip(Option.some(1), Option.none())).toEqual(Option.none());
  });
});

describe('Result ADT', () => {
  it('ok, err, map, flatMap, fold', () => {
    const ok = Result.ok<string, number>(42);
    const err = Result.err<string, number>('failed');

    expect(Result.map(ok, x => x * 2)).toEqual(Result.ok(84));
    expect(Result.map(err, x => x * 2)).toEqual(Result.err('failed'));

    expect(Result.flatMap(ok, x => Result.ok(x + 1))).toEqual(Result.ok(43));
    expect(Result.flatMap(ok, () => Result.err('new err'))).toEqual(Result.err('new err'));

    expect(Result.fold(ok, e => `err:${e}`, v => `ok:${v}`)).toBe('ok:42');
    expect(Result.fold(err, e => `err:${e}`, v => `ok:${v}`)).toBe('err:failed');
  });

  it('tryCatch, sequence, validate', () => {
    const okTry = Result.tryCatch(() => JSON.parse('{"a":1}'), e => String(e));
    expect(okTry._tag).toBe('Ok');

    const errTry = Result.tryCatch(() => JSON.parse('invalid json'), e => 'bad_json');
    expect(errTry).toEqual(Result.err('bad_json'));

    const seqOk = Result.sequence([Result.ok(1), Result.ok(2), Result.ok(3)]);
    expect(seqOk).toEqual(Result.ok([1, 2, 3]));

    const seqErr = Result.sequence([Result.ok(1), Result.err('err1'), Result.ok(3)]);
    expect(seqErr).toEqual(Result.err('err1'));

    const valRes = Result.validate([Result.ok(1), Result.err('e1'), Result.ok(2), Result.err('e2')]);
    expect(valRes).toEqual(Result.err(['e1', 'e2']));
  });
});

describe('Branded Types', () => {
  it('constructs branded IDs correctly', () => {
    const snipId = SnippetId.of('uuid-1');
    const scriptId = ScriptId.of('script-1');
    expect(snipId).toBe('uuid-1');
    expect(scriptId).toBe('script-1');
  });
});
