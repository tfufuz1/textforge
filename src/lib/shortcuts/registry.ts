export type ShortcutContext = 'global' | 'snippet_list' | 'snippet_editor' | 'script_editor';

export interface ShortcutCommand {
  readonly id: string;
  readonly key: string;
  readonly ctrl?: boolean;
  readonly shift?: boolean;
  readonly alt?: boolean;
  readonly context: ShortcutContext;
  readonly description: string;
  readonly actionId: string;
}

export const SHORTCUT_REGISTRY: readonly ShortcutCommand[] = [
  { id: 'command_palette', key: 'P', ctrl: true, shift: true, context: 'global', description: 'Command Palette öffnen', actionId: 'toggle_command_palette' },
  { id: 'quick_capture', key: 'v', ctrl: true, alt: true, context: 'global', description: 'Neues Snippet aus Zwischenablage (Quick Capture)', actionId: 'open_quick_capture' },
  { id: 'quick_search', key: 'k', ctrl: true, context: 'global', description: 'Schnellsuche in Snippet-Bibliothek', actionId: 'focus_quick_search' },
  { id: 'undo', key: 'z', ctrl: true, context: 'global', description: 'Aktion rückgängig machen', actionId: 'perform_undo' },
  { id: 'redo', key: 'y', ctrl: true, context: 'global', description: 'Aktion wiederholen', actionId: 'perform_redo' },
  { id: 'new_snippet', key: 'n', ctrl: true, context: 'global', description: 'Neues Snippet erstellen', actionId: 'create_snippet' },
  { id: 'open_settings', key: ',', ctrl: true, context: 'global', description: 'Einstellungen öffnen', actionId: 'navigate_settings' },

  { id: 'search_snippets', key: 'f', ctrl: true, context: 'snippet_list', description: 'Suche fokussieren', actionId: 'focus_search' },
  { id: 'duplicate_snippet', key: 'd', ctrl: true, context: 'snippet_list', description: 'Aktuelles Snippet duplizieren', actionId: 'duplicate_snippet' },

  { id: 'save_snippet', key: 's', ctrl: true, context: 'snippet_editor', description: 'Snippet speichern', actionId: 'save_snippet' },
  { id: 'toggle_preview', key: 'M', ctrl: true, shift: true, context: 'snippet_editor', description: 'Vorschau-Modus umschalten', actionId: 'toggle_editor_preview' },
  { id: 'copy_transformed', key: 'C', ctrl: true, shift: true, context: 'snippet_editor', description: 'Transformations-Ergebnis kopieren', actionId: 'copy_transformed_result' },

  { id: 'save_script', key: 's', ctrl: true, context: 'script_editor', description: 'Skript speichern', actionId: 'save_script' },
  { id: 'run_script', key: 'Enter', ctrl: true, context: 'script_editor', description: 'Skript ausführen', actionId: 'execute_script' }
];

export function findShortcutMatch(
  event: KeyboardEvent,
  activeContext: ShortcutContext = 'global'
): ShortcutCommand | undefined {
  const key = event.key;
  const ctrl = event.ctrlKey || event.metaKey;
  const shift = event.shiftKey;
  const alt = event.altKey;

  return SHORTCUT_REGISTRY.find((sc) => {
    if (sc.context !== 'global' && sc.context !== activeContext) return false;

    const matchesCtrl = Boolean(sc.ctrl) === ctrl;
    const matchesShift = Boolean(sc.shift) === shift;
    const matchesAlt = Boolean(sc.alt) === alt;
    const matchesKey = sc.key.toLowerCase() === key.toLowerCase();

    return matchesKey && matchesCtrl && matchesShift && matchesAlt;
  });
}
