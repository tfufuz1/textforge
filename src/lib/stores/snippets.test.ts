import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => mockInvoke(...args)
}));

import { notificationsStore } from './notifications';
import {
    setSearchQuery,
    setFilter,
    loadAllTags,
    allTagsStore,
    tagCloud,
    handleCreateSnippet,
    handleTrashSnippet,
    snippetFilterStore
} from './snippets';

describe('snippets store', () => {
    beforeEach(() => {
        vi.useFakeTimers();
        mockInvoke.mockReset();
        notificationsStore.set([]);
        mockInvoke.mockImplementation((cmd: string) => {
            if (cmd === 'list_snippets') {
                return Promise.resolve({ items: [], totalCount: 0, hasMore: false });
            }
            if (cmd === 'suggest_tags') {
                return Promise.resolve([
                    { name: 'typescript', usageCount: 10, color: null, lastUsedAt: 0, createdAt: 0 },
                    { name: 'svelte', usageCount: 5, color: null, lastUsedAt: 0, createdAt: 0 }
                ]);
            }
            if (cmd === 'get_undo_state') {
                return Promise.resolve({ canUndo: false, canRedo: false, undoLabel: null, redoLabel: null });
            }
            return Promise.resolve({});
        });
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    test('setSearchQuery debounces loadSnippets calls', async () => {
        setSearchQuery('a');
        setSearchQuery('ab');
        setSearchQuery('abc');

        // Not called immediately
        expect(mockInvoke).not.toHaveBeenCalledWith('list_snippets', expect.anything());

        // Fast-forward 250ms
        await vi.advanceTimersByTimeAsync(250);

        const listCalls = mockInvoke.mock.calls.filter(c => c[0] === 'list_snippets');
        expect(listCalls.length).toBe(1);
        expect(listCalls[0][1]).toEqual({
            filter: expect.objectContaining({
                searchQuery: 'abc'
            })
        });
    });

    test('setFilter routes searchQuery through setSearchQuery and applies rest immediately', async () => {
        setFilter({ searchQuery: 'testquery', locationType: 'favorites' });

        expect(get(snippetFilterStore).locationType).toBe('favorites');

        // list_snippets for locationType happens, debounced for searchQuery
        await vi.advanceTimersByTimeAsync(250);

        expect(get(snippetFilterStore).searchQuery).toBe('testquery');
    });

    test('loadAllTags fetches tags via suggest_tags and updates allTagsStore and tagCloud', async () => {
        await loadAllTags();

        expect(mockInvoke).toHaveBeenCalledWith('suggest_tags', { prefix: '', limit: 500 });

        const tags = get(allTagsStore);
        expect(tags).toEqual([
            { tag: 'typescript', count: 10 },
            { tag: 'svelte', count: 5 }
        ]);

        const cloud = get(tagCloud);
        expect(cloud).toEqual(tags);
    });

    test('handleError propagates errors to notificationsStore', async () => {
        mockInvoke.mockImplementation((cmd: string) => {
            if (cmd === 'create_snippet') {
                return Promise.reject(new Error('Database lock failure'));
            }
            return Promise.resolve({});
        });

        const result = await handleCreateSnippet({
            title: 'Test',
            content: 'Content',
            contentType: 'text/plain',
            tags: [],
            folderId: null
        });

        expect(result).toBeNull();

        const notifications = get(notificationsStore);
        expect(notifications.length).toBe(1);
        expect(notifications[0].severity).toBe('error');
        expect(notifications[0].title).toBe('Fehler');
        expect(notifications[0].message._tag).toBe('Some');
        if (notifications[0].message._tag === 'Some') {
            expect(notifications[0].message.value).toContain('Snippet erstellen: Database lock failure');
        }
    });

    test('mutation functions reload tags via loadAllTags', async () => {
        mockInvoke.mockImplementation((cmd: string) => {
            if (cmd === 'trash_snippet') {
                return Promise.resolve({});
            }
            if (cmd === 'list_snippets') {
                return Promise.resolve({ items: [], totalCount: 0, hasMore: false });
            }
            if (cmd === 'suggest_tags') {
                return Promise.resolve([
                    { name: 'react', usageCount: 2, color: null, lastUsedAt: 0, createdAt: 0 }
                ]);
            }
            if (cmd === 'get_undo_state') {
                return Promise.resolve({ canUndo: false, canRedo: false, undoLabel: null, redoLabel: null });
            }
            return Promise.resolve({});
        });

        await handleTrashSnippet('snip-1');

        expect(mockInvoke).toHaveBeenCalledWith('suggest_tags', { prefix: '', limit: 500 });
        expect(get(allTagsStore)).toEqual([
            { tag: 'react', count: 2 }
        ]);
    });
});
