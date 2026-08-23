import { describe, it, expect } from 'vitest';
import { findShortcutMatch, SHORTCUT_REGISTRY } from '../shortcuts/registry';

describe('Shortcuts Domain Logic', () => {
  it('contains expected shortcut commands in registry', () => {
    expect(SHORTCUT_REGISTRY.length).toBeGreaterThan(8);

    const cp = SHORTCUT_REGISTRY.find(s => s.id === 'command_palette');
    expect(cp?.key).toBe('P');
    expect(cp?.ctrl).toBe(true);
    expect(cp?.shift).toBe(true);

    const qc = SHORTCUT_REGISTRY.find(s => s.id === 'quick_capture');
    expect(qc?.key).toBe('v');
    expect(qc?.ctrl).toBe(true);
    expect(qc?.alt).toBe(true);

    const qs = SHORTCUT_REGISTRY.find(s => s.id === 'quick_search');
    expect(qs?.key).toBe('k');
    expect(qs?.ctrl).toBe(true);
  });

  it('matches keyboard event for global shortcut', () => {
    const mockEvent = {
      key: 'p',
      ctrlKey: true,
      shiftKey: true,
      altKey: false,
      metaKey: false,
    } as unknown as KeyboardEvent;

    const matched = findShortcutMatch(mockEvent, 'global');
    expect(matched).toBeDefined();
    expect(matched?.id).toBe('command_palette');
  });

  it('matches keyboard event for quick capture shortcut', () => {
    const mockEvent = {
      key: 'v',
      ctrlKey: true,
      shiftKey: false,
      altKey: true,
      metaKey: false,
    } as unknown as KeyboardEvent;

    const matched = findShortcutMatch(mockEvent, 'global');
    expect(matched).toBeDefined();
    expect(matched?.id).toBe('quick_capture');
  });

  it('matches keyboard event for context-specific shortcut', () => {
    const mockEvent = {
      key: 's',
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      metaKey: false,
    } as unknown as KeyboardEvent;

    const matched = findShortcutMatch(mockEvent, 'snippet_editor');
    expect(matched).toBeDefined();
    expect(matched?.id).toBe('save_snippet');
  });
});
