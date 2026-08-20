import type { ScriptId, UnixMs, TagName } from './adts';
import { Result, Option } from './adts';
import type { DomainError } from './errors';

export type ScriptType = 'js' | 'regex' | 'builtin';
export type ScriptCategory = 'text' | 'code' | 'security' | 'format' | 'analysis' | 'custom';

export type RegexFlags = string;

export type ScriptParameter =
  | { readonly _type: 'text';    readonly key: string; readonly label: string; readonly default: string;
      readonly placeholder: string; readonly required: boolean; readonly maxLength: Option<number> }
  | { readonly _type: 'number';  readonly key: string; readonly label: string; readonly default: number;
      readonly min: Option<number>; readonly max: Option<number>; readonly step: number; readonly unit: Option<string> }
  | { readonly _type: 'select';  readonly key: string; readonly label: string; readonly default: string;
      readonly options: ReadonlyArray<{ readonly value: string; readonly label: string }> }
  | { readonly _type: 'boolean'; readonly key: string; readonly label: string; readonly default: boolean;
      readonly description: Option<string> }
  | { readonly _type: 'regex';   readonly key: string; readonly label: string; readonly default: string;
      readonly validateOnChange: boolean }
  | { readonly _type: 'textarea'; readonly key: string; readonly label: string; readonly default: string;
      readonly rows: number; readonly placeholder: string };

export type ParameterValues = Readonly<Record<string, string | number | boolean>>;

export type TestResult =
  | { readonly _tag: 'Pass';  readonly actual: string;  readonly durationMs: number }
  | { readonly _tag: 'Fail';  readonly actual: string;  readonly durationMs: number }
  | { readonly _tag: 'Error'; readonly message: string; readonly durationMs: number };

export interface ScriptTest {
  readonly id:         string;
  readonly label:      string;
  readonly input:      string;
  readonly parameters: ParameterValues;
  readonly expected:   string;
  readonly lastResult: Option<TestResult>;
}

export interface ScriptVersion {
  readonly id: string;
  readonly version: number;
  readonly jsCode?: string;
  readonly regexPattern?: string;
  readonly regexReplacement?: string;
  readonly regexFlags?: string;
  readonly changeNote: string;
  readonly savedAt: UnixMs;
}

export interface Script {
  readonly id: ScriptId;
  readonly name: string;
  readonly description: string;
  readonly type: ScriptType;
  readonly category: ScriptCategory;

  readonly jsCode: Option<string>;
  readonly regexPattern: Option<string>;
  readonly regexReplacement: Option<string>;
  readonly regexFlags: RegexFlags;

  readonly parameters: ReadonlyArray<ScriptParameter>;
  readonly tests: ReadonlyArray<ScriptTest>;

  readonly isFavorite: boolean;
  readonly isSafetyCritical: boolean;
  readonly usageCount: number;
  readonly lastUsedAt: Option<UnixMs>;
  readonly createdAt: UnixMs;
  readonly updatedAt: UnixMs;

  readonly tags: ReadonlyArray<TagName>;
  readonly currentVersion: number;
  readonly color: Option<string>;
}

export const Script = {
  validateName: (name: string): Result<DomainError, string> => {
    const trimmed = name.trim();
    if (!trimmed) return Result.err({ code: 'EMPTY_SCRIPT_NAME' });
    return Result.ok(trimmed);
  },

  validateRegex: (pattern: string, flags: string): Result<DomainError, RegExp> => {
    try {
      const re = new RegExp(pattern, flags);
      return Result.ok(re);
    } catch (e: any) {
      return Result.err({
        code: 'INVALID_REGEX_PATTERN',
        pattern,
        details: e.message || String(e),
      });
    }
  },

  substituteParams: (text: string, params: ParameterValues = {}): string => {
    let result = text;
    for (const [k, v] of Object.entries(params)) {
      const placeholder = new RegExp(`\\{\\{\\s*${k}\\s*\\}\\}`, 'g');
      result = result.replace(placeholder, String(v));
    }
    return result;
  },

  executeRegex: (
    input: string,
    pattern: string,
    replacement: string,
    flags: string = 'g',
    params: ParameterValues = {}
  ): Result<DomainError, string> => {
    const subPattern = Script.substituteParams(pattern, params);
    const subReplacement = Script.substituteParams(replacement, params);
    const reResult = Script.validateRegex(subPattern, flags);
    if (reResult._tag === 'Err') {
      return reResult;
    }
    try {
      const output = input.replace(reResult.value, subReplacement);
      return Result.ok(output);
    } catch (e: any) {
      return Result.err({
        code: 'SCRIPT_RUNTIME_ERROR',
        details: e.message || String(e),
      });
    }
  },

  executeJS: (
    input: string,
    jsCode: string,
    params: ParameterValues = {}
  ): Result<DomainError, string> => {
    try {
      const utils = {
        lines: (t: string) => String(t || '').split(/\r?\n/),
        unlines: (arr: any) => Array.isArray(arr) ? arr.join('\n') : String(arr),
        words: (t: string) => String(t || '').trim().split(/\s+/).filter(Boolean),
        sortLines: (t: string) => utils.lines(t).sort().join('\n'),
        uniqueLines: (t: string) => Array.from(new Set(utils.lines(t))).join('\n'),
        reverseLines: (t: string) => utils.lines(t).reverse().join('\n'),
        trim: (t: string) => String(t || '').trim(),
        uppercase: (t: string) => String(t || '').toUpperCase(),
        lowercase: (t: string) => String(t || '').toLowerCase(),
        prettyJSON: (t: string) => JSON.stringify(JSON.parse(t), null, 2),
        minifyJSON: (t: string) => JSON.stringify(JSON.parse(t)),
        base64Encode: (t: string) => typeof btoa === 'function' ? btoa(t) : ((globalThis as any).Buffer ? (globalThis as any).Buffer.from(t).toString('base64') : t),
        base64Decode: (t: string) => typeof atob === 'function' ? atob(t) : ((globalThis as any).Buffer ? (globalThis as any).Buffer.from(t, 'base64').toString('utf-8') : t),
        redact: (t: string, mask = '***') => String(t || '').replace(/[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g, mask),
      };

      const fn = new Function('input', 'utils', 'params', `
        ${jsCode.includes('return') ? jsCode : `return (${jsCode});`}
      `);

      const rawResult = fn(input, utils, params);
      if (rawResult === undefined || rawResult === null) {
        return Result.ok('');
      }
      if (typeof rawResult === 'string') {
        return Result.ok(rawResult);
      }
      if (typeof rawResult === 'object') {
        return Result.ok(JSON.stringify(rawResult, null, 2));
      }
      return Result.ok(String(rawResult));
    } catch (e: any) {
      return Result.err({
        code: 'SCRIPT_RUNTIME_ERROR',
        details: e.message || String(e),
      });
    }
  },
} as const;
