export type Option<A> =
  | { readonly _tag: 'Some'; readonly value: A }
  | { readonly _tag: 'None' };

export const Option = {
  some:      <A>(value: A): Option<A>             => ({ _tag: 'Some', value }),
  none:      <A>(): Option<A>                     => ({ _tag: 'None' }),
  map:       <A, B>(opt: Option<A>, f: (a: A) => B): Option<B> =>
               opt._tag === 'Some' ? Option.some(f(opt.value)) : Option.none(),
  flatMap:   <A, B>(opt: Option<A>, f: (a: A) => Option<B>): Option<B> =>
               opt._tag === 'Some' ? f(opt.value) : Option.none(),
  getOrElse: <A>(opt: Option<A>, fallback: A): A  =>
               opt._tag === 'Some' ? opt.value : fallback,
  fold:      <A, B>(opt: Option<A>, onNone: () => B, onSome: (a: A) => B): B =>
               opt._tag === 'Some' ? onSome(opt.value) : onNone(),
  fromNullable: <A>(a: A | null | undefined): Option<A> =>
               a == null ? Option.none() : Option.some(a),
  zip:       <A, B>(a: Option<A>, b: Option<B>): Option<[A, B]> =>
               a._tag === 'Some' && b._tag === 'Some'
                 ? Option.some([a.value, b.value])
                 : Option.none(),
} as const;

export type Result<E, A> =
  | { readonly _tag: 'Ok';  readonly value: A }
  | { readonly _tag: 'Err'; readonly error: E };

export const Result = {
  ok:       <E, A>(value: A): Result<E, A>                  => ({ _tag: 'Ok',  value }),
  err:      <E, A>(error: E): Result<E, A>                  => ({ _tag: 'Err', error }),
  map:      <E, A, B>(r: Result<E, A>, f: (a: A) => B): Result<E, B> =>
               r._tag === 'Ok' ? Result.ok(f(r.value)) : r,
  flatMap:  <E, A, B>(r: Result<E, A>, f: (a: A) => Result<E, B>): Result<E, B> =>
               r._tag === 'Ok' ? f(r.value) : r,
  fold:     <E, A, B>(r: Result<E, A>, onErr: (e: E) => B, onOk: (a: A) => B): B =>
               r._tag === 'Ok' ? onOk(r.value) : onErr(r.error),
  tryCatch: <E, A>(f: () => A, toError: (e: unknown) => E): Result<E, A> => {
    try { return Result.ok(f()); }
    catch (e) { return Result.err(toError(e)); }
  },
  sequence: <E, A>(results: Result<E, A>[]): Result<E, A[]> => {
    const acc: A[] = [];
    for (const r of results) {
      if (r._tag === 'Err') return r;
      acc.push(r.value);
    }
    return Result.ok(acc);
  },
  validate: <E, A>(results: Result<E, A>[]): Result<E[], A[]> => {
    const errors: E[] = [];
    const values: A[] = [];
    for (const r of results) {
      if (r._tag === 'Err') errors.push(r.error);
      else values.push(r.value);
    }
    return errors.length > 0 ? Result.err(errors) : Result.ok(values);
  }
} as const;

export type NonEmptyArray<T> = [T, ...T[]];

declare const __brand: unique symbol;
export type Brand<B>        = { readonly [__brand]: B };
export type Branded<T, B>   = T & Brand<B>;

export type SnippetId    = Branded<string, 'SnippetId'>;
export type FolderId     = Branded<string, 'FolderId'>;
export type ClipEntryId  = Branded<string, 'ClipEntryId'>;
export type TagName      = Branded<string, 'TagName'>;
export type ScriptId     = Branded<string, 'ScriptId'>;
export type PipelineId   = Branded<string, 'PipelineId'>;
export type TemplateId   = Branded<string, 'TemplateId'>;
export type ScriptVerId  = Branded<string, 'ScriptVerId'>;
export type BundleId     = Branded<string, 'BundleId'>;

export type UnixMs       = Branded<number, 'UnixMs'>;
export type ByteSize     = Branded<number, 'ByteSize'>;
export type TokenCount   = Branded<number, 'TokenCount'>;
export type LineNumber   = Branded<number, 'LineNumber'>;

export const SnippetId   = { of: (s: string) => s as SnippetId };
export const ClipEntryId = { of: (s: string) => s as ClipEntryId };
export const FolderId    = { of: (s: string) => s as FolderId };
export const ScriptId    = { of: (s: string) => s as ScriptId };
export const PipelineId  = { of: (s: string) => s as PipelineId };
export const TemplateId  = { of: (s: string) => s as TemplateId };
export const ScriptVerId = { of: (s: string) => s as ScriptVerId };
