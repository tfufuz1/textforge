import { describe, test, expect } from 'vitest';
import { ClipboardEntry } from './clipboard-entry';
import { Option } from './adts';

describe('ClipboardEntry Domain Core', () => {
    test('create rejects empty or whitespace-only content', () => {
        expect(ClipboardEntry.create('', Option.none())._tag).toBe('None');
        expect(ClipboardEntry.create('   ', Option.none())._tag).toBe('None');
        expect(ClipboardEntry.create('\n\t  \n', Option.none())._tag).toBe('None');
    });

    test('create assigns UnixMs timestamp and calculates sizes, lines, and words correctly', () => {
        const text = 'Hello world\nThis is a multiline snippet.\nWith three lines.';
        const result = ClipboardEntry.create(text, Option.some('kate'));

        expect(result._tag).toBe('Some');
        if (result._tag === 'Some') {
            const entry = result.value;
            expect(entry.capturedAt).toBeGreaterThan(0);
            expect(entry.sourceApp._tag).toBe('Some');
            if (entry.sourceApp._tag === 'Some') {
                expect(entry.sourceApp.value).toBe('kate');
            }
            expect(entry.sizeBytes).toBe(new TextEncoder().encode(text).length);
            expect(entry.lineCount).toBe(3);
            expect(entry.wordCount).toBe(10);
            expect(entry.isPinned).toBe(false);
            expect(entry.promotedToSnippetId._tag).toBe('None');
        }
    });

    test('contentHash calculates valid 64-character hex SHA-256 string', () => {
        const text = 'Hello world';
        const result = ClipboardEntry.create(text, Option.none());
        expect(result._tag).toBe('Some');
        if (result._tag === 'Some') {
            // Known SHA-256 for 'Hello world'
            expect(result.value.contentHash).toBe('64ec88ca00b268e5ba1a35678a1b5316d212f4f366b2477232534a8aeca37f3c');
        }
    });

    test('contentType is automatically detected upon entry creation', () => {
        const jsonResult = ClipboardEntry.create('{"key": "value"}', Option.none());
        expect(jsonResult._tag).toBe('Some');
        if (jsonResult._tag === 'Some') {
            expect(jsonResult.value.contentType).toBe('json');
        }

        const urlResult = ClipboardEntry.create('https://example.com/api', Option.none());
        expect(urlResult._tag).toBe('Some');
        if (urlResult._tag === 'Some') {
            expect(urlResult.value.contentType).toBe('url');
        }
    });

    test('toSnippetDraft maps entry to a draft with trimmed title and inbox location', () => {
        const longText = '   First line of long text that should be used as snippet title exceeding sixty characters...   \nSecond line.';
        const result = ClipboardEntry.create(longText, Option.none());
        expect(result._tag).toBe('Some');
        if (result._tag === 'Some') {
            const draft = ClipboardEntry.toSnippetDraft(result.value);
            expect(draft.title.length).toBeLessThanOrEqual(60);
            expect(draft.title).toBe('First line of long text that should be used as snippet ti');
            expect(draft.content).toBe(longText);
            expect(draft.location).toEqual({ _type: 'inbox' });
        }
    });

    test('toSnippetDraft handles single short line correctly', () => {
        const shortText = 'Short note';
        const result = ClipboardEntry.create(shortText, Option.none());
        expect(result._tag).toBe('Some');
        if (result._tag === 'Some') {
            const draft = ClipboardEntry.toSnippetDraft(result.value);
            expect(draft.title).toBe('Short note');
            expect(draft.content).toBe('Short note');
        }
    });
});
