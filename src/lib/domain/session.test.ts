import { describe, it, expect } from 'vitest';
import { createDefaultSession, patchWorkspaceSession, type WorkspaceSession } from './session';

describe('Workspace Session Domain Logic', () => {
  it('creates default session properly', () => {
    const session = createDefaultSession();
    expect(session.activeView).toBe('clipboard');
    expect(session.sidebarOpen).toBe(true);
    expect(session.sidebarWidth).toBe(256);
    expect(session.previewMode).toBe('split');
  });

  it('patches workspace session correctly', () => {
    const current = createDefaultSession();
    const patched = patchWorkspaceSession(current, {
      activeView: 'snippets',
      selectedSnippetId: 'snip_123',
    });

    expect(patched.activeView).toBe('snippets');
    expect(patched.selectedSnippetId).toBe('snip_123');
    expect(patched.sidebarOpen).toBe(true);
    expect(patched.savedAt).toBeDefined();
  });
});
