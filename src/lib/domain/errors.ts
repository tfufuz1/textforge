import type { SnippetId, ScriptId, PipelineId } from './adts';

export type DomainError =
  // Snippet
  | { readonly code: 'EMPTY_TITLE' }
  | { readonly code: 'TITLE_TOO_LONG';    readonly max: number }
  | { readonly code: 'CONTENT_TOO_LARGE'; readonly maxBytes: number }
  | { readonly code: 'SNIPPET_NOT_FOUND'; readonly id: SnippetId }
  | { readonly code: 'DUPLICATE_TITLE';   readonly existing: SnippetId }
  | { readonly code: 'DUPLICATE_TAGS' }
  | { readonly code: 'TOO_MANY_TAGS';     readonly max: number }
  | { readonly code: 'INVALID_COLOR';     readonly value: string }
  | { readonly code: 'INVALID_TAG';       readonly raw: string }
  | { readonly code: 'STORAGE_ERROR';     readonly details: string }
  | { readonly code: 'UNDO_STACK_EMPTY' }
  | { readonly code: 'REDO_STACK_EMPTY' }

  // Script
  | { readonly code: 'EMPTY_SCRIPT_NAME' }
  | { readonly code: 'INVALID_REGEX_PATTERN';  readonly pattern: string; readonly details: string }
  | { readonly code: 'SCRIPT_SYNTAX_ERROR';     readonly details: string; readonly line?: number }
  | { readonly code: 'SCRIPT_RUNTIME_ERROR';    readonly details: string }
  | { readonly code: 'SCRIPT_TIMEOUT';          readonly limitMs: number }
  | { readonly code: 'SCRIPT_OUTPUT_TOO_LARGE'; readonly actualBytes: number; readonly limitBytes: number }
  | { readonly code: 'SCRIPT_INPUT_TOO_LARGE';  readonly actualBytes: number; readonly limitBytes: number }
  | { readonly code: 'SCRIPT_NOT_FOUND';        readonly id: ScriptId }
  | { readonly code: 'SCRIPT_INVALID_OUTPUT';   readonly details: string }

  // Pipeline
  | { readonly code: 'EMPTY_PIPELINE_NAME' }
  | { readonly code: 'PIPELINE_NOT_FOUND';    readonly id: PipelineId }
  | { readonly code: 'EMPTY_PIPELINE_STEPS' }
  | { readonly code: 'PIPELINE_STEP_FAILED';  readonly stepId: string; readonly details: string }

  // Template
  | { readonly code: 'MISSING_TEMPLATE_VAR'; readonly variable: string }
  | { readonly code: 'TEMPLATE_PARSE_ERROR'; readonly details: string }

  // Clipboard
  | { readonly code: 'CLIPBOARD_EMPTY' }
  | { readonly code: 'CLIPBOARD_ENTRY_NOT_FOUND'; readonly id: string }

  // Import / Export
  | { readonly code: 'IMPORT_CORRUPTED_JSON'; readonly details: string }
  | { readonly code: 'IMPORT_VERSION_MISMATCH'; readonly version: string };

export const DomainError = {
  describe: (e: DomainError): string => {
    switch (e.code) {
      case 'EMPTY_TITLE':                return 'Titel darf nicht leer sein.';
      case 'TITLE_TOO_LONG':             return `Titel zu lang (max. ${e.max} Zeichen).`;
      case 'CONTENT_TOO_LARGE':          return `Inhalt zu groß (max. 10 MB).`;
      case 'UNDO_STACK_EMPTY':           return 'Nichts zum Rückgängigmachen.';
      case 'REDO_STACK_EMPTY':           return 'Nichts zum Wiederholen.';
      case 'EMPTY_SCRIPT_NAME':          return 'Skript-Name darf nicht leer sein.';
      case 'INVALID_REGEX_PATTERN':      return `Ungültiges Regex-Muster "${e.pattern}": ${e.details}`;
      case 'SCRIPT_SYNTAX_ERROR':        return `Syntaxfehler im Skript: ${e.details}`;
      case 'SCRIPT_RUNTIME_ERROR':       return `Laufzeitfehler im Skript: ${e.details}`;
      case 'SCRIPT_TIMEOUT':             return `Skript überschritt Zeitlimit (${e.limitMs} ms).`;
      case 'SCRIPT_OUTPUT_TOO_LARGE':    return `Ausgabe zu groß (${e.actualBytes} > ${e.limitBytes} Bytes).`;
      case 'SCRIPT_INPUT_TOO_LARGE':     return `Eingabe zu groß (${e.actualBytes} > ${e.limitBytes} Bytes).`;
      case 'SCRIPT_INVALID_OUTPUT':      return `Ungültige Skript-Ausgabe: ${e.details}`;
      case 'EMPTY_PIPELINE_NAME':        return 'Pipeline-Name darf nicht leer sein.';
      case 'EMPTY_PIPELINE_STEPS':       return 'Pipeline enthält keine Schritte.';
      case 'PIPELINE_STEP_FAILED':       return `Pipeline-Schritt "${e.stepId}" fehlgeschlagen: ${e.details}`;
      case 'MISSING_TEMPLATE_VAR':       return `Fehlende Variable: {{${e.variable}}}`;
      case 'TEMPLATE_PARSE_ERROR':       return `Template Parsing-Fehler: ${e.details}`;
      case 'CLIPBOARD_EMPTY':            return 'Zwischenablage ist leer.';
      case 'IMPORT_CORRUPTED_JSON':      return `Ungültige Import-Datei: ${e.details}`;
      case 'IMPORT_VERSION_MISMATCH':    return `Nicht unterstützte Import-Version: ${e.version}`;
      default:                           return `Fehler: ${(e as any).code}`;
    }
  },
} as const;