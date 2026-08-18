import { describe, test, expect } from 'vitest';
import { ClipboardEntry } from './clipboard-entry';
import { Option } from './adts';

describe('ClipboardEntry', () => {
    test('create rejects empty content', () => {
        const result = ClipboardEntry.create('   ', Option.none());
        expect(result._tag).toBe('None');
    });

    test('create assigns UnixMs timestamp and calculates sizes', () => {
        const result = ClipboardEntry.create('hello world', Option.some('test_app'));
        expect(result._tag).toBe('Some');
        if (result._tag === 'Some') {
            const entry = result.value;
            expect(entry.capturedAt).toBeGreaterThan(0);
            expect(entry.sourceApp._tag).toBe('Some');
            if (entry.sourceApp._tag === 'Some') {
                expect(entry.sourceApp.value).toBe('test_app');
            }
            expect(entry.sizeBytes).toBe(11);
            expect(entry.wordCount).toBe(2);
            expect(entry.lineCount).toBe(1);
        }
    });
});
