import { describe, test, expect } from 'vitest';
import { Snippet, detectContentType } from './snippet';
import { Option } from './adts';

describe('Snippet Domain Core', () => {
    test('create generates a valid snippet with default values and auto-detected content type', () => {
        const res = Snippet.create({
            title: 'My Clipboard Snippet',
            content: '{"foo": "bar"}',
            location: { _type: 'inbox' },
            sourceApp: 'firefox'
        });

        expect(res._tag).toBe('Ok');
        if (res._tag === 'Ok') {
            const snip = res.value;
            expect(snip.title).toBe('My Clipboard Snippet');
            expect(snip.contentType).toBe('json');
            expect(snip.sourceApp._tag).toBe('Some');
            if (snip.sourceApp._tag === 'Some') {
                expect(snip.sourceApp.value).toBe('firefox');
            }
            expect(snip.usageCount).toBe(0);
            expect(snip.isPinned).toBe(false);
            expect(snip.favorite).toBe(false);
        }
    });

    test('create fails when title is empty or exceeds limit', () => {
        const emptyRes = Snippet.create({
            title: '   ',
            content: 'test',
            location: { _type: 'inbox' }
        });
        expect(emptyRes._tag).toBe('Err');

        const longTitle = 'a'.repeat(129);
        const longRes = Snippet.create({
            title: longTitle,
            content: 'test',
            location: { _type: 'inbox' }
        });
        expect(longRes._tag).toBe('Err');
    });

    test('detectContentType correctly identifies urls, templates and plain text', () => {
        expect(detectContentType('https://tauri.app')).toBe('url');
        expect(detectContentType('Hello {{name}}, welcome!')).toBe('template');
        expect(detectContentType('Plain text content')).toBe('plain_text');
    });

    test('update creates a new immutable snippet with updated properties', () => {
        const createRes = Snippet.create({
            title: 'Original Title',
            content: 'Original content',
            location: { _type: 'inbox' }
        });
        expect(createRes._tag).toBe('Ok');
        if (createRes._tag === 'Ok') {
            const original = createRes.value;
            const updateRes = Snippet.update(original, {
                title: 'New Title',
                isPinned: true
            });
            expect(updateRes._tag).toBe('Ok');
            if (updateRes._tag === 'Ok') {
                const updated = updateRes.value;
                expect(updated.title).toBe('New Title');
                expect(updated.isPinned).toBe(true);
                expect(original.title).toBe('Original Title');
                expect(original.isPinned).toBe(false);
            }
        }
    });
});
