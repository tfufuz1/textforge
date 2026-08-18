import { describe, it, expect } from 'vitest';
import { Script, type ScriptParameter } from './script';
import { DomainError } from './errors';
import { Option } from './adts';

describe('Script Domain', () => {
    it('validateName should reject empty names', () => {
        const result = Script.validateName('   ');
        expect(result._tag).toBe('Err');
        if (result._tag === 'Err') {
            expect(result.error.code).toBe('EMPTY_SCRIPT_NAME');
        }
    });

    it('validateName should accept valid names', () => {
        const result = Script.validateName('  Valid Name  ');
        expect(result._tag).toBe('Ok');
        if (result._tag === 'Ok') {
            expect(result.value).toBe('Valid Name');
        }
    });

    it('validateRegex should reject invalid regex', () => {
        const result = Script.validateRegex('[', 'g');
        expect(result._tag).toBe('Err');
        if (result._tag === 'Err') {
            expect(result.error.code).toBe('INVALID_REGEX_PATTERN');
        }
    });

    it('validateRegex should accept valid regex', () => {
        const result = Script.validateRegex('\\d+', 'g');
        expect(result._tag).toBe('Ok');
        if (result._tag === 'Ok') {
            expect(result.value).toBeInstanceOf(RegExp);
        }
    });

    it('DomainError.describe translates script errors correctly', () => {
        const err: DomainError = { code: 'SCRIPT_TIMEOUT', limitMs: 5000 };
        expect(DomainError.describe(err)).toBe('Skript überschritt Zeitlimit (5000 ms).');

        const err2: DomainError = { code: 'SCRIPT_SYNTAX_ERROR', details: 'Unexpected token', line: 10 };
        expect(DomainError.describe(err2)).toContain('Syntaxfehler im Skript: Unexpected token');
    });

    it('ScriptParameter types are valid', () => {
        // Just verifying the type structure compiles and can be constructed
        const param1: ScriptParameter = {
            _type: 'number',
            key: 'age',
            label: 'Age',
            default: 18,
            min: Option.some(0),
            max: Option.none(),
            step: 1,
            unit: Option.some('years')
        };
        expect(param1._type).toBe('number');
    });
});
