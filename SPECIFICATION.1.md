# TextForge — Interface-Spezifikation v2.1
## Vollständige kanonische Referenz für LLM-Entwickler
### Persönliches Text-Transformations-Tool (Tauri 2.x / KDE Plasma 6 / Wayland / SvelteKit)

---

> **Geltungsbereich & Verhältnis zu v2.0:**
> Diese Spezifikation ist eine Obermenge von v2.0. Alle v2.0-Typen bleiben gültig und binär-kompatibel.
> Neue Abschnitte sind mit `[NEU — v2.1]` markiert. Geänderte Abschnitte mit `[GEÄNDERT — v2.1]`.
> **Autoritative Quelle**: Diese Datei ist die einzige Wahrheit. Bei Konflikten mit v2.0 gilt v2.1.
>
> **Plattform-Update:** Zielplattform ist KDE Plasma 6 auf Wayland (nicht mehr MX-Linux/X11).
> Alle X11/XFixes-Pfade entfallen. Primär: `wl-clipboard` + KWin-D-Bus-API.
>
> **Entwicklungsreihenfolge:** Clipboard-Integration (§ 3) → Snippet-Bearbeitung (§ 4+) → Pipelines.
>
> Architekturprinzipien folgen dem **JS-Principles-System-Prompt v1.0** (Beilage).

---

## § 0 — LEITPRINZIPIEN [ERWEITERT — v2.0]

```
// [FP-Scala]  Unmögliche Zustände sind undarstellbar — Typen als erste Verteidigungslinie
// [PhilSD]    Tiefe Module: kleine Interfaces, große Implementierung
// [DDD]       Ubiquitäre Sprache: Code-Vokabular = Domänenvokabular
// [PragProg]  DRY auf Wissen — jede Regel hat genau eine autoritative Quelle
// [CleanCode] Namen enthüllen Absicht, nicht Implementierung
// [DDIA]      Datenintegrität ist nicht optional — WAL, Foreign Keys, Constraints immer an
// [Refactor]  Isoliere Änderungen hinter tiefen Modulen — nie rohe SQLite-Rows im Frontend
```

### Entwicklungsreihenfolge (Modul-Priorität)

```
PHASE 1 — Clipboard-Kern (§ 8, § 2.5, § 5.2, § 18.2, § 17 clipboard_history)
  ✓ ClipboardMonitor (wl-paste subprocess, Wayland-native) [Status: Partially Implemented - Active monitor, source-app detection unreliable]
  ✓ ClipboardEntry Domain-Typ + SHA-256-Dedup [Status: Implemented]
  ✓ SQLite: clipboard_history + FTS5 [Status: Partially Implemented - Search queries partially bypass FTS5]
  ✓ IPC: list/get/pin/delete/clear_clipboard_history [Status: Implemented]
  ✓ Frontend: ClipboardHistory.svelte, ClipboardEntry.svelte [Status: Implemented]
  ✓ Flow A vollständig (Clipboard-Eintrag → Anzeige) [Status: Implemented]
  → Deliverable: App zeigt live Clipboard-Verlauf, Einträge sind durchsuchbar [Status: Implemented]

PHASE 2 — Snippet-Erstellung aus Clipboard (§ 2.1, § 18.1 promote_*, § 9 Undo)
  ✓ Snippet.create() + promote_clipboard_to_snippet IPC [Status: Implemented]
  ✓ UndoStack für snippet_create [Status: Partially Implemented - Basic single updates supported, folders/scripts/bulk actions missing]
  ✓ Frontend: ClipboardEntryActions.svelte (Pin, Promote, Delete) [Status: Implemented]
  → Deliverable: Clipboard-Eintrag → Snippet in einem Klick [Status: Implemented]

PHASE 3 — Snippet-Bearbeitung (§ 2.1, § 3 Script, § 4 Pipeline, § 6 Template)
  ✓ Snippet CRUD vollständig [Status: Implemented]
  ✓ Script + Pipeline-Ausführung via QuickJS-Sandbox [Status: Partially Implemented - Complex structures, loops, conditionals missing]
  ✓ Template-Engine [Status: Partially Implemented - Basic placeholders working, loops/conditionals missing in Rust]
  → Deliverable: Snippets bearbeiten, transformieren, als Template verwenden [Status: Partially Implemented]

PHASE 4 — Erweiterte Features (§ 10 Import/Export, § 13 Bulk, § 14 Diff, ...) [Status: Partially Implemented - tfbundle manifest and dependency export incomplete]
```

### Architektur-Schichten (Pure Core / Effect Shell)

```
┌──────────────────────────────────────────────────────────────────┐
│  DOMAIN CORE (rein, zero dependencies, 100 % testbar)            │
│  • Alle Datentypen, Validierungen, Transformationslogik          │
│  • Option<A>, Result<E,A>, ADTs, TextStats, DiffResult           │
│  • Ratio: ~80 % des Gesamtcodes                                  │
├──────────────────────────────────────────────────────────────────┤
│  APPLICATION (Orchestrierung, Use Cases)                         │
│  • Pipeline-Ausführung, Bulk-Operationen, Undo-Stack             │
│  • Fehlertolerante Komposition aus Domain-Funktionen             │
│  • Ratio: ~12 %                                                  │
├──────────────────────────────────────────────────────────────────┤
│  EFFECT SHELL (SQLite, Filesystem, QuickJS, Tauri IPC, Clipboard)│
│  • Genau eine Möglichkeit pro Seiteneffekt                       │
│  • Ratio: ~8 %                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### Architektur-Invarianten (nie verletzen)

```
INVARIANT-A: Kein Domain-Core-Modul importiert aus effect/ oder ipc/
INVARIANT-B: Kein DTO verlässt das ipc/-Modul ohne Konvertierung in Domain-Typ
INVARIANT-C: Stores werden ausschließlich durch IPC-Aufrufe mutiert — kein direktes $store = ...
INVARIANT-D: Jede Mutation erzeugt ein neues Objekt (Spread / structuredClone)
INVARIANT-E: Fehler sind Result<DomainError, T> — niemals throw() in Business-Logik
INVARIANT-F: SQLite-Schema ist append-only (neue Migrations-Dateien, keine Änderungen an bestehenden)
INVARIANT-G: QuickJS-Sandbox hat kein Netzwerk, kein Filesystem, kein Clipboard
INVARIANT-H: Alle Timestamps sind UnixMs (UTC, Millisekunden) — keine lokalen Zeitzonenstrings
```

---

## § 1 — KERN-ADTs [ERWEITERT — v2.0]

### 1.1 Option\<A\> — Eliminierung von null

```typescript
// [FP-Scala] Referential transparency: Option statt null/undefined
type Option<A> =
  | { readonly _tag: 'Some'; readonly value: A }
  | { readonly _tag: 'None' };

const Option = {
  some:      <A>(value: A): Option<A>             => ({ _tag: 'Some', value }),
  none:      <A>(): Option<A>                     => ({ _tag: 'None' }),
  map:       <A, B>(opt: Option<A>, f: (a: A) => B): Option<B> =>
               opt._tag === 'Some' ? Option.some(f(opt.value)) : Option.none(),
  flatMap:   <A, B>(opt: Option<A>, f: (a: A) => Option<B>): Option<B> =>
               opt._tag === 'Some' ? f(opt.value) : Option.none(),
  getOrElse: <A>(opt: Option<A>, fallback: A): A  =>
               opt._tag === 'Some' ? opt.value : fallback,
  fold:      <A, B>(opt: Option<A>, onNone: () => B, onSome: (a: A) => B): B =>
               opt._tag === 'Some' ? onSome(opt.value) : onNone(),
  fromNullable: <A>(a: A | null | undefined): Option<A> =>
               a == null ? Option.none() : Option.some(a),
  // [v2.0] Neue Kombinator
  zip:       <A, B>(a: Option<A>, b: Option<B>): Option<[A, B]> =>
               a._tag === 'Some' && b._tag === 'Some'
                 ? Option.some([a.value, b.value])
                 : Option.none(),
} as const;
```

### 1.2 Result\<E, A\> — Fehler als Werte

```typescript
// [FP-Scala] Exceptions verletzen referenzielle Transparenz — Fehler sind Werte
type Result<E, A> =
  | { readonly _tag: 'Ok';  readonly value: A }
  | { readonly _tag: 'Err'; readonly error: E };

const Result = {
  ok:       <E, A>(value: A): Result<E, A>                  => ({ _tag: 'Ok',  value }),
  err:      <E, A>(error: E): Result<E, A>                  => ({ _tag: 'Err', error }),
  map:      <E, A, B>(r: Result<E, A>, f: (a: A) => B): Result<E, B> =>
              r._tag === 'Ok' ? Result.ok(f(r.value)) : r,
  flatMap:  <E, A, B>(r: Result<E, A>, f: (a: A) => Result<E, B>): Result<E, B> =>
              r._tag === 'Ok' ? f(r.value) : r,
  fold:     <E, A, B>(r: Result<E, A>, onErr: (e: E) => B, onOk: (a: A) => B): B =>
              r._tag === 'Ok' ? onOk(r.value) : onErr(r.error),
  tryCatch: <E, A>(f: () => A, toError: (e: unknown) => E): Result<E, A> => {
    try { return Result.ok(f()); }
    catch (e) { return Result.err(toError(e)); }
  },
  // [v2.0] Sequenz: alle-oder-nichts über mehrere Results
  sequence: <E, A>(results: Result<E, A>[]): Result<E, A[]> => {
    const values: A[] = [];
    for (const r of results) {
      if (r._tag === 'Err') return r;
      values.push(r.value);
    }
    return Result.ok(values);
  },
  // [v2.0] Sammle alle Fehler statt beim ersten zu stoppen
  validate: <E, A>(results: Result<E, A>[]): Result<E[], A[]> => {
    const errors: E[] = [];
    const values: A[] = [];
    for (const r of results) {
      if (r._tag === 'Err') errors.push(r.error);
      else values.push(r.value);
    }
    return errors.length > 0 ? Result.err(errors) : Result.ok(values);
  },
} as const;
```

### 1.3 Branded Types [ERWEITERT — v2.0]

```typescript
// [FP-Scala] Make impossible states impossible
declare const __brand: unique symbol;
type Brand<B>        = { readonly [__brand]: B };
type Branded<T, B>   = T & Brand<B>;

// Identity-Typen
type SnippetId    = Branded<string, 'SnippetId'>;
type ScriptId     = Branded<string, 'ScriptId'>;
type PipelineId   = Branded<string, 'PipelineId'>;
type FolderId     = Branded<string, 'FolderId'>;
type TagName      = Branded<string, 'TagName'>;
type FilePath     = Branded<string, 'FilePath'>;
type TemplateId   = Branded<string, 'TemplateId'>;    // [NEU v2.0]
type ClipEntryId  = Branded<string, 'ClipEntryId'>;   // [NEU v2.0]
type ScriptVerId  = Branded<string, 'ScriptVerId'>;   // [NEU v2.0]
type BundleId     = Branded<string, 'BundleId'>;      // [NEU v2.0]

// Maß-Typen
type UnixMs       = Branded<number, 'UnixMs'>;
type ByteSize     = Branded<number, 'ByteSize'>;
type TokenCount   = Branded<number, 'TokenCount'>;    // [NEU v2.0]
type LineNumber   = Branded<number, 'LineNumber'>;    // [NEU v2.0]

// Konstruktoren
const SnippetId   = { of: (s: string) => s as SnippetId };
const ScriptId    = { of: (s: string) => s as ScriptId };
const PipelineId  = { of: (s: string) => s as PipelineId };
const FolderId    = { of: (s: string) => s as FolderId };
const TemplateId  = { of: (s: string) => s as TemplateId };
const ClipEntryId = { of: (s: string) => s as ClipEntryId };

const TagName = {
  parse: (raw: string): Result<DomainError, TagName> =>
    /^[a-z0-9_\-]{1,32}$/.test(raw.trim().toLowerCase())
      ? Result.ok(raw.trim().toLowerCase() as TagName)
      : Result.err({ code: 'INVALID_TAG', raw }),
};

// [v2.0] NonEmptyArray — verhindert leere Arrays wo ein Element Pflicht ist
type NonEmptyArray<A> = [A, ...A[]];
const NonEmptyArray = {
  of:   <A>(first: A, rest: A[] = []): NonEmptyArray<A> => [first, ...rest],
  head: <A>(arr: NonEmptyArray<A>): A => arr[0],
  fromArray: <A>(arr: A[]): Option<NonEmptyArray<A>> =>
    arr.length > 0 ? Option.some(arr as NonEmptyArray<A>) : Option.none(),
};
```

---

## § 2 — DOMÄNEN-DATENTYPEN [ERWEITERT — v2.0]

### 2.1 Snippet — Kern-Entity [v1.0-kompatibel]

```typescript
// [DDD] Aggregate Root: Snippet kapselt alle Invarianten
interface Snippet {
  // Identity
  readonly id:          SnippetId;
  readonly title:       string;         // 1–128 Zeichen
  readonly content:     string;         // max 10 MB

  // Klassifikation
  readonly tags:        ReadonlyArray<TagName>;
  readonly location:    SnippetLocation;
  readonly contentType: ContentType;

  // Zeitstempel (UTC, ms)
  readonly createdAt:   UnixMs;
  readonly updatedAt:   UnixMs;

  // Statistik
  readonly usageCount:  number;
  readonly isPinned:    boolean;

  // [v2.0] Neue Felder
  readonly sourceApp:   Option<string>;   // Quell-Anwendung (bei Clipboard-Import)
  readonly isTemplate:  boolean;          // Enthält {{variable}}-Platzhalter
  readonly color:       Option<string>;   // Hex-Farbe für visuelle Markierung (#RRGGBB)
  readonly favorite:    boolean;          // Schnellzugriff
}

// Invarianten (in allen Mutations-Funktionen geprüft):
// INVARIANT-1: title.length ∈ [1, 128]
// INVARIANT-2: content.length ≤ 10 * 1024 * 1024
// INVARIANT-3: tags ist ein Set (keine Duplikate), max 20 Tags
// INVARIANT-4: updatedAt ≥ createdAt
// INVARIANT-5: usageCount ≥ 0
// INVARIANT-6: color matches /^#[0-9A-Fa-f]{6}$/ oder None
// [v2.0] INVARIANT-7: isTemplate === true gdw. content enthält /\{\{[\w.]+\}\}/

type SnippetPatch = Partial<Pick<Snippet,
  'title' | 'content' | 'tags' | 'location' | 'isPinned' | 'color' | 'favorite'
>>;

const Snippet = {
  create: (draft: {
    title:    string;
    content:  string;
    location: SnippetLocation;
    sourceApp?: string;
  }): Result<DomainError, Snippet> => {
    if (draft.title.trim().length === 0) return Result.err({ code: 'EMPTY_TITLE' });
    if (draft.title.length > 128)        return Result.err({ code: 'TITLE_TOO_LONG', max: 128 });
    if (draft.content.length > 10 * 1024 * 1024)
      return Result.err({ code: 'CONTENT_TOO_LARGE', maxBytes: 10 * 1024 * 1024 });

    const now = Date.now() as UnixMs;
    const content = draft.content;
    return Result.ok({
      id:          SnippetId.of(crypto.randomUUID()),
      title:       draft.title.trim(),
      content,
      tags:        [],
      location:    draft.location,
      contentType: detectContentType(content),
      isTemplate:  /\{\{[\w.]+\}\}/.test(content),
      createdAt:   now,
      updatedAt:   now,
      usageCount:  0,
      isPinned:    false,
      favorite:    false,
      sourceApp:   Option.fromNullable(draft.sourceApp),
      color:       Option.none(),
    });
  },

  // [Immutability] update erzeugt immer ein neues Objekt
  update: (snippet: Snippet, patch: SnippetPatch): Result<DomainError, Snippet> => {
    const merged = { ...snippet, ...patch, updatedAt: Date.now() as UnixMs };
    // Revalidierung der Invarianten
    if (patch.content !== undefined) {
      merged.contentType = detectContentType(merged.content);
      merged.isTemplate  = /\{\{[\w.]+\}\}/.test(merged.content);
    }
    return Snippet.validate(merged);
  },

  duplicate: (original: Snippet): Result<DomainError, Snippet> =>
    Snippet.create({
      title:    `${original.title} (Kopie)`,
      content:  original.content,
      location: original.location,
    }),

  validate: (s: Snippet): Result<DomainError, Snippet> => {
    if (!s.title || s.title.trim().length === 0) return Result.err({ code: 'EMPTY_TITLE' });
    if (s.title.length > 128)                    return Result.err({ code: 'TITLE_TOO_LONG', max: 128 });
    if (new Set(s.tags).size !== s.tags.length)  return Result.err({ code: 'DUPLICATE_TAGS' });
    if (s.tags.length > 20)                      return Result.err({ code: 'TOO_MANY_TAGS', max: 20 });
    if (s.color._tag === 'Some' && !/^#[0-9A-Fa-f]{6}$/.test(s.color.value))
      return Result.err({ code: 'INVALID_COLOR', value: s.color.value });
    return Result.ok(s);
  },

  // [v2.0] Abgeleitete Metadaten — reine Funktionen, niemals gespeichert
  sizeBytes:  (s: Snippet): ByteSize  => new TextEncoder().encode(s.content).length as ByteSize,
  wordCount:  (s: Snippet): number    => s.content.trim() === '' ? 0 : s.content.trim().split(/\s+/).length,
  lineCount:  (s: Snippet): number    => s.content.split('\n').length,
  charCount:  (s: Snippet): number    => s.content.length,
  charNoSpaceCount: (s: Snippet): number => s.content.replace(/\s/g, '').length,
} as const;
```

### 2.2 SnippetLocation — Ort / Reiter [v1.0-kompatibel]

```typescript
type SnippetLocation =
  | { readonly _type: 'inbox'   }
  | { readonly _type: 'archive' }
  | { readonly _type: 'trash';  readonly deletedAt: UnixMs }
  | {
      readonly _type:    'folder';
      readonly folderId: FolderId;
      readonly path:     ReadonlyArray<string>;  // Breadcrumbs
    };

interface Folder {
  readonly id:        FolderId;
  readonly name:      string;               // 1–64 Zeichen
  readonly parentId:  Option<FolderId>;
  readonly sortOrder: number;
  readonly icon:      Option<string>;       // Emoji oder Icon-Name
  readonly color:     Option<string>;       // Hex-Farbe (#RRGGBB)
  readonly createdAt: UnixMs;
  // [v2.0]
  readonly snippetCount: number;            // Derived — nicht gespeichert
}
```

### 2.3 ContentType — Automatische Klassifikation [ERWEITERT — v2.0]

```typescript
// [FP-Scala] Totale Funktion — detectContentType(content) ist referentiell transparent
type ContentType =
  // Text & Markup
  | 'plain_text'   | 'markdown'    | 'html'        | 'xml'
  // Programmiersprachen
  | 'javascript'   | 'typescript'  | 'python'       | 'rust'
  | 'go'           | 'java'        | 'kotlin'       | 'swift'
  | 'cpp'          | 'c'           | 'csharp'       | 'php'
  | 'ruby'         | 'bash'        | 'powershell'
  // Datenformate
  | 'json'         | 'yaml'        | 'toml'         | 'csv'
  | 'sql'          | 'graphql'
  // Style
  | 'css'          | 'scss'        | 'less'
  // Spezial
  | 'url'          | 'file_path'   | 'regex'        | 'template'
  | 'unknown';

// [FP-Scala] Totale Funktion — jeder Input hat einen Output
const detectContentType = (content: string): ContentType => {
  const t = content.trim();
  // Kurzschlüsse für eindeutige Muster
  if (/^https?:\/\//.test(t)) return 'url';
  if (/^(\/[\w.-]+)+\/?$/.test(t) || /^[A-Za-z]:\\/.test(t)) return 'file_path';
  if ((t.startsWith('{') || t.startsWith('[')) && isValidJSON(t)) return 'json';
  if (/\{\{[\w.]+\}\}/.test(t)) return 'template';

  // Markdown-Heuristiken (Ordnung ist wichtig)
  if (/^---\n/.test(t) || /#{1,6}\s/.test(t) || /\*\*|__|\[.+\]\(/.test(t)) return 'markdown';
  if (/^<[a-zA-Z][^>]*>/.test(t) && /<\/[a-zA-Z]+>/.test(t)) return 'html';
  if (/^<\?xml|^<[a-zA-Z]+:[a-zA-Z]+/.test(t)) return 'xml';

  // Programmiersprachen
  if (/^(const|let|var|function|import|export|class|=>)\s/.test(t) && !t.includes('def ')) return 'javascript';
  if (/^(interface|type |enum |namespace|declare)\s/.test(t)) return 'typescript';
  if (/^(def |class |import |from .* import|async def|@\w+\n)/.test(t)) return 'python';
  if (/^(fn |use |mod |struct |impl |pub |enum |trait )/.test(t)) return 'rust';
  if (/^(package |import "|(func|type|var|const) \w)/.test(t)) return 'go';
  if (/^(public |private |class |interface |import java)/.test(t)) return 'java';
  if (/^(fun |val |var |data class|object |companion)/.test(t)) return 'kotlin';
  if (/^(import (Foundation|UIKit|SwiftUI)|func |var |let |class )/.test(t)) return 'swift';
  if (/^(#include|int main|void |printf|struct |typedef)/.test(t)) return 'c';
  if (/^(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP)\s/i.test(t)) return 'sql';
  if (/^(query|mutation|subscription|type \w+ \{)/.test(t)) return 'graphql';
  if (/^(#!\/bin\/(bash|sh|zsh)|echo |grep |sed |awk )/.test(t)) return 'bash';
  if (/^(---|\w+:\s*\n  \w+:)/.test(t)) return 'yaml';
  if (/^\[[\w.-]+\]\n\w+ *= /.test(t)) return 'toml';
  if (/^(\.|#)[a-zA-Z][\w-]*\s*\{/.test(t)) return 'css';

  return 'plain_text';
};

const isValidJSON = (s: string): boolean => {
  try { JSON.parse(s); return true; } catch { return false; }
};
```

### 2.4 DomainError — Erschöpfendes Fehlermodell [ERWEITERT — v2.0]

```typescript
// [FP-Scala] Fehler als Werte — vollständige Aufzählung aller möglichen Fehler
type DomainError =
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

  // Script
  | { readonly code: 'SCRIPT_SYNTAX_ERROR';     readonly details: string; readonly line?: number }
  | { readonly code: 'SCRIPT_RUNTIME_ERROR';    readonly details: string }
  | { readonly code: 'SCRIPT_TIMEOUT';          readonly limitMs: number }
  | { readonly code: 'SCRIPT_OUTPUT_TOO_LARGE'; readonly actualBytes: number; readonly limitBytes: number }
  | { readonly code: 'SCRIPT_INPUT_TOO_LARGE';  readonly actualBytes: number; readonly limitBytes: number }
  | { readonly code: 'SCRIPT_NOT_FOUND';        readonly id: ScriptId }
  | { readonly code: 'SCRIPT_INVALID_OUTPUT';   readonly details: string }  // [v2.0] Output kein String

  // Template [NEU v2.0]
  | { readonly code: 'TEMPLATE_MISSING_VARIABLE'; readonly variable: string }
  | { readonly code: 'TEMPLATE_CIRCULAR_REF';     readonly variable: string }
  | { readonly code: 'TEMPLATE_PARSE_ERROR';      readonly details: string }

  // Pipeline
  | { readonly code: 'PIPELINE_ABORTED';         readonly atStep: string; readonly reason: DomainError }
  | { readonly code: 'PIPELINE_NOT_FOUND';        readonly id: PipelineId }
  | { readonly code: 'PIPELINE_EMPTY' }
  | { readonly code: 'PIPELINE_STEP_NOT_FOUND';   readonly stepId: string }

  // Clipboard [NEU v2.0]
  | { readonly code: 'CLIPBOARD_READ_ERROR';      readonly details: string }
  | { readonly code: 'CLIPBOARD_WRITE_ERROR';     readonly details: string }
  | { readonly code: 'CLIPBOARD_EMPTY' }

  // Import/Export [NEU v2.0]
  | { readonly code: 'EXPORT_WRITE_ERROR';        readonly path: string; readonly details: string }
  | { readonly code: 'IMPORT_PARSE_ERROR';        readonly details: string }
  | { readonly code: 'IMPORT_VERSION_MISMATCH';   readonly expected: string; readonly got: string }
  | { readonly code: 'IMPORT_CHECKSUM_MISMATCH' }

  // Undo [NEU v2.0]
  | { readonly code: 'UNDO_STACK_EMPTY' }
  | { readonly code: 'REDO_STACK_EMPTY' }

  // Storage
  | { readonly code: 'STORAGE_ERROR';             readonly details: string }
  | { readonly code: 'CONSTRAINT_ERROR';          readonly details: string }
  | { readonly code: 'MIGRATION_ERROR';           readonly details: string }  // [v2.0]

  // Filter
  | { readonly code: 'INVALID_FILTER';            readonly field: string; readonly reason: string };

// Menschenlesbare Beschreibungen
const DomainError = {
  describe: (e: DomainError): string => {
    switch (e.code) {
      case 'EMPTY_TITLE':                return 'Titel darf nicht leer sein.';
      case 'TITLE_TOO_LONG':             return `Titel zu lang (max. ${e.max} Zeichen).`;
      case 'CONTENT_TOO_LARGE':          return `Inhalt zu groß (max. 10 MB).`;
      case 'SCRIPT_TIMEOUT':             return `Skript überschritt Zeitlimit (${e.limitMs} ms).`;
      case 'SCRIPT_OUTPUT_TOO_LARGE':    return `Ausgabe zu groß (${e.actualBytes} > ${e.limitBytes} Bytes).`;
      case 'PIPELINE_ABORTED':           return `Pipeline gestoppt bei Schritt "${e.atStep}".`;
      case 'TEMPLATE_MISSING_VARIABLE':  return `Template-Variable "{{${e.variable}}}" hat keinen Wert.`;
      case 'CLIPBOARD_EMPTY':            return 'Zwischenablage ist leer.';
      case 'UNDO_STACK_EMPTY':           return 'Nichts zum Rückgängigmachen.';
      case 'IMPORT_VERSION_MISMATCH':    return `Bundle-Version ${e.got} nicht kompatibel (erwartet ${e.expected}).`;
      default:                           return `Fehler: ${e.code}`;
    }
  },
} as const;
```

### 2.5 ClipboardEntry [NEU — v2.0]

```typescript
// [DDD] ClipboardEntry ist ein Value Object — einmal erstellt, niemals mutiert
// Captured clipboard snapshots werden persistent gespeichert
interface ClipboardEntry {
  readonly id:           ClipEntryId;
  readonly content:      string;
  readonly contentHash:  string;          // SHA-256 für Dedup
  readonly contentType:  ContentType;     // Automatisch erkannt
  readonly sourceApp:    Option<string>;  // Quell-Anwendung (Wayland: KWin D-Bus / xdg-foreign)
  readonly capturedAt:   UnixMs;

  // Filterbare Metadaten
  readonly sizeBytes:    ByteSize;
  readonly lineCount:    number;
  readonly wordCount:    number;
  readonly isPinned:     boolean;
  readonly tags:         ReadonlyArray<TagName>;

  // Lebenszyklus
  readonly promotedToSnippetId: Option<SnippetId>;  // Wurde zu Snippet konvertiert
}

// Invarianten:
// CLIP-INV-1: content !== '' (leere Einträge werden verworfen)
// CLIP-INV-2: sizeBytes === new TextEncoder().encode(content).length
// CLIP-INV-3: Zwei Einträge mit identischem contentHash → Duplikat (wird verworfen)
// CLIP-INV-4: Maximale Anzahl Einträge: settings.clipboard.maxEntries (default: 500)
//             älteste unpinned Einträge werden automatisch gelöscht (LRU)

const ClipboardEntry = {
  create: (content: string, sourceApp: Option<string>): Option<ClipboardEntry> => {
    if (content.trim().length === 0) return Option.none(); // Leere Inhalte verwerfen
    const bytes   = new TextEncoder().encode(content);
    const now     = Date.now() as UnixMs;
    return Option.some({
      id:           ClipEntryId.of(crypto.randomUUID()),
      content,
      contentHash:  sha256(content),  // Rust-Seite: SHA-256
      contentType:  detectContentType(content),
      sourceApp,
      capturedAt:   now,
      sizeBytes:    bytes.length as ByteSize,
      lineCount:    content.split('\n').length,
      wordCount:    content.trim() === '' ? 0 : content.trim().split(/\s+/).length,
      isPinned:     false,
      tags:         [],
      promotedToSnippetId: Option.none(),
    });
  },

  // Ein ClipboardEntry als Snippet-Basis verwenden
  toSnippetDraft: (entry: ClipboardEntry): { title: string; content: string; location: SnippetLocation } => ({
    title:    entry.content.slice(0, 60).trim() || 'Clipboard-Import',
    content:  entry.content,
    location: { _type: 'inbox' },
  }),
} as const;
```

### 2.6 TextStats — Text-Analyse-Ergebnis [NEU — v2.0]

```typescript
// [FP-Scala] Reine Funktion: TextStats ist ein unveränderlicher Snapshot
interface TextStats {
  // Grundzählung
  readonly charCount:        number;
  readonly charNoSpaceCount: number;
  readonly wordCount:        number;
  readonly lineCount:        number;
  readonly paragraphCount:   number;
  readonly sentenceCount:    number;   // Heuristisch: Endet mit . ! ?

  // Token-Schätzung (cl100k_base / GPT-4-Kompatibel)
  // Formel: max(charCount / 4, wordCount * 0.75) — schnelle Approximation
  readonly estimatedTokens:  TokenCount;

  // Wort-Statistiken
  readonly uniqueWordCount:  number;
  readonly avgWordLength:    number;    // Zeichen pro Wort, 2 Dezimalstellen
  readonly longestWord:      string;
  readonly mostFrequentWords: ReadonlyArray<{ word: string; count: number }>; // Top 10

  // Lesbarkeit
  readonly avgSentenceLength: number;  // Wörter pro Satz
  readonly fleschKincaidGrade: Option<number>; // None wenn < 100 Wörter

  // Zeilenstats
  readonly avgLineLength:    number;
  readonly longestLineLength: number;
  readonly emptyLineCount:   number;

  // Zeitmessung
  readonly readingTimeMs:    number;   // ~200 WPM
  readonly computedAt:       UnixMs;
}

// [FP-Scala] Totale reine Funktion
const computeTextStats = (content: string): TextStats => {
  const charCount        = content.length;
  const charNoSpaceCount = content.replace(/\s/g, '').length;
  const words            = content.trim() === '' ? [] : content.trim().split(/\s+/);
  const lines            = content.split('\n');
  const sentences        = content.match(/[^.!?]+[.!?]+/g) ?? [];

  const wordFreq: Record<string, number> = {};
  for (const w of words) {
    const lw = w.toLowerCase().replace(/[^\w]/g, '');
    if (lw.length > 2) wordFreq[lw] = (wordFreq[lw] ?? 0) + 1;
  }
  const mostFrequentWords = Object.entries(wordFreq)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10)
    .map(([word, count]) => ({ word, count }));

  const wordCount       = words.length;
  const uniqueWordCount = new Set(words.map(w => w.toLowerCase())).size;
  const totalWordLen    = words.reduce((s, w) => s + w.length, 0);
  const avgWordLength   = wordCount > 0 ? Math.round((totalWordLen / wordCount) * 100) / 100 : 0;
  const longestWord     = words.reduce((a, b) => b.length > a.length ? b : a, '');

  const lineCount        = lines.length;
  const emptyLineCount   = lines.filter(l => l.trim() === '').length;
  const avgLineLength    = lineCount > 0 ? Math.round(charCount / lineCount * 10) / 10 : 0;
  const longestLineLength = Math.max(...lines.map(l => l.length), 0);

  const sentenceCount     = sentences.length;
  const paragraphs        = content.split(/\n\s*\n/).filter(p => p.trim().length > 0);
  const paragraphCount    = paragraphs.length;
  const avgSentenceLength = sentenceCount > 0 ? Math.round(wordCount / sentenceCount * 10) / 10 : 0;

  const estimatedTokens  = Math.round(Math.max(charCount / 4, wordCount * 0.75)) as TokenCount;
  const readingTimeMs    = Math.round((wordCount / 200) * 60_000);

  // Flesch-Kincaid — nur bei ausreichend Text sinnvoll
  const fleschKincaidGrade: Option<number> = wordCount >= 100 && sentenceCount > 0
    ? Option.some(
        Math.round(
          (0.39 * (wordCount / sentenceCount) + 11.8 * (charNoSpaceCount / wordCount) - 15.59) * 10
        ) / 10
      )
    : Option.none();

  return {
    charCount, charNoSpaceCount, wordCount, lineCount, paragraphCount,
    sentenceCount, estimatedTokens, uniqueWordCount, avgWordLength,
    longestWord, mostFrequentWords, avgSentenceLength, fleschKincaidGrade,
    avgLineLength, longestLineLength, emptyLineCount,
    readingTimeMs, computedAt: Date.now() as UnixMs,
  };
};
```

### 2.7 DiffResult — Textvergleich [NEU — v2.0]

```typescript
// Verwendet Myers-Diff-Algorithmus (Rust-Seite: similar crate)
// [PhilSD] Tiefes Modul: Diff-Details hinter einfacher Schnittstelle

type DiffLineKind = 'equal' | 'insert' | 'delete' | 'replace';

interface DiffLine {
  readonly kind:        DiffLineKind;
  readonly oldLineNum:  Option<LineNumber>;
  readonly newLineNum:  Option<LineNumber>;
  readonly content:     string;
}

interface DiffResult {
  readonly lines:        ReadonlyArray<DiffLine>;
  readonly addedLines:   number;
  readonly deletedLines: number;
  readonly unchanged:    number;
  readonly similarity:   number;    // 0.0 – 1.0 (Jaccard-Ähnlichkeit auf Tokens)
}

// Inline-Diff für Zeichen innerhalb einer Zeile
interface InlineDiff {
  readonly kind:    'equal' | 'insert' | 'delete';
  readonly text:    string;
}
```

### 2.8 WorkspaceSession [NEU — v2.0]

```typescript
// [DDD] Workspace-Zustand — was beim nächsten App-Start wiederhergestellt wird
// Persistiert als JSON in settings.key = 'session.workspace'

interface WorkspaceSession {
  readonly activeView:       AppView;
  readonly lastActiveSnippetId: Option<SnippetId>;
  readonly lastActiveScriptId:  Option<ScriptId>;
  readonly lastActivePipelineId: Option<PipelineId>;
  readonly sidebarWidth:     number;          // Pixel (Default: 280)
  readonly previewMode:      PreviewMode;
  readonly filterState:      SnippetFilter;   // Letzter Filter bleibt erhalten
  readonly openEditorTabs:   ReadonlyArray<EditorTab>;  // Max 8 Tabs
  readonly savedAt:          UnixMs;
}

type AppView = 'snippets' | 'scripts' | 'pipelines' | 'clipboard' | 'settings';

type PreviewMode = 'editor' | 'preview' | 'split';

interface EditorTab {
  readonly entityType: 'snippet' | 'script' | 'pipeline';
  readonly entityId:   string;
  readonly isDirty:    boolean;           // Ungespeicherte Änderungen
  readonly scrollPos:  number;            // Scroll-Position in Pixeln
  readonly cursorLine: Option<LineNumber>;
}
```

---

## § 3 — SCRIPT [ERWEITERT — v2.0]

### 3.1 Script-Entity [v1.0-kompatibel, erweitert]

```typescript
// [DDD] Script ist ein eigenständiges Aggregate
interface Script {
  readonly id:               ScriptId;
  readonly name:             string;         // 1–64 Zeichen
  readonly description:      string;
  readonly type:             ScriptType;
  readonly category:         ScriptCategory;

  readonly jsCode:           Option<string>;
  readonly regexPattern:     Option<string>;
  readonly regexReplacement: Option<string>;
  readonly regexFlags:       RegexFlags;

  readonly parameters:       ReadonlyArray<ScriptParameter>;
  readonly tests:            ReadonlyArray<ScriptTest>;

  readonly isFavorite:       boolean;
  readonly isSafetyCritical: boolean;
  readonly usageCount:       number;
  readonly lastUsedAt:       Option<UnixMs>;
  readonly createdAt:        UnixMs;
  readonly updatedAt:        UnixMs;

  // [v2.0] Neue Felder
  readonly tags:             ReadonlyArray<TagName>;
  readonly currentVersion:   number;          // Inkrementiert bei jeder Speicherung
  readonly color:            Option<string>;  // Visuelle Markierung
}

type ScriptType     = 'js' | 'regex' | 'builtin';
type ScriptCategory = 'text' | 'code' | 'security' | 'format' | 'analysis' | 'custom';
```

### 3.2 ScriptParameter [v1.0-kompatibel]

```typescript
// [PhilSD] Tiefes Modul: Komplexe Skripte hinter einfacher GUI
type ScriptParameter =
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
  // [v2.0] Mehrzeilige Texteingabe
  | { readonly _type: 'textarea'; readonly key: string; readonly label: string; readonly default: string;
      readonly rows: number; readonly placeholder: string };

type ParameterValues = Readonly<Record<string, string | number | boolean>>;
```

### 3.3 ScriptTest [v1.0-kompatibel]

```typescript
interface ScriptTest {
  readonly id:         string;
  readonly label:      string;
  readonly input:      string;
  readonly parameters: ParameterValues;
  readonly expected:   string;
  readonly lastResult: Option<TestResult>;
}

type TestResult =
  | { readonly _tag: 'Pass';  readonly actual: string;  readonly durationMs: number }
  | { readonly _tag: 'Fail';  readonly actual: string;  readonly durationMs: number }
  | { readonly _tag: 'Error'; readonly message: string; readonly durationMs: number };

interface ScriptTestSummary {
  readonly total:   number;
  readonly passed:  number;
  readonly failed:  number;
  readonly errors:  number;
  readonly ranAt:   UnixMs;
}
```

### 3.4 RegexFlags [v1.0-kompatibel]

```typescript
interface RegexFlags {
  readonly global:     boolean;  // g
  readonly ignoreCase: boolean;  // i
  readonly multiline:  boolean;  // m
  readonly dotAll:     boolean;  // s
  readonly unicode:    boolean;  // u
}
const RegexFlags = {
  default:      (): RegexFlags         => ({ global: true, ignoreCase: false, multiline: false, dotAll: false, unicode: true }),
  toFlagString: (f: RegexFlags): string =>
    [f.global && 'g', f.ignoreCase && 'i', f.multiline && 'm', f.dotAll && 's', f.unicode && 'u']
      .filter(Boolean).join(''),
  fromFlagString: (s: string): RegexFlags => ({
    global: s.includes('g'), ignoreCase: s.includes('i'), multiline: s.includes('m'),
    dotAll: s.includes('s'), unicode: s.includes('u'),
  }),
} as const;
```

### 3.5 ScriptVersion [NEU — v2.0]

```typescript
// [PragProg] Versionierung ist kein Extra — es ist Sicherheitsnetz für Experimente
interface ScriptVersion {
  readonly id:          ScriptVerId;
  readonly scriptId:    ScriptId;
  readonly version:     number;         // Monoton steigend
  readonly jsCode:      Option<string>;
  readonly regexPattern: Option<string>;
  readonly regexReplacement: Option<string>;
  readonly regexFlags:  RegexFlags;
  readonly parameters:  ReadonlyArray<ScriptParameter>;
  readonly savedAt:     UnixMs;
  readonly changeNote:  Option<string>; // Optionaler Änderungsgrund
}

// Invarianten:
// SCRVER-1: version ist eindeutig pro scriptId
// SCRVER-2: Maximal 20 Versionen pro Skript werden aufbewahrt (FIFO)
// SCRVER-3: Builtin-Skripte haben keine Versionen
```

---

## § 4 — TRANSFORMATION PIPELINE [v1.0-kompatibel, erweitert]

### 4.1 Pipeline-Entity [v1.0-kompatibel]

```typescript
interface Pipeline {
  readonly id:          PipelineId;
  readonly name:        string;
  readonly description: string;
  readonly steps:       ReadonlyArray<PipelineStep>;
  readonly strictMode:  boolean;
  readonly createdAt:   UnixMs;
  readonly updatedAt:   UnixMs;
  // [v2.0]
  readonly tags:        ReadonlyArray<TagName>;
  readonly favorite:    boolean;
  readonly isTemplate:  boolean;   // Kann als Vorlage für neue Pipelines verwendet werden
}

interface PipelineStep {
  readonly id:              string;
  readonly order:           number;
  readonly scriptId:        Option<ScriptId>;
  readonly builtinId:       Option<BuiltinId>;
  readonly parameterValues: ParameterValues;
  readonly enabled:         boolean;
  readonly label:           string;
  readonly failurePolicy:   FailurePolicy;
  readonly isSafetyCritical: boolean;
  // [v2.0]
  readonly condition:       Option<PipelineCondition>;  // Schritt nur ausführen wenn Bedingung erfüllt
}

// [v2.0] Bedingte Pipeline-Ausführung
type PipelineCondition =
  | { readonly _type: 'content_type_is'; readonly types: ReadonlyArray<ContentType> }
  | { readonly _type: 'size_gt';         readonly bytes: ByteSize }
  | { readonly _type: 'size_lt';         readonly bytes: ByteSize }
  | { readonly _type: 'contains_regex';  readonly pattern: string }
  | { readonly _type: 'line_count_gt';   readonly n: number };

type FailurePolicy = 'abort' | 'warn' | 'passthrough';
```

### 4.2 Pipeline-Ausführungsergebnis [v1.0-kompatibel]

```typescript
interface PipelineExecutionResult {
  readonly success:         boolean;
  readonly finalOutput:     Option<string>;
  readonly stepResults:     ReadonlyArray<StepExecutionResult>;
  readonly warnings:        ReadonlyArray<PipelineWarning>;
  readonly totalDurationMs: number;
  // [v2.0]
  readonly skippedSteps:    ReadonlyArray<string>;  // StepIds die durch Condition übersprungen
}

interface StepExecutionResult {
  readonly stepId:         string;
  readonly label:          string;
  readonly input:          string;
  readonly output:         string;
  readonly durationMs:     number;
  readonly error:          Option<DomainError>;
  readonly policyApplied:  FailurePolicy;
  readonly wasSkipped:     boolean;           // [v2.0] Durch Condition übersprungen
  readonly conditionResult: Option<boolean>;  // [v2.0] Ergebnis der Condition-Prüfung
}
```

### 4.3 Eingebaute Transformationen (Builtins) [STARK ERWEITERT — v2.0]

```typescript
// [PragProg] DRY: BUILTIN_REGISTRY ist die EINZIGE Quelle der Wahrheit
type BuiltinId =
  // ── Text-Grundoperationen ─────────────────────────────────────────
  | 'trim'                   // Führende/folgende Leerzeichen entfernen
  | 'remove_empty_lines'     // Leerzeilen löschen
  | 'collapse_whitespace'    // Mehrfaches Whitespace auf 1 reduzieren
  | 'normalize_whitespace'   // Alle Whitespace-Varianten → reguläres Leerzeichen [v2.0]
  | 'remove_non_ascii'       // Alle Nicht-ASCII-Zeichen entfernen [v2.0]
  | 'normalize_unicode'      // NFC-Normalisierung (Akzente kombinieren) [v2.0]
  | 'remove_accents'         // Diakritische Zeichen entfernen (é→e) [v2.0]
  | 'truncate'               // Text auf N Zeichen kürzen (mit …)
  | 'summary_cut'            // Anfang+Ende behalten, Mitte kürzen (für Logs)
  | 'first_n_lines'          // Erste N Zeilen behalten [v2.0]
  | 'last_n_lines'           // Letzte N Zeilen behalten [v2.0]
  | 'wrap_text'              // Zeilenumbruch bei N Zeichen (Word-Wrap) [v2.0]

  // ── Groß-/Kleinschreibung ─────────────────────────────────────────
  | 'uppercase'              // GROSSBUCHSTABEN
  | 'lowercase'              // kleinbuchstaben
  | 'title_case'             // Jedes Wort Groß
  | 'sentence_case'          // Erster Buchstabe groß, Rest klein [v2.0]
  | 'alternating_case'       // wEcHsElNdE gRoSsScHrEiBuNg [v2.0]
  | 'rot13'                  // ROT13-Verschlüsselung [v2.0]

  // ── Zeilenoperationen ─────────────────────────────────────────────
  | 'sort_lines'             // Zeilen alphabetisch sortieren [v2.0]
  | 'sort_lines_desc'        // Zeilen umgekehrt alphabetisch [v2.0]
  | 'sort_lines_by_length'   // Zeilen nach Länge sortieren [v2.0]
  | 'reverse_lines'          // Zeilenreihenfolge umkehren [v2.0]
  | 'unique_lines'           // Doppelte Zeilen entfernen [v2.0]
  | 'shuffle_lines'          // Zeilen zufällig mischen [v2.0]
  | 'add_line_numbers'       // "1. ", "2. " voranstellen [v2.0]
  | 'remove_line_numbers'    // Führende Nummern entfernen [v2.0]
  | 'prefix_lines'           // Präfix vor jede Zeile [v2.0]
  | 'suffix_lines'           // Suffix nach jede Zeile [v2.0]
  | 'indent'                 // N Leerzeichen/Tabs einrücken [v2.0]
  | 'dedent'                 // Gemeinsamen Einzug entfernen [v2.0]
  | 'join_lines'             // Zeilen mit Trennzeichen verbinden [v2.0]
  | 'reverse_text'           // Zeichenkette umkehren [v2.0]

  // ── Code-Operationen ──────────────────────────────────────────────
  | 'wrap_markdown_block'    // ```sprache ... ``` um Code
  | 'strip_markdown'         // Markdown-Syntax entfernen
  | 'markdown_to_html'       // Markdown → HTML rendern [v2.0]
  | 'strip_html_tags'        // HTML-Tags entfernen, Text behalten [v2.0]
  | 'pretty_json'            // JSON formatieren (2-Space-Indent)
  | 'minify_json'            // JSON komprimieren
  | 'minify_code'            // Kommentare und Whitespace entfernen
  | 'extract_code_blocks'    // Alle ```...``` Blöcke extrahieren
  | 'extract_errors'         // Exceptions und Stacktraces filtern
  | 'extract_json_keys'      // Alle JSON-Schlüssel als Liste [v2.0]
  | 'flatten_json'           // Verschachteltes JSON flach machen [v2.0]
  | 'xml_pretty'             // XML formatieren [v2.0]
  | 'xml_minify'             // XML komprimieren [v2.0]
  | 'remove_comments'        // Code-Kommentare entfernen (//  # /* */) [v2.0]
  | 'escape_json_string'     // String für JSON-Wert escapen [v2.0]
  | 'unescape_json_string'   // JSON-escapes auflösen [v2.0]

  // ── Kodierung/Dekodierung [v2.0] ──────────────────────────────────
  | 'base64_encode'
  | 'base64_decode'
  | 'url_encode'             // %XX-Codierung des gesamten Strings
  | 'url_encode_component'   // Nur Sonderzeichen codieren
  | 'url_decode'
  | 'html_entity_encode'     // & → &amp; < → &lt; etc.
  | 'html_entity_decode'
  | 'hash_sha256'            // SHA-256 des Inhalts als Hex [v2.0]

  // ── Namenskonventionen ────────────────────────────────────────────
  | 'camel_to_snake'
  | 'snake_to_camel'
  | 'snake_to_pascal'        // [v2.0]
  | 'to_slug'                // Für URLs: "Hallo Welt" → "hallo-welt"
  | 'to_kebab_case'          // [v2.0]
  | 'to_constant_case'       // SCREAMING_SNAKE [v2.0]

  // ── Daten-Konvertierung ───────────────────────────────────────────
  | 'csv_to_json'
  | 'json_to_csv'
  | 'json_to_yaml'           // [v2.0]
  | 'yaml_to_json'           // [v2.0]
  | 'table_to_markdown'      // Leerzeichen-ausgerichtete Tabelle → MD [v2.0]
  | 'align_columns'          // Spalten durch Padding ausrichten [v2.0]

  // ── Extraktion [v2.0] ──────────────────────────────────────────────
  | 'extract_emails'         // Alle E-Mail-Adressen extrahieren
  | 'extract_urls'           // Alle URLs extrahieren
  | 'extract_numbers'        // Alle Zahlen extrahieren
  | 'extract_markdown_headings' // # Überschriften extrahieren
  | 'extract_yaml_frontmatter'  // YAML-Frontmatter (zwischen ---) extrahieren
  | 'extract_json_values'    // Alle JSON-Werte (ohne Keys) extrahieren

  // ── Analyse & Statistik ───────────────────────────────────────────
  | 'estimate_tokens'        // Token-Schätzung anhängen (als Kommentar)
  | 'with_stats'             // Statistiken (Zeichen, Wörter, Zeilen) anhängen
  | 'with_full_stats'        // Erweiterte Statistiken (TextStats) anhängen [v2.0]
  | 'count_occurrences'      // Anzahl von Muster-Treffern zählen [v2.0]

  // ── Sicherheit (immer safety-critical = true) ─────────────────────
  | 'redact_sensitive'       // IPs, Tokens, API-Keys maskieren
  | 'strip_pii'              // PII vollständig entfernen

  // ── Template [v2.0] ───────────────────────────────────────────────
  | 'fill_template';         // {{variable}} mit params-Werten füllen
```

---

## § 5 — FILTER & QUERY [ERWEITERT — v2.0]

### 5.1 SnippetFilter [ERWEITERT — v2.0]

```typescript
interface SnippetFilter {
  // Volltext-Suche (FTS5 trigram index)
  readonly searchQuery:    Option<string>;

  // Tags
  readonly tags:           ReadonlyArray<TagName>;
  readonly tagsMode:       'all' | 'any';

  // Ort
  readonly locations:      ReadonlyArray<LocationFilter>;

  // Zeitraum
  readonly dateRange:      Option<DateRangeFilter>;
  readonly dateField:      'createdAt' | 'updatedAt';

  // Größe
  readonly sizeRange:      Option<SizeRangeFilter>;

  // Inhaltstyp
  readonly contentTypes:   ReadonlyArray<ContentType>;

  // Eigenschaften
  readonly isPinned:       Option<boolean>;
  readonly isFavorite:     Option<boolean>;   // [v2.0]
  readonly isTemplate:     Option<boolean>;   // [v2.0]
  readonly sourceApp:      Option<string>;    // [v2.0] Nach Quell-Anwendung filtern

  // Zählbereiche [v2.0]
  readonly wordCountRange: Option<NumberRange>;
  readonly lineCountRange: Option<NumberRange>;
  readonly usageCountMin:  Option<number>;

  // Sortierung
  readonly sortBy:         SortField;
  readonly sortDir:        'asc' | 'desc';
}

// [v2.0]
interface NumberRange {
  readonly min: Option<number>;
  readonly max: Option<number>;
}

type SortField =
  | 'title' | 'createdAt' | 'updatedAt' | 'size' | 'usageCount'
  | 'wordCount' | 'lineCount' | 'title_relevance';  // title_relevance: nur bei searchQuery

// Filter-Hilfsfunktionen [FP-Scala] reine Funktionen
const SnippetFilter = {
  default: (): SnippetFilter => ({
    searchQuery: Option.none(), tags: [], tagsMode: 'all',
    locations: [{ _type: 'all' }], dateRange: Option.none(), dateField: 'updatedAt',
    sizeRange: Option.none(), contentTypes: [], isPinned: Option.none(),
    isFavorite: Option.none(), isTemplate: Option.none(), sourceApp: Option.none(),
    wordCountRange: Option.none(), lineCountRange: Option.none(), usageCountMin: Option.none(),
    sortBy: 'updatedAt', sortDir: 'desc',
  }),
  merge: (base: SnippetFilter, patch: Partial<SnippetFilter>): SnippetFilter =>
    ({ ...base, ...patch }),
  // [v2.0] Schnellfilter-Fabriken
  onlyPinned:    (): Partial<SnippetFilter> => ({ isPinned: Option.some(true) }),
  onlyFavorites: (): Partial<SnippetFilter> => ({ isFavorite: Option.some(true) }),
  onlyTemplates: (): Partial<SnippetFilter> => ({ isTemplate: Option.some(true) }),
  today: (): Partial<SnippetFilter>         => ({
    dateRange: Option.some({ from: Option.some(startOfDayMs()), to: Option.none(), preset: Option.some('today') }),
    dateField: 'updatedAt',
  }),
} as const;
```

### 5.2 ClipboardFilter [NEU — v2.0]

```typescript
interface ClipboardFilter {
  readonly searchQuery:  Option<string>;
  readonly contentTypes: ReadonlyArray<ContentType>;
  readonly sourceApps:   ReadonlyArray<string>;
  readonly dateRange:    Option<DateRangeFilter>;
  readonly sizeRange:    Option<SizeRangeFilter>;
  readonly isPinned:     Option<boolean>;
  readonly promoted:     Option<boolean>;  // Wurde zu Snippet konvertiert
  readonly tags:         ReadonlyArray<TagName>;
  readonly sortBy:       'capturedAt' | 'size' | 'sourceApp';
  readonly sortDir:      'asc' | 'desc';
}
```

### 5.3 Gemeinsame Filter-Typen [v1.0-kompatibel]

```typescript
type LocationFilter =
  | { readonly _type: 'inbox' } | { readonly _type: 'archive' }
  | { readonly _type: 'folder'; readonly folderId: FolderId }
  | { readonly _type: 'all' };

interface DateRangeFilter {
  readonly from:   Option<UnixMs>;
  readonly to:     Option<UnixMs>;
  readonly preset: Option<DatePreset>;
}
type DatePreset = 'today' | 'last_7_days' | 'last_30_days' | 'this_month' | 'last_month' | 'custom';

interface SizeRangeFilter {
  readonly minBytes: Option<ByteSize>;
  readonly maxBytes: Option<ByteSize>;
  readonly preset:   Option<SizePreset>;
}
type SizePreset = 'tiny' | 'small' | 'medium' | 'large' | 'custom';

interface PagedResult<A> {
  readonly items:    ReadonlyArray<A>;
  readonly total:    number;
  readonly page:     number;
  readonly pageSize: number;
  readonly hasNext:  boolean;
  readonly hasPrev:  boolean;
}

interface SnippetListItem {
  readonly id:          SnippetId;
  readonly title:       string;
  readonly preview:     string;       // Erste 200 Zeichen
  readonly tags:        ReadonlyArray<TagName>;
  readonly location:    SnippetLocation;
  readonly contentType: ContentType;
  readonly size:        ByteSize;
  readonly lineCount:   number;
  readonly wordCount:   number;
  readonly createdAt:   UnixMs;
  readonly updatedAt:   UnixMs;
  readonly usageCount:  number;
  readonly isPinned:    boolean;
  readonly isFavorite:  boolean;    // [v2.0]
  readonly isTemplate:  boolean;    // [v2.0]
  readonly color:       Option<string>; // [v2.0]
  readonly matchScore:  Option<number>; // FTS5 rank bei Suche
}
```

---

## § 6 — TEMPLATE ENGINE [NEU — v2.0]

### 6.1 Überblick & Syntax-Spezifikation

```
Templates sind Snippets mit ContentType = 'template' und isTemplate = true.
Sie enthalten {{variable}}-Platzhalter die beim Rendern ersetzt werden.

SYNTAX-REGELN (autoritativ):
  {{variable}}              — Einfacher Platzhalter [Status: Implemented]
  {{variable:DefaultWert}}  — Optionaler Platzhalter mit Fallback [Status: Implemented]
  {{variable|filter}}       — Platzhalter mit eingebautem Filter [Status: Partially Implemented - Supported in JS domain, uncomplete in Rust]
  {{variable:Default|filter}} — Mit Fallback und Filter [Status: Partially Implemented - Supported in JS domain, uncomplete in Rust]
  {{ variable }}            — Führende/folgende Leerzeichen im Namen erlaubt (werden getrimmt) [Status: Implemented]

FILTER-OPERATOREN (eingebaut, ohne Sandbox): [Status: Partially Implemented - Implemented in JS domain, missing/unimplemented in Rust backend rendering]
  |upper       — GROSSBUCHSTABEN
  |lower       — kleinbuchstaben
  |title       — Jedes Wort Groß
  |trim        — trim()
  |slug        — URL-freundlich
  |snake       — snake_case
  |camel       — camelCase
  |json        — JSON.stringify()
  |base64      — Base64-Encode
  |url         — URL-Encode
  |truncate:N  — Auf N Zeichen kürzen (mit …)
  |lines       — Zeilenanzahl des Werts
  |words       — Wortanzahl des Werts
  |default:X   — Wenn leer → X (Alias für :X-Syntax)

SPEZIAL-VARIABLEN (immer verfügbar, kein Input nötig): [Status: Missing - Not resolved automatically in Rust or Svelte templates]
  {{_date}}        — Aktuelles Datum (ISO 8601: 2025-08-15)
  {{_time}}        — Aktuelle Zeit (HH:MM:SS)
  {{_datetime}}    — Datum + Zeit
  {{_timestamp}}   — Unix-Timestamp (ms)
  {{_uuid}}        — Neue UUID v4
  {{_clipboard}}   — Aktueller Clipboard-Inhalt
  {{_input}}       — Eingabetext (bei Pipeline-Step-Verwendung)
  {{_linecount}}   — Zeilenzahl von _input/_clipboard
  {{_wordcount}}   — Wortzahl von _input/_clipboard
  {{_charcount}}   — Zeichenzahl von _input/_clipboard

BEDINGTE BLÖCKE (v2.0): [Status: Partially Implemented - Basic regex parser on JS frontend, completely missing in Rust backend]
  {{#if variable}}...Inhalt...{{/if}}
  {{#if variable}}...{{#else}}...Fallback...{{/if}}
  {{#unless variable}}...{{/unless}}

SCHLEIFEN (v2.0, für Array-Variablen): [Status: Missing - Loops not implemented or processed in either Svelte or Rust template domains]
  {{#each items}}
    {{this}} — aktuelles Item
    {{@index}} — 0-basierter Index
    {{@first}} — Boolean, ob erstes Element
    {{@last}} — Boolean, ob letztes Element
  {{/each}}

VERBOTEN in Template-Variablennamen: [Status: Implemented]
  - Leerzeichen (außer am Rand, werden getrimmt)
  - Sonderzeichen außer . _ -
  - Beginnen mit Ziffer
  - Beginnen mit _ (reserviert für Spezial-Variablen)
```

### 6.2 Template-Typen

```typescript
interface TemplateVariable {
  readonly name:        string;           // Extrahiert aus {{name}}
  readonly hasDefault:  boolean;
  readonly defaultVal:  Option<string>;
  readonly filter:      Option<string>;   // z.B. 'upper', 'truncate:100'
  readonly isSpecial:   boolean;          // Beginnt mit _ → automatisch gesetzt
  readonly isRequired:  boolean;          // Kein Default, kein Special
  readonly occurrences: number;           // Wie oft kommt diese Variable vor
}

// Kontext für Template-Rendering
type TemplateContext = Readonly<Record<string, string | string[]>>;

interface TemplateRenderResult {
  readonly output:           string;
  readonly resolvedVariables: Readonly<Record<string, string>>;
  readonly unresolvedVars:   ReadonlyArray<string>;
  readonly warnings:         ReadonlyArray<string>;
}

interface ParsedTemplate {
  readonly variables:    ReadonlyArray<TemplateVariable>;
  readonly requiredVars: ReadonlyArray<string>;   // name, !hasDefault && !isSpecial
  readonly optionalVars: ReadonlyArray<string>;   // hasDefault || isSpecial
  readonly hasConditionals: boolean;
  readonly hasLoops:     boolean;
}
```

### 6.3 Template-Renderer (Reine Funktion)

```typescript
// [FP-Scala] Totale Funktion: ParsedTemplate + Context → Result
// Kein I/O, keine Seiteneffekte — vollständig testbar

const TemplateRenderer = {
  // Schritt 1: Template parsen (extrahiert alle Variablen)
  parse: (template: string): Result<DomainError, ParsedTemplate> => {
    const varRegex = /\{\{\s*(#\w+\s+)?([_a-zA-Z][\w.-]*)(:[^|{}]*)?((?:\|[\w:]+)*)\s*\}\}/g;
    // ... Parsing-Logik
    return Result.ok({
      variables:       extractVariables(template),
      requiredVars:    extractRequired(template),
      optionalVars:    extractOptional(template),
      hasConditionals: /\{\{#if\s/.test(template),
      hasLoops:        /\{\{#each\s/.test(template),
    });
  },

  // Schritt 2: Rendern — alle Variablen ersetzen
  render: (
    template: string,
    context:  TemplateContext,
    options:  { strict: boolean } = { strict: false }
  ): Result<DomainError, TemplateRenderResult> => {
    // Bei strict = true: TEMPLATE_MISSING_VARIABLE wenn Variable fehlt und kein Default
    // Bei strict = false: unresolved Variablen bleiben als {{variable}} erhalten
    // ...Rendering-Logik (rein, deterministisch)
    return Result.ok({ output: '...', resolvedVariables: {}, unresolvedVars: [], warnings: [] });
  },

  // Extrahiert alle einzigartigen Variablennamen (ohne Spezial-Variablen)
  extractVariableNames: (template: string): ReadonlyArray<string> => {
    const matches = template.matchAll(/\{\{\s*([_a-zA-Z][\w.-]*)(?::[^|{}]*)?(?:\|[\w:]+)*\s*\}\}/g);
    return [...new Set([...matches].map(m => m[1]).filter(n => !n.startsWith('_')))];
  },

  // Baut das Formular für eine ParsedTemplate auf
  buildForm: (parsed: ParsedTemplate): ReadonlyArray<FormField> => {
    // Erzeugt FormField[] für alle requiredVars + optionalVars
    return parsed.requiredVars.map(name => ({
      name, label: name.replace(/[_-]/g, ' '), required: true, defaultVal: Option.none(),
    }));
  },
} as const;

interface FormField {
  readonly name:       string;
  readonly label:      string;
  readonly required:   boolean;
  readonly defaultVal: Option<string>;
}
```

---

## § 7 — MARKDOWN-VORSCHAU [v1.0-kompatibel]

```typescript
interface MarkdownRenderOptions {
  readonly sanitize:        boolean;    // XSS-Schutz (immer true in Produktion)
  readonly highlightCode:   boolean;    // Syntax-Highlighting via Prism
  readonly mathSupport:     boolean;    // KaTeX für LaTeX
  readonly mermaidSupport:  boolean;    // Mermaid-Diagramme
  readonly linkTarget:      '_blank' | '_self';
  readonly maxHeadingLevel: 1 | 2 | 3 | 4 | 5 | 6;
  // [v2.0]
  readonly tableOfContents: boolean;    // Inhaltsverzeichnis automatisch generieren
  readonly lineNumbers:     boolean;    // Zeilennummern in Code-Blöcken
  readonly copyButton:      boolean;    // Kopieren-Button in Code-Blöcken
}

type PreviewMode = 'editor' | 'preview' | 'split';
```

---

## § 8 — CLIPBOARD-INTEGRATION [GEÄNDERT — v2.1] (KDE Plasma 6 / Wayland)

> **Voraussetzung:** `wl-clipboard` muss installiert sein: `sudo apt install wl-clipboard kde-cli-tools`
> Ohne `wl-paste` fällt der Monitor auf arboard-Polling (500ms) zurück — funktionsfähig, aber weniger reaktiv.

### 8.1 Architektur-Überblick [GEÄNDERT — v2.1]

```
KDE Plasma 6 läuft ausschließlich auf Wayland (kein X11-Fallback mehr).
ClipboardMonitor nutzt den nativen Wayland-Mechanismus via wl-clipboard.

PRIMÄR (Wayland / KDE): [Status: Implemented]
  wl-paste --watch                — subprocess, reagiert auf jede Clipboard-Änderung
  Kein Polling nötig — wl-paste blockiert bis zur nächsten Änderung (push-basiert)
  Crate: kein x11rb mehr. Subprocess via tokio::process::Command.

FALLBACK (falls wl-paste nicht verfügbar): [Status: Implemented]
  500ms adaptives Polling via arboard::Clipboard::get_text()
  Erkennbar: which wl-paste gibt leeren Exit-Code ≠ 0

QUELL-APP-ERKENNUNG (KDE/Wayland): [Status: Partially Implemented - KWin D-Bus calls via qdbus6 exist but are prone to D-Bus timeout/environment differences]
  Strategie 1: KWin D-Bus API  → org.kde.KWin / activeWindow → PID → /proc/PID/comm
  Strategie 2: qdbus6 org.kde.KWin /KWin activeWindow (subprocess)
  Strategie 3: xdg-foreign-portal (falls KWin-D-Bus nicht verfügbar)
  Fallback:    None (graceful — kein Panic)

INVARIANT: Der Monitor läuft in eigenem Thread. Niemals auf dem tokio-Runtime-Thread. [Status: Implemented]
INVARIANT: Deduplication via SHA-256 (identischer Hash → verwerfen) [Status: Implemented]
INVARIANT: Max 500 Einträge (LRU → älteste unpinned löschen) [Status: Partially Implemented - Trim logic lacks SQLite-level trigger enforcement on inserts]
INVARIANT: Mindestgröße 3 Zeichen — Einzelzeichen-Copies ignorieren [Status: Implemented]
INVARIANT: wl-paste subprocess wird bei App-Ende via child.kill() sauber beendet. [Status: Partially Implemented - tokio::spawn does not track task/child termination lifetime closely]
```

### 8.2 Rust-Signaturen (Effect Shell) [GEÄNDERT — v2.1]

```rust
// src-tauri/src/clipboard/mod.rs

pub struct ClipboardMonitorConfig {
    pub min_content_length: usize,      // Default: 3
    pub dedup_window_ms:    u64,        // Default: 500 — doppelter Copy innerhalb
    pub max_entries:        u32,        // Default: 500
}

/// Startet den Wayland-nativen Monitor (wl-paste --watch subprocess).
/// [PhilSD] Tiefes Modul: Caller ist plattformunabhängig
pub fn start_monitor(
    config: ClipboardMonitorConfig,
    app_handle: AppHandle,
) -> Result<(), MonitorError>;

// Implementierung (KDE Plasma 6 / Wayland):
// 1. Prüfe ob wl-paste verfügbar: which wl-paste → Ok/Err
// 2. Ja: tokio::process::Command::new("wl-paste").arg("--watch").arg("--no-newline")
//         → stdout zeilenweise lesen (jede neue Zeile = neuer Clipboard-Inhalt)
// 3. Nein (Fallback): 500ms tokio::time::interval + arboard::Clipboard::get_text()

// Jeder erkannte Clipboard-Wechsel:
// 1. Lese Inhalt (aus wl-paste stdout oder arboard)
// 2. Prüfe Mindestlänge (>= min_content_length)
// 3. SHA-256 berechnen, gegen last_hash prüfen (Dedup)
// 4. detect_source_app() aufrufen (KWin D-Bus → PID → /proc/PID/comm)
// 5. ClipboardEntry bauen und via tokio::mpsc senden
// 6. Backend emittiert Tauri-Event: "clipboard:new_entry"

pub enum MonitorError {
    WlPasteNotFound,                    // wl-paste nicht installiert
    WaylandSubprocessFailed(String),    // wl-paste subprocess Fehler
    FallbackPollingError(String),       // arboard-Fehler im Polling-Modus
}
```

### 8.3 Quell-App-Erkennung (KDE / Wayland) [GEÄNDERT — v2.1]

```rust
// src-tauri/src/clipboard/source_app.rs

pub fn detect_source_app() -> Option<String> {
    // Strategie 1: KWin D-Bus (KDE Plasma 6 — bevorzugt, kein subprocess nötig)
    // org.kde.KWin → /KWin → activeWindow() → XdgToplevel-Handle → PID
    // PID → /proc/{pid}/comm für App-Namen
    if let Some(name) = try_kwin_dbus_active_window() { return Some(name); }

    // Strategie 2: qdbus6 subprocess (KWin D-Bus ohne native Bindings)
    // Command: qdbus6 org.kde.KWin /KWin activeWindow
    // → liefert Wayland-Surface-ID → weitere qdbus6-Abfrage für PID
    if let Some(name) = try_qdbus6_kwin()             { return Some(name); }

    // Strategie 3: /proc/self/loginuid + procfs-Scan (Fallback, langsamer)
    if let Some(name) = try_procfs_active()            { return Some(name); }

    // Alle Strategien graceful — None bei Fehler, kein Panic
    None
}
```

### 8.4 Clipboard Write-Back

```typescript
// Frontend: Clipboard-Inhalt schreiben
// Tauri IPC → arboard::Clipboard::set_text()  (arboard unterstützt Wayland nativ via wl-clipboard)
// [INVARIANT] Nur der Nutzer schreibt explizit — kein automatisches Write-Back

// Nach erfolgreichem Write:
// 1. Eintrag in usage_history (action: 'copy')
// 2. snippet.usageCount++ via update_snippet IPC
// 3. UI zeigt kurze Bestätigung ("Kopiert!" Toast, 1,5s)
```

---

## § 9 — UNDO / REDO SYSTEM [NEU — v2.0] [Status: Partially Implemented - Rust backend uses custom basic UndoStack struct with minimal single-action tracking; missing folders, bulk operations, scripts and pipelines actions]

### 9.1 Undo-Action (Discriminated Union)

```typescript
// [FP-Scala] Exhaustive pattern matching — alle Aktionen sind explizit
type UndoAction =
  // Snippet-Aktionen
  | { readonly _type: 'snippet_update';     readonly before: Snippet;   readonly after: Snippet }
  | { readonly _type: 'snippet_create';     readonly created: Snippet }
  | { readonly _type: 'snippet_delete';     readonly deleted: Snippet }
  | { readonly _type: 'snippet_move';       readonly id: SnippetId; readonly from: SnippetLocation; readonly to: SnippetLocation }

  // Script-Aktionen
  | { readonly _type: 'script_update';      readonly before: Script;    readonly after: Script }
  | { readonly _type: 'script_create';      readonly created: Script }
  | { readonly _type: 'script_delete';      readonly deleted: Script }

  // Pipeline-Aktionen
  | { readonly _type: 'pipeline_update';    readonly before: Pipeline;  readonly after: Pipeline }

  // Transform-Aktionen (reversibel via Undo des Original-Texts)
  | { readonly _type: 'transform_apply';    readonly snippetId: SnippetId;
      readonly originalContent: string;   readonly transformedContent: string;
      readonly pipelineId: Option<PipelineId>; readonly scriptId: Option<ScriptId> }

  // Bulk-Aktionen [NEU v2.0]
  | { readonly _type: 'bulk_operation';     readonly operations: ReadonlyArray<UndoAction> }

  // Ordner
  | { readonly _type: 'folder_create';      readonly created: Folder }
  | { readonly _type: 'folder_rename';      readonly id: FolderId; readonly from: string; readonly to: string }
  | { readonly _type: 'folder_delete';      readonly deleted: Folder; readonly movedSnippets: ReadonlyArray<SnippetId> };

// Metadaten für jede Undo-Action
interface UndoEntry {
  readonly action:      UndoAction;
  readonly performedAt: UnixMs;
  readonly description: string;  // Menschenlesbar: "Snippet 'Mein Text' bearbeitet"
}
```

### 9.2 Undo-Stack (In-Memory)

```typescript
// [FP-Scala] Unveränderliche Zustandsmaschine — Stack ist ein Wert
interface UndoStack {
  readonly undoable: ReadonlyArray<UndoEntry>;  // Neueste vorne
  readonly redoable: ReadonlyArray<UndoEntry>;  // Neueste vorne
  readonly maxSize:  number;                    // Default: 50
}

const UndoStack = {
  empty:   (maxSize = 50): UndoStack => ({ undoable: [], redoable: [], maxSize }),

  // Neue Aktion hinzufügen — löscht Redo-Stack (Branch-Point)
  push: (stack: UndoStack, entry: UndoEntry): UndoStack => ({
    undoable: [entry, ...stack.undoable].slice(0, stack.maxSize),
    redoable: [],  // Neue Aktion → kein Redo mehr möglich
    maxSize:  stack.maxSize,
  }),

  undo: (stack: UndoStack): Result<DomainError, { entry: UndoEntry; newStack: UndoStack }> => {
    if (stack.undoable.length === 0) return Result.err({ code: 'UNDO_STACK_EMPTY' });
    const [entry, ...rest] = stack.undoable;
    return Result.ok({
      entry,
      newStack: { undoable: rest, redoable: [entry, ...stack.redoable], maxSize: stack.maxSize },
    });
  },

  redo: (stack: UndoStack): Result<DomainError, { entry: UndoEntry; newStack: UndoStack }> => {
    if (stack.redoable.length === 0) return Result.err({ code: 'REDO_STACK_EMPTY' });
    const [entry, ...rest] = stack.redoable;
    return Result.ok({
      entry,
      newStack: { undoable: [entry, ...stack.undoable], redoable: rest, maxSize: stack.maxSize },
    });
  },

  canUndo: (stack: UndoStack): boolean => stack.undoable.length > 0,
  canRedo: (stack: UndoStack): boolean => stack.redoable.length > 0,
} as const;

// IMPLEMENTIERUNGS-HINWEIS:
// UndoStack lebt im Svelte-Store (in-memory).
// Bei App-Neustart: Undo-Stack ist leer (kein Persistieren des Stacks).
// usage_history (SQLite) bleibt erhalten — für langfristige Nachvollziehbarkeit.
// Undo-Operationen schreiben erneut die DB (undo = inverse operation).
```

---

## § 10 — IMPORT / EXPORT [NEU — v2.0]

### 10.1 TextForge Bundle Format (.tfbundle) [Status: Partially Implemented - Bundles only process Snippets, manifest lacks counts for scripts/pipelines/folders and checksum verification]

```
Ein .tfbundle ist eine ZIP-Datei mit festgelegter Struktur:

textforge-bundle/
├── manifest.json          — Metadaten + Checksummen
├── snippets/
│   ├── {uuid}.json        — Ein JSON-File pro Snippet (SnippetDto)
│   └── ...
├── scripts/
│   ├── {uuid}.json        — Ein JSON-File pro Script
│   └── ...
├── pipelines/
│   ├── {uuid}.json        — Ein JSON-File pro Pipeline
│   └── ...
└── folders/
    └── folders.json       — Alle Ordner (FolderDto[])

manifest.json:
{
  "bundleVersion": "1.0",
  "appVersion":    "2.0.0",
  "bundleId":      "<UUID>",
  "createdAt":     1234567890000,
  "platform":      "KDE Plasma 6 / Wayland",
  "counts": { "snippets": 42, "scripts": 8, "pipelines": 3, "folders": 5 },
  "checksums": {
    "snippets/{uuid}.json": "<sha256>",
    ...
  }
}
```

### 10.2 Export-Typen [Status: Partially Implemented - Only tfbundle and raw json are supported via UI/IPC. Individual ExportRequest/ExportResult maps are incomplete]

```typescript
type ExportFormat =
  | 'bundle'         // .tfbundle — vollständiger Import/Export
  | 'markdown'       // .md — Snippet-Inhalt (bei Markdown-Content)
  | 'text'           // .txt — Snippet-Inhalt als Plain Text
  | 'json'           // .json — Einzelnes Snippet als JSON
  | 'json_array'     // .json — Mehrere Snippets als JSON-Array
  | 'csv';           // .csv — Snippets-Metadaten als Tabelle

interface ExportRequest {
  readonly format:     ExportFormat;
  readonly snippetIds: Option<ReadonlyArray<SnippetId>>;  // None = alle
  readonly scriptIds:  Option<ReadonlyArray<ScriptId>>;
  readonly pipelineIds: Option<ReadonlyArray<PipelineId>>;
  readonly includeFolders: boolean;
  readonly outputPath: FilePath;    // Vom Nutzer gewählt (Tauri file dialog)
}

interface ExportResult {
  readonly path:           FilePath;
  readonly bytesWritten:   ByteSize;
  readonly exportedCounts: { snippets: number; scripts: number; pipelines: number };
  readonly format:         ExportFormat;
  readonly completedAt:    UnixMs;
}
```

### 10.3 Import-Typen [Status: Partially Implemented - Overwrite conflict policy implemented, skip/rename policies are missing. Errors are returned as strings rather than DomainError types]

```typescript
interface ImportRequest {
  readonly sourcePath:    FilePath;
  readonly format:        'bundle' | 'json' | 'json_array' | 'text' | 'markdown';
  readonly conflictPolicy: ImportConflictPolicy;
  readonly targetLocation: SnippetLocation;  // Wo neue Snippets landen (default: inbox)
}

type ImportConflictPolicy =
  | 'skip'      // Existierendes beibehalten, neues überspringen
  | 'overwrite' // Existierendes überschreiben
  | 'rename'    // Neues umbenennen (" (Import)" anhängen)
  | 'ask';      // UI fragt für jeden Konflikt

interface ImportResult {
  readonly imported:  { snippets: number; scripts: number; pipelines: number; folders: number };
  readonly skipped:   number;
  readonly conflicts: ReadonlyArray<ImportConflict>;
  readonly errors:    ReadonlyArray<{ item: string; error: DomainError }>;
  readonly completedAt: UnixMs;
}

interface ImportConflict {
  readonly existingId:   string;
  readonly existingTitle: string;
  readonly importedTitle: string;
  readonly resolution:   'skipped' | 'overwritten' | 'renamed';
}
```

---

## § 11 — KEYBOARD SHORTCUTS [NEU — v2.0] [Status: Missing - Frontend shortcut map and registry not implemented in SvelteKit client]

### 11.1 Vollständige Shortcut-Map

```typescript
// [CleanCode] Alle Shortcuts sind an einem Ort definiert — kein Verstreuen in Components
// [PragProg] DRY: ShortcutMap ist die EINZIGE Quelle der Wahrheit für Tastenkombinationen

type ModifierKey = 'Ctrl' | 'Shift' | 'Alt' | 'Meta';
type ShortcutContext = 'global' | 'snippet_list' | 'snippet_editor' | 'script_editor'
                     | 'pipeline_editor' | 'clipboard' | 'search';

interface Shortcut {
  readonly key:        string;              // Taste (z.B. 'n', 'Enter', 'F1')
  readonly modifiers:  ReadonlyArray<ModifierKey>;
  readonly context:    ShortcutContext;
  readonly action:     string;              // Identifier für die Aktion
  readonly description: string;
  readonly customizable: boolean;
}

// GLOBALE SHORTCUTS (immer aktiv)
const GLOBAL_SHORTCUTS: ReadonlyArray<Shortcut> = [
  { key: 'n',      modifiers: ['Ctrl'],        context: 'global', action: 'snippet:new',            description: 'Neues Snippet erstellen',          customizable: true  },
  { key: 'f',      modifiers: ['Ctrl'],        context: 'global', action: 'search:focus',           description: 'Suche fokussieren',                customizable: true  },
  { key: 'z',      modifiers: ['Ctrl'],        context: 'global', action: 'undo',                   description: 'Rückgängig',                       customizable: false },
  { key: 'z',      modifiers: ['Ctrl','Shift'],context: 'global', action: 'redo',                   description: 'Wiederholen',                      customizable: false },
  { key: 's',      modifiers: ['Ctrl'],        context: 'global', action: 'save',                   description: 'Speichern',                        customizable: false },
  { key: ',',      modifiers: ['Ctrl'],        context: 'global', action: 'settings:open',          description: 'Einstellungen öffnen',             customizable: false },
  { key: '1',      modifiers: ['Ctrl'],        context: 'global', action: 'view:snippets',          description: 'Snippets-Ansicht',                 customizable: true  },
  { key: '2',      modifiers: ['Ctrl'],        context: 'global', action: 'view:scripts',           description: 'Skripte-Ansicht',                  customizable: true  },
  { key: '3',      modifiers: ['Ctrl'],        context: 'global', action: 'view:pipelines',         description: 'Pipelines-Ansicht',                customizable: true  },
  { key: '4',      modifiers: ['Ctrl'],        context: 'global', action: 'view:clipboard',         description: 'Zwischenablage-Ansicht',           customizable: true  },
  { key: 'p',      modifiers: ['Ctrl','Shift'],context: 'global', action: 'command_palette:open',  description: 'Command Palette öffnen',           customizable: true  },
];

// SNIPPET-LISTE
const SNIPPET_LIST_SHORTCUTS: ReadonlyArray<Shortcut> = [
  { key: 'Enter',  modifiers: [],             context: 'snippet_list', action: 'snippet:open',          description: 'Snippet öffnen',                   customizable: false },
  { key: 'c',      modifiers: ['Ctrl'],        context: 'snippet_list', action: 'snippet:copy_to_clip',  description: 'In Zwischenablage kopieren',        customizable: true  },
  { key: 'd',      modifiers: ['Ctrl'],        context: 'snippet_list', action: 'snippet:duplicate',     description: 'Snippet duplizieren',              customizable: true  },
  { key: 'Delete', modifiers: [],             context: 'snippet_list', action: 'snippet:trash',          description: 'In Papierkorb verschieben',         customizable: false },
  { key: 'p',      modifiers: ['Ctrl'],        context: 'snippet_list', action: 'snippet:toggle_pin',    description: 'Pin umschalten',                   customizable: true  },
  { key: 'ArrowUp',modifiers: [],             context: 'snippet_list', action: 'list:prev',              description: 'Vorheriges Element',               customizable: false },
  { key: 'ArrowDown',modifiers:[],            context: 'snippet_list', action: 'list:next',              description: 'Nächstes Element',                 customizable: false },
  { key: 't',      modifiers: ['Ctrl'],        context: 'snippet_list', action: 'snippet:transform',     description: 'Transformation anwenden',          customizable: true  },
];

// SNIPPET-EDITOR (Monaco)
const SNIPPET_EDITOR_SHORTCUTS: ReadonlyArray<Shortcut> = [
  { key: 'm',      modifiers: ['Ctrl','Shift'],context: 'snippet_editor', action: 'preview:toggle',       description: 'Vorschau umschalten',              customizable: true  },
  { key: 'c',      modifiers: ['Ctrl','Shift'],context: 'snippet_editor', action: 'snippet:copy_content', description: 'Gesamten Inhalt kopieren',          customizable: true  },
  { key: 'r',      modifiers: ['Ctrl','Shift'],context: 'snippet_editor', action: 'stats:show',           description: 'Text-Statistiken anzeigen',        customizable: true  },
  { key: 'e',      modifiers: ['Ctrl','Shift'],context: 'snippet_editor', action: 'transform:quick',      description: 'Schnell-Transformation',           customizable: true  },
  { key: 'Escape', modifiers: [],             context: 'snippet_editor', action: 'editor:blur',           description: 'Editor verlassen',                 customizable: false },
];

// SKRIPT-EDITOR
const SCRIPT_EDITOR_SHORTCUTS: ReadonlyArray<Shortcut> = [
  { key: 'Enter', modifiers: ['Ctrl'],         context: 'script_editor', action: 'script:run',            description: 'Skript ausführen (Live-Test)',      customizable: false },
  { key: 'Enter', modifiers: ['Ctrl','Shift'], context: 'script_editor', action: 'script:run_tests',      description: 'Alle Tests ausführen',             customizable: false },
  { key: 'F1',    modifiers: [],              context: 'script_editor', action: 'script:show_docs',      description: 'Dokumentation/utils.* anzeigen',   customizable: false },
];

// Benutzerdefinierte Shortcuts (persistiert in settings)
interface CustomShortcut {
  readonly action:    string;
  readonly key:       string;
  readonly modifiers: ReadonlyArray<ModifierKey>;
}
```

---

## § 12 — TEXT-ANALYSE & STATISTIKEN [NEU — v2.0]

*(Typen in § 2.6 definiert. Dieser Abschnitt spezifiziert die Darstellung.)*

### 12.1 Darstellungs-Spezifikation

```
TextStats wird angezeigt in:
  A) Stats-Panel (Sidebar, immer sichtbar wenn aktiviert)
  B) Tooltip beim Hover über Metadaten-Chips
  C) Builtin 'with_full_stats' — als Kommentar an Text angehängt

Format für 'with_full_stats' Builtin-Ausgabe:
─────────────────────────────────────────────
{original_content}

---
📊 TextForge Statistics  2025-01-15 14:30:22
  Characters:  1,234 (987 without spaces)
  Words:       201 (168 unique)
  Lines:       45 (3 empty)
  Paragraphs:  8
  Sentences:   22
  Avg word:    4.9 chars   Longest: "interoperability"
  Avg line:    27.4 chars  Longest line: 89 chars
  ~Tokens:     310 (cl100k_base estimate)
  Reading:     ~1 min
  FK Grade:    8.2
  Top words:   "the" (12), "function" (8), "type" (7)
─────────────────────────────────────────────
```

### 12.2 Token-Schätzungs-Modelle

```typescript
type TokenizerModel = 'cl100k' | 'p50k' | 'simple';

const estimateTokens = (content: string, model: TokenizerModel = 'cl100k'): TokenCount => {
  // [FP-Scala] Totale reine Funktion — kein externer Tokenizer nötig
  switch (model) {
    // cl100k_base (GPT-4, Claude approximation):
    // Englisch: ~4 Zeichen / Token
    // Code: ~3 Zeichen / Token
    // Formel: max(charCount/4, wordCount*0.75)
    case 'cl100k': {
      const chars  = content.length;
      const words  = content.trim().split(/\s+/).length;
      return Math.round(Math.max(chars / 4, words * 0.75)) as TokenCount;
    }
    // p50k (GPT-3): ~5 Zeichen / Token
    case 'p50k':   return Math.round(content.length / 5) as TokenCount;
    // Simple: Wörter = Tokens (Baseline)
    case 'simple': return content.trim().split(/\s+/).length as TokenCount;
  }
};
```

---

## § 13 — BULK-OPERATIONEN [NEU — v2.0]

### 13.1 Bulk-Operation-Typen

```typescript
// [PhilSD] Tiefes Modul: Alle Bulk-Ops haben einheitliches Interface
type BulkOperation =
  | {
      readonly _type:       'bulk_transform';
      readonly snippetIds:  NonEmptyArray<SnippetId>;
      readonly pipelineId:  PipelineId;
      readonly saveResults: boolean;   // true: Snippet-Inhalt ersetzen; false: nur Vorschau
    }
  | {
      readonly _type:      'bulk_tag';
      readonly snippetIds: NonEmptyArray<SnippetId>;
      readonly addTags:    ReadonlyArray<TagName>;
      readonly removeTags: ReadonlyArray<TagName>;
    }
  | {
      readonly _type:      'bulk_move';
      readonly snippetIds: NonEmptyArray<SnippetId>;
      readonly targetLocation: SnippetLocation;
    }
  | {
      readonly _type:      'bulk_delete';
      readonly snippetIds: NonEmptyArray<SnippetId>;
      readonly permanent:  boolean;   // false = Papierkorb, true = permanent (nur aus Papierkorb)
    }
  | {
      readonly _type:        'bulk_export';
      readonly snippetIds:   NonEmptyArray<SnippetId>;
      readonly format:       ExportFormat;
      readonly outputPath:   FilePath;
    }
  | {
      readonly _type:       'bulk_pin';
      readonly snippetIds:  NonEmptyArray<SnippetId>;
      readonly pinned:      boolean;
    }
  | {
      readonly _type:       'bulk_favorite';
      readonly snippetIds:  NonEmptyArray<SnippetId>;
      readonly favorite:    boolean;
    };

interface BulkOperationResult {
  readonly operation:   BulkOperation;
  readonly succeeded:   ReadonlyArray<SnippetId>;
  readonly failed:      ReadonlyArray<{ id: SnippetId; error: DomainError }>;
  readonly totalCount:  number;
  readonly durationMs:  number;
  // Für bulk_transform mit saveResults = false:
  readonly previews:    Option<ReadonlyArray<{ id: SnippetId; preview: string }>>;
}

// Invariante: Bulk-Operationen sind atomar auf Einzel-Ebene.
// Eine fehlgeschlagene Einzel-Operation stoppt NICHT die restlichen.
// Das Gesamtergebnis enthält succeeded + failed für Transparenz.
// Bulk-Operationen erzeugen EINEN UndoEntry (type: 'bulk_operation').
```

---

## § 14 — DIFF / VERGLEICH [NEU — v2.0]

```typescript
// Backend: similar::TextDiff (Rust crate) — Myers-Algorithmus
// Frontend: Darstellung in DiffViewer.svelte-Komponente

// Verwendung:
// A) Transform-Preview: original vs. transformierter Text (nebeneinander)
// B) Script-Test: expected vs. actual
// C) Snippet-Versions-Vergleich (via ScriptVersion)
// D) Before/After in Pipeline-Schritt-Ergebnis

// IPC-Aufruf: compute_diff(original: string, modified: string) → DiffResultDto

interface DiffResultDto {
  lines:        DiffLineDto[];
  addedLines:   number;
  deletedLines: number;
  unchanged:    number;
  similarity:   number;      // 0.0 – 1.0
}

interface DiffLineDto {
  kind:        'equal' | 'insert' | 'delete';
  oldLineNum:  number | null;
  newLineNum:  number | null;
  content:     string;
}

// Darstellungs-Modi:
// 'unified'  — Unified-Diff (wie git diff)
// 'split'    — Nebeneinander (wie GitHub-Review)
// 'inline'   — Inline-Hervorhebung (Zeichenebene)
type DiffViewMode = 'unified' | 'split' | 'inline';
```

---

## § 15 — NOTIFICATION SYSTEM [NEU — v2.0]

```typescript
// [CleanCode] Single Responsibility: Notifications sind eigenständiger Concern
// Keine Snackbars/Toasts direkt in Business-Logik-Komponenten

type NotificationSeverity = 'success' | 'info' | 'warning' | 'error';

interface AppNotification {
  readonly id:          string;         // UUID
  readonly severity:    NotificationSeverity;
  readonly title:       string;
  readonly message:     Option<string>;
  readonly duration:    number;         // ms (0 = persistent bis Dismiss)
  readonly action:      Option<NotificationAction>;
  readonly createdAt:   UnixMs;
}

interface NotificationAction {
  readonly label:   string;
  readonly handler: string;   // Action-Identifier (z.B. 'undo', 'open_snippet:{id}')
}

// Vordefinierte Notifications (DRY — eine Quelle)
const Notifications = {
  snippetSaved:      (title: string): AppNotification => ({ id: uuid(), severity: 'success', title: 'Gespeichert', message: Option.some(`"${title}" wurde gespeichert`),    duration: 1500, action: Option.none(), createdAt: now() }),
  snippetCopied:     ():              AppNotification => ({ id: uuid(), severity: 'success', title: 'Kopiert',     message: Option.none(),                                      duration: 1200, action: Option.none(), createdAt: now() }),
  transformComplete: (ms: number):    AppNotification => ({ id: uuid(), severity: 'success', title: 'Transformation abgeschlossen', message: Option.some(`In ${ms}ms`),        duration: 2000, action: Option.none(), createdAt: now() }),
  transformError:    (e: DomainError):AppNotification => ({ id: uuid(), severity: 'error',   title: 'Fehler',      message: Option.some(DomainError.describe(e)),               duration: 5000, action: Option.none(), createdAt: now() }),
  undoAvailable:     (desc: string):  AppNotification => ({ id: uuid(), severity: 'info',    title: 'Rückgängig möglich', message: Option.some(desc),                          duration: 3000, action: Option.some({ label: 'Rückgängig', handler: 'undo' }), createdAt: now() }),
  importComplete:    (r: ImportResult): AppNotification => ({ id: uuid(), severity: 'success', title: 'Import abgeschlossen', message: Option.some(`${r.imported.snippets} Snippets importiert`), duration: 3000, action: Option.none(), createdAt: now() }),
} as const;
```

---

## § 16 — WORKSPACE SESSION [NEU — v2.0]

```typescript
// WorkspaceSession-Persistenz-Strategie:
// 1. Session wird in settings.key = 'session.workspace' als JSON gespeichert
// 2. Beim App-Start: Session laden, letzte View + Filter wiederherstellen
// 3. Beim App-Ende (on_window_close): Session schreiben

// Session wird bei jeder signifikanten UI-Interaktion debounced (2s) gespeichert:
// - Tab-Wechsel
// - Filter-Änderung
// - Snippet/Script-Öffnen
// - Sidebar-Resize
// - PreviewMode-Wechsel

// Session darf NIE personenbezogene Daten enthalten (nur IDs, keine Inhalte)

const WorkspaceSession = {
  default: (): WorkspaceSession => ({
    activeView:            'snippets',
    lastActiveSnippetId:   Option.none(),
    lastActiveScriptId:    Option.none(),
    lastActivePipelineId:  Option.none(),
    sidebarWidth:          280,
    previewMode:           'split',
    filterState:           SnippetFilter.default(),
    openEditorTabs:        [],
    savedAt:               Date.now() as UnixMs,
  }),

  toSettings: (session: WorkspaceSession): string => JSON.stringify(session),
  fromSettings: (json: string): Result<DomainError, WorkspaceSession> =>
    Result.tryCatch(() => JSON.parse(json), () => ({ code: 'STORAGE_ERROR', details: 'Session Parse Error' })),
} as const;
```

---

## § 17 — SQLITE SCHEMA v2.0

```sql
-- migrations/001_initial.sql (identisch mit v1.0 — nicht ändern)
-- migrations/002_v2_extensions.sql — NUR NEUE TABELLEN UND SPALTEN

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ── ERWEITERUNGEN AN BESTEHENDEN TABELLEN ──────────────────────────────────────
-- [INVARIANT-F] Keine ALTER TABLE auf bestehende Spalten — nur neue hinzufügen

ALTER TABLE snippets ADD COLUMN source_app   TEXT;
ALTER TABLE snippets ADD COLUMN is_template  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE snippets ADD COLUMN is_favorite  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE snippets ADD COLUMN color        TEXT CHECK(color GLOB '#??????' OR color IS NULL);

ALTER TABLE scripts  ADD COLUMN is_favorite  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE scripts  ADD COLUMN current_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE scripts  ADD COLUMN color        TEXT CHECK(color GLOB '#??????' OR color IS NULL);

ALTER TABLE pipelines ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
ALTER TABLE pipelines ADD COLUMN is_template INTEGER NOT NULL DEFAULT 0;

-- FTS-Trigger aktualisieren (is_template in FTS-Felder aufnehmen):
DROP TRIGGER IF EXISTS snip_fts_update;
CREATE TRIGGER snip_fts_update AFTER UPDATE ON snippets
  BEGIN
    DELETE FROM snippets_fts WHERE rowid = old.rowid;
    INSERT INTO snippets_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
  END;

-- ── NEUE TABELLEN ──────────────────────────────────────────────────────────────

-- Zwischenablage-Verlauf
CREATE TABLE IF NOT EXISTS clipboard_history (
  id               TEXT PRIMARY KEY,
  content          TEXT NOT NULL,
  content_hash     TEXT NOT NULL UNIQUE,           -- SHA-256 für Dedup, UNIQUE
  content_type     TEXT NOT NULL DEFAULT 'plain_text',
  source_app       TEXT,
  captured_at      INTEGER NOT NULL,
  size_bytes       INTEGER GENERATED ALWAYS AS (length(content)) VIRTUAL,
  line_count       INTEGER GENERATED ALWAYS AS (
                     length(content) - length(replace(content, char(10), '')) + 1
                   ) VIRTUAL,
  is_pinned        INTEGER NOT NULL DEFAULT 0,
  promoted_to_snippet_id TEXT REFERENCES snippets(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_clip_captured  ON clipboard_history(captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_clip_pinned    ON clipboard_history(is_pinned, captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_clip_type      ON clipboard_history(content_type);
CREATE INDEX IF NOT EXISTS idx_clip_source    ON clipboard_history(source_app);

-- Tags für Clipboard-Einträge
CREATE TABLE IF NOT EXISTS clipboard_tags (
  entry_id TEXT NOT NULL REFERENCES clipboard_history(id) ON DELETE CASCADE,
  tag      TEXT NOT NULL,
  PRIMARY KEY (entry_id, tag)
);

-- FTS für Clipboard
CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(
  content,
  content='clipboard_history',
  content_rowid='rowid',
  tokenize='trigram'
);
CREATE TRIGGER IF NOT EXISTS clip_fts_insert AFTER INSERT ON clipboard_history
  BEGIN INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content); END;
CREATE TRIGGER IF NOT EXISTS clip_fts_update AFTER UPDATE ON clipboard_history
  BEGIN
    DELETE FROM clipboard_fts WHERE rowid = old.rowid;
    INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content);
  END;
CREATE TRIGGER IF NOT EXISTS clip_fts_delete AFTER DELETE ON clipboard_history
  BEGIN DELETE FROM clipboard_fts WHERE rowid = old.rowid; END;

-- Skript-Versionen
CREATE TABLE IF NOT EXISTS script_versions (
  id                  TEXT PRIMARY KEY,
  script_id           TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
  version             INTEGER NOT NULL,
  js_code             TEXT,
  regex_pattern       TEXT,
  regex_replacement   TEXT,
  regex_flags         TEXT NOT NULL DEFAULT 'g',
  parameters_json     TEXT NOT NULL DEFAULT '[]',
  change_note         TEXT,
  saved_at            INTEGER NOT NULL,
  UNIQUE(script_id, version)  -- Pro Skript eindeutige Versionen
);

CREATE INDEX IF NOT EXISTS idx_scrver_script ON script_versions(script_id, version DESC);

-- Trigger: Max 20 Versionen pro Skript (automatisches FIFO)
CREATE TRIGGER IF NOT EXISTS limit_script_versions
AFTER INSERT ON script_versions
BEGIN
  DELETE FROM script_versions
  WHERE script_id = NEW.script_id
    AND id NOT IN (
      SELECT id FROM script_versions
      WHERE script_id = NEW.script_id
      ORDER BY version DESC
      LIMIT 20
    );
END;

-- Template-Variablen (gecachte Analyse für Schnell-Zugriff)
CREATE TABLE IF NOT EXISTS template_variables (
  snippet_id   TEXT NOT NULL REFERENCES snippets(id) ON DELETE CASCADE,
  variable     TEXT NOT NULL,
  has_default  INTEGER NOT NULL DEFAULT 0,
  default_val  TEXT,
  is_required  INTEGER NOT NULL DEFAULT 1,
  occurrences  INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (snippet_id, variable)
);

CREATE INDEX IF NOT EXISTS idx_tmpl_var ON template_variables(snippet_id);

-- Trigger: template_variables aktuell halten
-- (Wird vom Rust-Backend nach jedem Snippet-Update aufgerufen — kein DB-Trigger
--  da Regex-Extraktion in Rust effizienter ist)

-- Erweiterte Einstellungen (migrations/002 fügt neue Defaults hinzu)
INSERT OR IGNORE INTO settings VALUES
  -- Clipboard
  ('clipboard.enabled',            'true',  unixepoch() * 1000),
  ('clipboard.max_entries',        '500',   unixepoch() * 1000),
  ('clipboard.min_length',         '3',     unixepoch() * 1000),
  ('clipboard.dedup_window_ms',    '500',   unixepoch() * 1000),

  -- Undo
  ('undo.max_stack_size',          '50',    unixepoch() * 1000),

  -- Export
  ('export.default_format',        'bundle',unixepoch() * 1000),
  ('export.include_metadata',      'true',  unixepoch() * 1000),

  -- UI (v2.0)
  ('ui.show_stats_panel',          'true',  unixepoch() * 1000),
  ('ui.diff_mode',                 'split', unixepoch() * 1000),
  ('ui.shortcut_hints',            'true',  unixepoch() * 1000),
  ('ui.confirm_bulk_delete',       'true',  unixepoch() * 1000),

  -- Template
  ('template.strict_mode',         'false', unixepoch() * 1000),

  -- Script-Versionen
  ('script.max_versions',          '20',    unixepoch() * 1000),

  -- Session
  ('session.restore_on_start',     'true',  unixepoch() * 1000);
```

---

## § 18 — TAURI IPC COMMANDS v2.0

### 18.1 Snippet-Commands [v1.0-kompatibel + Erweiterungen]

```rust
// ── ALLE v1.0-Commands bleiben unverändert ───────────────────────────────────
// list_snippets, get_snippet, list_all_tags, list_folders
// create_snippet, update_snippet, duplicate_snippet
// trash_snippet, restore_snippet, delete_snippet_permanently, empty_trash
// create_folder, rename_folder, delete_folder

// ── NEUE v2.0-Commands ───────────────────────────────────────────────────────

/// Gibt TextStats für einen Text zurück (reine Berechnung — keine DB)
#[tauri::command]
pub async fn compute_text_stats(
    content: String,
) -> Result<TextStatsDto, String>;

/// Parst ein Template und gibt alle Variablen zurück
#[tauri::command]
pub async fn parse_template(
    content: String,
) -> Result<ParsedTemplateDto, String>;

/// Rendert ein Template mit gegebenem Kontext
#[tauri::command]
pub async fn render_template(
    content:    String,
    context:    HashMap<String, String>,
    strict:     bool,
) -> Result<TemplateRenderResultDto, String>;

/// Berechnet Diff zwischen zwei Texten
#[tauri::command]
pub async fn compute_diff(
    original: String,
    modified: String,
) -> Result<DiffResultDto, String>;

/// Führt eine Bulk-Operation aus
#[tauri::command]
pub async fn execute_bulk_operation(
    operation: BulkOperationDto,
    state:     State<'_, AppState>,
) -> Result<BulkOperationResultDto, String>;

/// Undo — macht die letzte Aktion rückgängig
#[tauri::command]
pub async fn undo(
    state: State<'_, AppState>,
) -> Result<UndoEntryDto, String>;

/// Redo — wiederholt die letzte rückgängig gemachte Aktion
#[tauri::command]
pub async fn redo(
    state: State<'_, AppState>,
) -> Result<UndoEntryDto, String>;

/// Aktueller Zustand des Undo/Redo-Stacks
#[tauri::command]
pub async fn get_undo_state(
    state: State<'_, AppState>,
) -> Result<UndoStateDto, String>;

/// Speichert eine neue Version eines Skripts
#[tauri::command]
pub async fn save_script_version(
    script_id:   String,
    change_note: Option<String>,
    state:       State<'_, AppState>,
) -> Result<String, String>;  // Gibt ScriptVerId zurück

/// Listet Versionen eines Skripts
#[tauri::command]
pub async fn list_script_versions(
    script_id: String,
    state:     State<'_, AppState>,
) -> Result<Vec<ScriptVersionDto>, String>;

/// Stellt eine Skript-Version wieder her
#[tauri::command]
pub async fn restore_script_version(
    script_id:  String,
    version_id: String,
    state:      State<'_, AppState>,
) -> Result<(), String>;
```

### 18.2 Clipboard-Commands [NEU — v2.0]

```rust
/// Paginierte Zwischenablage-History mit Filter
#[tauri::command]
pub async fn list_clipboard_history(
    filter:    Option<ClipboardFilterDto>,
    page:      Option<u32>,
    page_size: Option<u32>,
    state:     State<'_, AppState>,
) -> Result<PagedResultDto<ClipboardEntryListItemDto>, String>;

/// Einzelnen Clipboard-Eintrag abrufen
#[tauri::command]
pub async fn get_clipboard_entry(
    id:    String,
    state: State<'_, AppState>,
) -> Result<ClipboardEntryDto, String>;

/// Clipboard-Eintrag pinnen/entpinnen
#[tauri::command]
pub async fn pin_clipboard_entry(
    id:     String,
    pinned: bool,
    state:  State<'_, AppState>,
) -> Result<(), String>;

/// Clipboard-Eintrag zu Snippet promoten
#[tauri::command]
pub async fn promote_clipboard_to_snippet(
    entry_id: String,
    title:    Option<String>,   // None = automatisch aus Content
    location: SnippetLocationDto,
    state:    State<'_, AppState>,
) -> Result<String, String>;   // Gibt SnippetId zurück

/// Clipboard-History-Eintrag löschen
#[tauri::command]
pub async fn delete_clipboard_entry(
    id:    String,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// Clipboard-History leeren (nur unpinned)
#[tauri::command]
pub async fn clear_clipboard_history(
    keep_pinned: bool,
    state:       State<'_, AppState>,
) -> Result<u32, String>;  // Anzahl gelöschter Einträge

/// Aktuellen System-Clipboard-Inhalt lesen (ohne Monitor-Trigger)
#[tauri::command]
pub async fn read_clipboard_now(
    state: State<'_, AppState>,
) -> Result<Option<String>, String>;

/// Text in Clipboard schreiben
#[tauri::command]
pub async fn write_to_clipboard(
    content:    String,
    snippet_id: Option<String>,  // Für usage tracking
    state:      State<'_, AppState>,
) -> Result<(), String>;
```

### 18.3 Import/Export-Commands [NEU — v2.0]

```rust
/// Exportiert Daten in gewähltem Format
/// Öffnet Tauri FileSaveDialog für Ausgabepfad
#[tauri::command]
pub async fn export_data(
    request: ExportRequestDto,
    state:   State<'_, AppState>,
) -> Result<ExportResultDto, String>;

/// Importiert aus .tfbundle oder anderen Formaten
/// Öffnet Tauri FileOpenDialog für Eingabedatei
#[tauri::command]
pub async fn import_data(
    request: ImportRequestDto,
    state:   State<'_, AppState>,
) -> Result<ImportResultDto, String>;

/// Vorschau eines Import-Bundles (ohne Datenbank-Änderungen)
#[tauri::command]
pub async fn preview_import(
    source_path: String,
    state:       State<'_, AppState>,
) -> Result<ImportPreviewDto, String>;
```

### 18.4 Settings und Session [ERWEITERT — v2.0]

```rust
// v1.0-Commands: get_all_settings, set_setting — bleiben unverändert

/// Workspace-Session laden
#[tauri::command]
pub async fn get_workspace_session(
    state: State<'_, AppState>,
) -> Result<Option<WorkspaceSessionDto>, String>;

/// Workspace-Session speichern
#[tauri::command]
pub async fn save_workspace_session(
    session: WorkspaceSessionDto,
    state:   State<'_, AppState>,
) -> Result<(), String>;

/// Datenbank-Statistiken (für Settings-Seite)
#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<DatabaseStatsDto, String>;
```

### 18.5 Tauri-Events (Backend → Frontend Push)

```typescript
// Events werden via app_handle.emit() gesendet
// Frontend abonniert via listen() in +layout.svelte

// Clipboard-Monitor: neuer Eintrag
// payload: ClipboardEntryListItemDto
listen('clipboard:new_entry', (event) => { ... });

// Pipeline-Ausführung: Fortschritt (bei langen Pipelines)
// payload: { stepId: string; stepLabel: string; stepIndex: number; totalSteps: number }
listen('pipeline:step_started', (event) => { ... });
listen('pipeline:step_complete', (event) => { ... });

// Bulk-Operationen: Fortschritt
// payload: { processed: number; total: number; currentId: string }
listen('bulk:progress', (event) => { ... });

// Import: Fortschritt
// payload: { processed: number; total: number; currentTitle: string }
listen('import:progress', (event) => { ... });
```

---

## § 19 — DTO-STRUKTUREN v2.0

```typescript
// [DDD] Anti-Corruption Layer: DTOs sind die IPC-Grenze
// [v1.0 DTOs bleiben vollständig erhalten]

// ── NEUE v2.0 DTOs ───────────────────────────────────────────────────────────

interface ClipboardEntryDto {
  id:            string;
  content:       string;
  contentHash:   string;
  contentType:   string;
  sourceApp:     string | null;
  capturedAt:    number;
  sizeBytes:     number;
  lineCount:     number;
  wordCount:     number;
  isPinned:      boolean;
  tags:          string[];
  promotedToSnippetId: string | null;
}

interface ClipboardEntryListItemDto {
  id:          string;
  preview:     string;           // Erste 200 Zeichen
  contentType: string;
  sourceApp:   string | null;
  capturedAt:  number;
  sizeBytes:   number;
  isPinned:    boolean;
  matchScore:  number | null;
}

interface TextStatsDto {
  charCount:          number;
  charNoSpaceCount:   number;
  wordCount:          number;
  lineCount:          number;
  paragraphCount:     number;
  sentenceCount:      number;
  estimatedTokens:    number;
  uniqueWordCount:    number;
  avgWordLength:      number;
  longestWord:        string;
  mostFrequentWords:  Array<{ word: string; count: number }>;
  avgSentenceLength:  number;
  fleschKincaidGrade: number | null;
  avgLineLength:      number;
  longestLineLength:  number;
  emptyLineCount:     number;
  readingTimeMs:      number;
}

interface ParsedTemplateDto {
  variables:       TemplateVariableDto[];
  requiredVars:    string[];
  optionalVars:    string[];
  hasConditionals: boolean;
  hasLoops:        boolean;
}

interface TemplateVariableDto {
  name:        string;
  hasDefault:  boolean;
  defaultVal:  string | null;
  filter:      string | null;
  isSpecial:   boolean;
  isRequired:  boolean;
  occurrences: number;
}

interface TemplateRenderResultDto {
  output:             string;
  resolvedVariables:  Record<string, string>;
  unresolvedVars:     string[];
  warnings:           string[];
}

interface DiffResultDto {
  lines:        DiffLineDto[];
  addedLines:   number;
  deletedLines: number;
  unchanged:    number;
  similarity:   number;
}

interface DiffLineDto {
  kind:        'equal' | 'insert' | 'delete';
  oldLineNum:  number | null;
  newLineNum:  number | null;
  content:     string;
}

interface BulkOperationDto {
  type:           string;
  snippetIds:     string[];
  pipelineId?:    string;
  saveResults?:   boolean;
  addTags?:       string[];
  removeTags?:    string[];
  targetLocation?: SnippetLocationDto;
  permanent?:     boolean;
  format?:        string;
  outputPath?:    string;
  pinned?:        boolean;
  favorite?:      boolean;
}

interface BulkOperationResultDto {
  succeeded:    string[];
  failed:       Array<{ id: string; error: string }>;
  totalCount:   number;
  durationMs:   number;
  previews:     Array<{ id: string; preview: string }> | null;
}

interface UndoStateDto {
  canUndo:       boolean;
  canRedo:       boolean;
  undoDescription: string | null;  // Beschreibung der nächsten Undo-Aktion
  redoDescription: string | null;
  undoCount:     number;
  redoCount:     number;
}

interface UndoEntryDto {
  description: string;
  performedAt: number;
}

interface ScriptVersionDto {
  id:          string;
  version:     number;
  savedAt:     number;
  changeNote:  string | null;
}

interface ExportRequestDto {
  format:        string;
  snippetIds:    string[] | null;
  scriptIds:     string[] | null;
  pipelineIds:   string[] | null;
  includeFolders: boolean;
  outputPath:    string;
}

interface ExportResultDto {
  path:          string;
  bytesWritten:  number;
  exportedCounts: { snippets: number; scripts: number; pipelines: number };
  format:        string;
  completedAt:   number;
}

interface ImportRequestDto {
  sourcePath:      string;
  format:          string;
  conflictPolicy:  string;
  targetLocationType: string;
  targetFolderId:  string | null;
}

interface ImportResultDto {
  imported:   { snippets: number; scripts: number; pipelines: number; folders: number };
  skipped:    number;
  conflicts:  Array<{ existingTitle: string; importedTitle: string; resolution: string }>;
  errors:     Array<{ item: string; error: string }>;
  completedAt: number;
}

interface ImportPreviewDto {
  bundleVersion: string;
  appVersion:    string;
  createdAt:     number;
  counts:        { snippets: number; scripts: number; pipelines: number; folders: number };
  compatible:    boolean;
  incompatibilityReason: string | null;
}

interface WorkspaceSessionDto {
  activeView:            string;
  lastActiveSnippetId:   string | null;
  lastActiveScriptId:    string | null;
  lastActivePipelineId:  string | null;
  sidebarWidth:          number;
  previewMode:           string;
  openEditorTabs:        EditorTabDto[];
}

interface EditorTabDto {
  entityType: string;
  entityId:   string;
  isDirty:    boolean;
  scrollPos:  number;
  cursorLine: number | null;
}

interface DatabaseStatsDto {
  snippetCount:        number;
  scriptCount:         number;
  pipelineCount:       number;
  clipboardEntryCount: number;
  tagCount:            number;
  folderCount:         number;
  totalSizeBytes:      number;
  oldestEntry:         number | null;
  dbFilePath:          string;
  walEnabled:          boolean;
}
```

---

## § 20 — FRONTEND-ARCHITEKTUR v2.0 (SvelteKit)

### 20.1 Store-Topologie [ERWEITERT — v2.0]

```typescript
// [FP-Scala] Stores sind der einzige Ort für veränderlichen Zustand
// Alle Derived Stores sind rein abgeleitet — kein versteckter Zustand

// ── PRIMITIVE STORES (Quelle der Wahrheit) ──────────────────────────────────
const snippetsStore          = writable<SnippetListItem[]>([]);
const activeFolderStore      = writable<Option<FolderId>>(Option.none());
const filterStore            = writable<SnippetFilter>(SnippetFilter.default());
const foldersStore           = writable<Folder[]>([]);
const scriptsStore           = writable<Script[]>([]);
const pipelinesStore         = writable<Pipeline[]>([]);
const clipboardStore         = writable<ClipboardEntryListItemDto[]>([]);   // [v2.0]
const clipboardFilterStore   = writable<ClipboardFilter>({ /* defaults */ }); // [v2.0]
const undoStateStore         = writable<UndoStateDto>({ canUndo: false, canRedo: false, undoDescription: null, redoDescription: null, undoCount: 0, redoCount: 0 }); // [v2.0]
const notificationsStore     = writable<AppNotification[]>([]);             // [v2.0]
const activeSessionStore     = writable<WorkspaceSession>(WorkspaceSession.default()); // [v2.0]
const loadingStore           = writable<LoadingState>({ snippets: false, transform: false, bulk: false }); // [v2.0]
const errorStore             = writable<Option<DomainError>>(Option.none());

// ── DERIVED STORES (rein abgeleitet — niemals direkt setzen) ─────────────────
const filteredSnippets       = derived([snippetsStore, filterStore], applyFilter);
const tagCloud               = derived(snippetsStore, buildTagCloud);
const folderTree             = derived(foldersStore, buildFolderTree);
const activeSnippet          = derived([snippetsStore, activeSessionStore], findActiveSnippet);
const pinnedSnippets         = derived(snippetsStore, ss => ss.filter(s => s.isPinned));
const favoriteSnippets       = derived(snippetsStore, ss => ss.filter(s => s.isFavorite));
const templateSnippets       = derived(snippetsStore, ss => ss.filter(s => s.isTemplate));
const recentClipboardItems   = derived(clipboardStore, cs => cs.slice(0, 10));  // [v2.0]
const pinnedClipboardItems   = derived(clipboardStore, cs => cs.filter(c => c.isPinned)); // [v2.0]
```

### 20.2 Erweiterte Dateistruktur v2.0

```
src/
├── lib/
│   ├── domain/
│   │   ├── adts.ts                  # Option, Result, NonEmptyArray — KEINE Deps
│   │   ├── snippet.ts               # Snippet, SnippetLocation, ContentType
│   │   ├── script.ts                # Script, ScriptParameter, ScriptTest, ScriptVersion
│   │   ├── pipeline.ts              # Pipeline, PipelineStep, FailurePolicy, Condition
│   │   ├── filter.ts                # SnippetFilter, ClipboardFilter, DateRange...
│   │   ├── template.ts              # TemplateRenderer, TemplateVariable (rein)
│   │   ├── text-stats.ts            # computeTextStats, estimateTokens (rein)
│   │   ├── diff.ts                  # DiffResult, DiffLine (Typen, rein)
│   │   ├── clipboard-entry.ts       # ClipboardEntry, ClipboardEntry.create (rein)
│   │   ├── undo.ts                  # UndoStack, UndoAction (rein)
│   │   ├── session.ts               # WorkspaceSession (rein)
│   │   ├── notifications.ts         # AppNotification, Notifications.*
│   │   ├── import-export.ts         # ExportRequest, ImportResult, Bundle-Format
│   │   └── errors.ts                # DomainError (erschöpfend)
│   │
│   ├── ipc/
│   │   ├── snippets.ts              # Typisierte invoke()-Wrapper für alle Snippet-Commands
│   │   ├── scripts.ts               # Script-Commands
│   │   ├── pipelines.ts             # Pipeline-Commands
│   │   ├── clipboard.ts             # Clipboard-Commands [v2.0]
│   │   ├── transform.ts             # Transform-Commands (execute_script, execute_pipeline)
│   │   ├── text-analysis.ts         # compute_text_stats, parse_template, render_template [v2.0]
│   │   ├── diff.ts                  # compute_diff [v2.0]
│   │   ├── bulk.ts                  # execute_bulk_operation [v2.0]
│   │   ├── undo.ts                  # undo, redo, get_undo_state [v2.0]
│   │   ├── import-export.ts         # export_data, import_data, preview_import [v2.0]
│   │   ├── session.ts               # get/save_workspace_session [v2.0]
│   │   └── settings.ts              # get_all_settings, set_setting, get_database_stats
│   │
│   ├── stores/
│   │   ├── snippets.ts              # snippetsStore, filteredSnippets, tagCloud...
│   │   ├── scripts.ts               # scriptsStore, activeScript
│   │   ├── pipelines.ts             # pipelinesStore
│   │   ├── clipboard.ts             # clipboardStore, recentItems [v2.0]
│   │   ├── undo.ts                  # undoStateStore, push/undo/redo actions [v2.0]
│   │   ├── notifications.ts         # notificationsStore, push/dismiss [v2.0]
│   │   ├── session.ts               # activeSessionStore, sessionPersistence [v2.0]
│   │   ├── ui.ts                    # PreviewMode, DiffMode, activeTab...
│   │   └── settings.ts              # settingsStore
│   │
│   ├── components/
│   │   ├── snippets/
│   │   │   ├── SnippetList.svelte             # Paginiert, virtuell scroll
│   │   │   ├── SnippetListItem.svelte         # Zeile mit Metadaten + Farb-Chip
│   │   │   ├── SnippetEditor.svelte           # Monaco + Metadaten-Formular
│   │   │   ├── SnippetPreview.svelte          # Markdown-Vorschau
│   │   │   ├── SnippetEditorLayout.svelte     # Split: Editor | Preview
│   │   │   ├── SnippetBulkToolbar.svelte      # Erscheint bei Mehrfachauswahl [v2.0]
│   │   │   ├── SnippetColorPicker.svelte      # Farb-Markierung [v2.0]
│   │   │   └── SnippetStats.svelte            # TextStats-Panel [v2.0]
│   │   │
│   │   ├── filter/
│   │   │   ├── FilterPanel.svelte             # Gesamter Filter-Bereich
│   │   │   ├── TagFilter.svelte               # Tag-Chips AND/OR
│   │   │   ├── LocationFilter.svelte          # Ordner-Baum
│   │   │   ├── DateRangeFilter.svelte         # Preset + Custom
│   │   │   ├── SizeFilter.svelte              # Größenbereich-Slider
│   │   │   ├── ContentTypeFilter.svelte       # Icon-Grid
│   │   │   ├── WordCountFilter.svelte         # [v2.0]
│   │   │   └── QuickFilterBar.svelte          # Pinned/Favorites/Templates [v2.0]
│   │   │
│   │   ├── scripts/
│   │   │   ├── ScriptList.svelte              # Skriptbibliothek
│   │   │   ├── ScriptEditor.svelte            # Monaco + Parameter-GUI
│   │   │   ├── ScriptParameterForm.svelte     # Dynamisches Formular
│   │   │   ├── ScriptTester.svelte            # Live-Test: Input → Output
│   │   │   ├── RegexBuilder.svelte            # Visueller Regex-Builder
│   │   │   ├── ScriptTestSuite.svelte         # Test-Ergebnisse + Run
│   │   │   └── ScriptVersionHistory.svelte    # Versionsliste + Restore [v2.0]
│   │   │
│   │   ├── pipeline/
│   │   │   ├── PipelineEditor.svelte          # Drag-Drop Step-Builder
│   │   │   ├── PipelineStep.svelte            # Einzelner Schritt
│   │   │   ├── PipelineRunner.svelte          # Ausführung + Schritt-Ergebnis
│   │   │   ├── StepResultCard.svelte          # Schritt-Ergebnis mit Diff [v2.0]
│   │   │   └── PipelineConditionEditor.svelte # Schritt-Bedingung [v2.0]
│   │   │
│   │   ├── clipboard/                         # [NEU v2.0]
│   │   │   ├── ClipboardHistory.svelte        # Paginierte History
│   │   │   ├── ClipboardEntry.svelte          # Einzelner Eintrag
│   │   │   ├── ClipboardFilter.svelte         # Filter-Panel für Clipboard
│   │   │   └── ClipboardEntryActions.svelte   # Pin, Promote, Delete
│   │   │
│   │   ├── template/                          # [NEU v2.0]
│   │   │   ├── TemplateForm.svelte            # Auto-generiertes Variablen-Formular
│   │   │   ├── TemplatePreview.svelte         # Gerenderte Vorschau
│   │   │   └── TemplateVariableChip.svelte    # Variable-Tag im Editor
│   │   │
│   │   ├── diff/                              # [NEU v2.0]
│   │   │   ├── DiffViewer.svelte              # Unified/Split/Inline-Diff
│   │   │   └── DiffStats.svelte               # +N -N Zeilen Zusammenfassung
│   │   │
│   │   ├── import-export/                     # [NEU v2.0]
│   │   │   ├── ExportDialog.svelte            # Format + Auswahl
│   │   │   ├── ImportDialog.svelte            # Datei + Konflikt-Policy
│   │   │   └── ImportPreviewTable.svelte      # Vorschau vor Import
│   │   │
│   │   └── shared/
│   │       ├── MonacoEditor.svelte            # Monaco (JS, Regex, MD, Text)
│   │       ├── TagInput.svelte                # Chip-basierte Tag-Eingabe
│   │       ├── ErrorBanner.svelte             # DomainError → Nachricht
│   │       ├── ConfirmDialog.svelte           # Zerstörerische Aktionen
│   │       ├── Toast.svelte                   # AppNotification-Anzeige [v2.0]
│   │       ├── ToastContainer.svelte          # Toast-Stack [v2.0]
│   │       ├── UndoRedoButtons.svelte         # Toolbar-Buttons [v2.0]
│   │       ├── CommandPalette.svelte          # Ctrl+Shift+P Suche [v2.0]
│   │       ├── KeyboardShortcutHint.svelte    # Shortcut-Badge [v2.0]
│   │       └── StatsPanel.svelte              # TextStats-Darstellung [v2.0]
│   │
│   └── utils/
│       ├── content-type.ts          # detectContentType() — rein
│       ├── markdown.ts              # renderMarkdown() — rein
│       ├── size-format.ts           # formatBytes() — rein
│       ├── date-format.ts           # formatUnixMs() — rein
│       ├── template-render.ts       # client-seitige TemplateRenderer-Impl [v2.0]
│       ├── shortcut-registry.ts     # ShortcutMap + Event-Listener [v2.0]
│       ├── session-persistence.ts   # Debounced Session-Speicherung [v2.0]
│       └── clipboard-events.ts     # Tauri-Event-Listener für Clipboard [v2.0]
│
└── routes/
    ├── +layout.svelte               # App-Shell: Sidebar + Haupt + Toasts + Session
    ├── snippets/
    │   ├── +page.svelte             # Liste + Filter
    │   └── [id]/+page.svelte        # Editor + Vorschau
    ├── scripts/
    │   ├── +page.svelte             # Skriptbibliothek
    │   └── [id]/+page.svelte        # Editor + Tester + Versionen
    ├── pipelines/
    │   ├── +page.svelte             # Alle Pipelines
    │   └── [id]/+page.svelte        # Builder + Runner
    ├── clipboard/
    │   └── +page.svelte             # [NEU v2.0] Clipboard-History
    └── settings/+page.svelte        # Einstellungen + DB-Stats
```

---

## § 21 — SANDBOX-SPEZIFIKATION (QuickJS) [v1.0-kompatibel, erweitert]

### 21.1 Erlaubt / Verboten [unverändert aus v1.0]

```
ERLAUBT:
  ✓ String-Methoden (split, replace, match, trim, padStart, repeat, ...)
  ✓ Array-Methoden (map, filter, reduce, flat, flatMap, find, sort, ...)
  ✓ Object.keys/values/entries/assign/fromEntries
  ✓ JSON.parse / JSON.stringify
  ✓ Math.*
  ✓ Date (nur Lesen: Date.now(), new Date().toISOString())
  ✓ RegExp
  ✓ Number.parseInt/parseFloat/isNaN/isFinite
  ✓ utils.* — Injiziertes Prelude (siehe 21.2)
  ✓ params.* — GUI-Parameter
  ✓ console.log → wird als warning gesammelt

VERBOTEN:
  ✗ fetch / XMLHttpRequest / WebSocket
  ✗ fs / require / import
  ✗ process / Deno / Bun / globalThis.global
  ✗ eval() / Function() Konstruktor
  ✗ WebAssembly
  ✗ SharedArrayBuffer / Atomics
  ✗ Clipboard-Zugriff (Output nur via return)
  ✗ setTimeout / setInterval (synchron-only Sandbox)
```

### 21.2 utils.* Prelude [ERWEITERT — v2.0]

```javascript
const utils = {
  // ── Text-Helfer ──────────────────────────────────────────────────────────────
  lines:           (s)       => s.split('\n'),
  unlines:         (arr)     => arr.join('\n'),
  words:           (s)       => s.trim().split(/\s+/).filter(Boolean),
  unwords:         (arr)     => arr.join(' '),
  trim:            (s)       => s.trim(),
  trimLines:       (s)       => utils.unlines(utils.lines(s).map(l => l.trim())),
  truncate:        (s, n)    => s.length <= n ? s : s.slice(0, n) + '…',
  wrap:            (s, n)    => {
    // Word-wrap bei n Zeichen
    const words = s.split(' ');
    const result = [];
    let line = '';
    for (const w of words) {
      if ((line + w).length > n) { result.push(line.trimEnd()); line = ''; }
      line += w + ' ';
    }
    if (line.trim()) result.push(line.trimEnd());
    return result.join('\n');
  },
  padLeft:         (s, n, c) => s.padStart(n, c ?? ' '),
  padRight:        (s, n, c) => s.padEnd(n, c ?? ' '),
  repeat:          (s, n)    => s.repeat(n),
  reverse:         (s)       => [...s].reverse().join(''),
  capitalize:      (s)       => s.charAt(0).toUpperCase() + s.slice(1).toLowerCase(),
  titleCase:       (s)       => s.replace(/\b\w/g, c => c.toUpperCase()),
  slugify:         (s)       => s.toLowerCase().replace(/[^\w\s-]/g, '').replace(/[\s_-]+/g, '-').replace(/^-+|-+$/g, ''),
  camelToSnake:    (s)       => s.replace(/[A-Z]/g, c => '_' + c.toLowerCase()),
  snakeToCamel:    (s)       => s.replace(/_([a-z])/g, (_, c) => c.toUpperCase()),
  count:           (s, sub)  => (s.match(new RegExp(sub.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g')) || []).length,

  // ── Zeilen-Helfer ────────────────────────────────────────────────────────────
  sortLines:       (s)       => utils.unlines(utils.lines(s).sort()),
  uniqueLines:     (s)       => utils.unlines([...new Set(utils.lines(s))]),
  reverseLines:    (s)       => utils.unlines(utils.lines(s).reverse()),
  filterLines:     (s, pred) => utils.unlines(utils.lines(s).filter(pred)),
  mapLines:        (s, fn)   => utils.unlines(utils.lines(s).map(fn)),
  numberedLines:   (s)       => utils.unlines(utils.lines(s).map((l, i) => `${i + 1}. ${l}`)),
  prefixLines:     (s, pfx)  => utils.mapLines(s, l => pfx + l),
  suffixLines:     (s, sfx)  => utils.mapLines(s, l => l + sfx),
  indentLines:     (s, n)    => utils.prefixLines(s, ' '.repeat(n)),

  // ── JSON-Helfer ──────────────────────────────────────────────────────────────
  prettyJSON:      (s)       => JSON.stringify(JSON.parse(s), null, 2),
  minifyJSON:      (s)       => JSON.stringify(JSON.parse(s)),
  parseJSON:       (s)       => JSON.parse(s),
  stringifyJSON:   (v, ind)  => JSON.stringify(v, null, ind ?? 2),
  jsonKeys:        (s)       => Object.keys(JSON.parse(s)).join('\n'),

  // ── Code-Helfer ──────────────────────────────────────────────────────────────
  wrapMarkdown:    (s, lang) => '```' + (lang || '') + '\n' + s + '\n```',
  stripMarkdown:   (s)       => s.replace(/#{1,6}\s|(\*\*|__)(.*?)\1|\*|_|\[([^\]]*)\]\([^)]*\)|`[^`]*`|```[\s\S]*?```/g, '$3'),
  extractCodeBlocks: (s)     => {
    const blocks = [...s.matchAll(/```(?:\w+)?\n([\s\S]*?)```/g)].map(m => m[1]);
    return blocks.join('\n---\n');
  },

  // ── Sicherheits-Helfer ───────────────────────────────────────────────────────
  redact: (s) => s
    .replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, '[IP]')
    .replace(/[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g, '[EMAIL]')
    .replace(/\b(sk|pk|api)[-_][\w\-]{8,}/gi, '[API_KEY]')
    .replace(/Bearer\s+[\w\-._~+/]+=*/gi, 'Bearer [TOKEN]')
    .replace(/\b(?:\d[ \-]?){13,16}\b/g, '[CARD]'),

  // ── Codierung ────────────────────────────────────────────────────────────────
  base64Encode:    (s)    => btoa(unescape(encodeURIComponent(s))),
  base64Decode:    (s)    => decodeURIComponent(escape(atob(s))),
  urlEncode:       (s)    => encodeURIComponent(s),
  urlDecode:       (s)    => decodeURIComponent(s),
  htmlEscape:      (s)    => s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;'),
  htmlUnescape:    (s)    => s.replace(/&amp;/g,'&').replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&quot;/g,'"').replace(/&#39;/g,"'"),

  // ── Analyse ──────────────────────────────────────────────────────────────────
  charCount:       (s)    => s.length,
  wordCount:       (s)    => s.trim() === '' ? 0 : s.trim().split(/\s+/).length,
  lineCount:       (s)    => s.split('\n').length,
  tokenEstimate:   (s)    => Math.round(Math.max(s.length / 4, s.trim().split(/\s+/).length * 0.75)),

  // ── Datum/Zeit (nur Lesen) ───────────────────────────────────────────────────
  today:     ()    => new Date().toISOString().split('T')[0],
  timestamp: ()    => new Date().toISOString(),
  unixMs:    ()    => Date.now(),
};
```

### 21.3 Skript-Vertrag [v1.0-kompatibel]

```javascript
// Verfügbare Variablen im Skript-Scope:
// input:  string           — Der Eingabetext (immer gesetzt, nie null)
// utils:  UtilsObject      — Sicheres Helfer-Objekt
// params: ParameterValues  — GUI-Parameter (Record<string, string|number|boolean>)

// Regeln:
// 1. Der Rückgabewert des letzten Ausdrucks ist der Output
// 2. Rückgabewert MUSS ein String sein — sonst ScriptInvalidOutputError
// 3. Kein explicit return nötig bei einzeiligen Ausdrücken
// 4. Bei Fehler: DomainError.SCRIPT_RUNTIME_ERROR mit details

// Beispiele:
input.trim()                                          // Einfach
utils.lines(input).filter(l => l.trim()).join('\n')  // Mittel
const n = params.count ?? 3;                         // Mit Parameter
utils.unlines(utils.lines(input).slice(0, n))
```

---

## § 22 — SETTINGS SCHEMA v2.0

```typescript
// Settings sind String-Werte in SQLite (key-value)
// Der Typ ist implizit durch den Key definiert
// [PragProg] DRY: Alle Settings-Defaults an EINEM Ort

const SETTINGS_SCHEMA = {
  // Sandbox
  'sandbox.output_limit_bytes':     { type: 'number', default: 524288 },   // 512 KB
  'sandbox.input_limit_bytes':      { type: 'number', default: 2097152 },  // 2 MB
  'sandbox.timeout_ms':             { type: 'number', default: 3000 },
  'sandbox.large_mode_enabled':     { type: 'boolean', default: false },
  'sandbox.large_mode_limit_bytes': { type: 'number', default: 10485760 }, // 10 MB

  // Pipeline
  'pipeline.global_strict_mode':    { type: 'boolean', default: false },

  // Clipboard [v2.0]
  'clipboard.enabled':              { type: 'boolean', default: true },
  'clipboard.max_entries':          { type: 'number',  default: 500 },
  'clipboard.min_length':           { type: 'number',  default: 3 },
  'clipboard.dedup_window_ms':      { type: 'number',  default: 500 },

  // UI
  'ui.theme':                       { type: 'string',  default: 'dark', options: ['dark', 'light', 'system'] },
  'ui.preview_mode':                { type: 'string',  default: 'split', options: ['editor', 'preview', 'split'] },
  'ui.snippets_page_size':          { type: 'number',  default: 50 },
  'ui.editor_font_size':            { type: 'number',  default: 14 },
  'ui.editor_font_family':          { type: 'string',  default: 'JetBrains Mono' },
  'ui.editor_word_wrap':            { type: 'boolean', default: true },
  'ui.show_stats_panel':            { type: 'boolean', default: true },    // [v2.0]
  'ui.diff_mode':                   { type: 'string',  default: 'split', options: ['unified', 'split', 'inline'] }, // [v2.0]
  'ui.shortcut_hints':              { type: 'boolean', default: true },    // [v2.0]
  'ui.confirm_bulk_delete':         { type: 'boolean', default: true },    // [v2.0]
  'ui.sidebar_collapsed':           { type: 'boolean', default: false },   // [v2.0]

  // Filter
  'filter.default_sort_by':         { type: 'string',  default: 'updatedAt' },
  'filter.default_sort_dir':        { type: 'string',  default: 'desc' },

  // Template [v2.0]
  'template.strict_mode':           { type: 'boolean', default: false },

  // Script [v2.0]
  'script.max_versions':            { type: 'number',  default: 20 },
  'script.auto_save_version':       { type: 'boolean', default: true },    // Bei jeder Speicherung

  // Undo [v2.0]
  'undo.max_stack_size':            { type: 'number',  default: 50 },

  // Export [v2.0]
  'export.default_format':          { type: 'string',  default: 'bundle' },
  'export.include_metadata':        { type: 'boolean', default: true },
  'export.last_directory':          { type: 'string',  default: '' },

  // Session [v2.0]
  'session.restore_on_start':       { type: 'boolean', default: true },
  'session.workspace':              { type: 'string',  default: '' },      // JSON: WorkspaceSession
} as const satisfies Record<string, { type: string; default: unknown; options?: string[] }>;

type SettingKey = keyof typeof SETTINGS_SCHEMA;
```

---

## § 23 — TECHNOLOGIE-STACK v2.0

```
BACKEND (Rust / Tauri 2.x):
├── tauri 2.x                — App-Framework, IPC, Fenster, FileDialog
├── sqlx + sqlite3           — Async DB, Type-Safe Queries, WAL-Mode, FTS5
├── rquickjs                 — QuickJS-Sandbox für JS-Transformationen
├── regex                    — Rust-native Regex-Engine (Transformationen)
├── tokio                    — Async Runtime (Clipboard-Monitor, DB)
├── serde / serde_json       — Serialisierung
├── sha2                     — SHA-256 für Content-Hashing und Dedup
├── uuid                     — UUID v4 für IDs
├── chrono                   — Timestamps (UTC)
├── arboard                  — Cross-platform Clipboard-Lesen und -Schreiben
├── similar                  — Myers-Diff-Algorithmus (DiffResult)
└── zip                      — .tfbundle Import/Export (ZIP-Archiv)

FRONTEND (TypeScript / SvelteKit):
├── SvelteKit                — UI-Framework (Stores, Routing)
├── Monaco Editor            — Editor für JS, Regex, Markdown, Text
├── marked + DOMPurify       — Markdown-Rendering + XSS-Schutz
├── Prism.js                 — Syntax-Highlighting
├── svelte-dnd-action        — Drag & Drop im Pipeline-Builder
├── Tailwind CSS 4           — Utility Styles
├── Vitest                   — Unit Tests für Pure-Core-Funktionen
├── diff-match-patch         — Client-seitiger Inline-Diff-Renderer [v2.0]
└── KaTeX                    — LaTeX-Rendering (opt-in) [v2.0]

LINUX-PLATFORM (KDE Plasma 6, Wayland-only) [GEÄNDERT — v2.1]:
├── Display: Wayland (kein X11-Fallback)
├── Clipboard: wl-paste --watch subprocess (primär) / arboard-Polling (Fallback)
├── Source App: KWin D-Bus API / qdbus6 subprocess / procfs-Fallback
├── Tauri baut: .deb und AppImage
└── Benötigt: wl-clipboard (apt install wl-clipboard) + qdbus6 (in kde-cli-tools)

CARGO.TOML DEPENDENCIES:
[dependencies]
tauri         = { version = "2", features = ["dialog"] }
sqlx          = { version = "0.7", features = ["sqlite", "runtime-tokio", "macros"] }
rquickjs      = "0.6"
regex         = "1"
tokio         = { version = "1", features = ["full"] }
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
sha2          = "0.10"
uuid          = { version = "1", features = ["v4"] }
chrono        = { version = "0.4", features = ["serde"] }
arboard       = "3"
similar       = "2"
zip           = "2"

# Keine x11rb-Abhängigkeit mehr — Wayland-only via wl-paste subprocess
# wl-clipboard muss auf dem System installiert sein (kde-cli-tools enthält qdbus6)
```

---

## § 24 — ARCHITEKTUR-CHECKLISTE v2.0

Vor **jeder** Implementierung vollständig prüfen:

```
── DOMAIN CORE ─────────────────────────────────────────────────────────────────
□ Gibt die Funktion Result<DomainError, T> statt zu werfen?
□ Gibt es null-Rückgaben? → Durch Option<T> ersetzen
□ Wird ein Argument mutiert? → Neue Kopie erstellen (Spread / structuredClone)
□ Hat die Funktion mehr als 3 Parameter? → Options-Objekt einführen
□ Ist der Fehlerfall in DomainError erschöpfend dargestellt?
□ Ist ein unmöglicher Zustand im Typ darstellbar? → Union-Typ verfeinern
□ Ist die Funktion ohne DB/QuickJS/Clipboard testbar?
□ Ist der Name eine Absichtsaussage (was), keine Implementierungsaussage (wie)?
□ Sind alle Timestamps UnixMs (UTC, Millisekunden)?

── SNIPPET / SCRIPT / PIPELINE ─────────────────────────────────────────────────
□ Wurden alle INVARIANT-* für die betroffene Entity geprüft?
□ Wurde updatedAt bei jeder Mutation aktualisiert?
□ Sind alle FTS5-Trigger nach Schema-Änderungen noch intakt?
□ Wird isSafetyCritical korrekt auf 'abort' gesetzt?
□ Haben neue Felder Defaults die bestehende Daten nicht brechen?

── TEMPLATE ─────────────────────────────────────────────────────────────────────
□ Werden alle {{_spezial}} Variablen korrekt injiziert?
□ Wird strict_mode aus Settings berücksichtigt?
□ Sind zirkuläre Referenzen in Templates abgesichert?

── CLIPBOARD ────────────────────────────────────────────────────────────────────
□ Wird leerer Content (< min_length) verworfen?
□ Wird SHA-256-Dedup korrekt angewendet?
□ Wird das LRU-Limit (max_entries) enforced?
□ Werden nur unpinned Einträge beim LRU-Trim gelöscht?

── UNDO/REDO ────────────────────────────────────────────────────────────────────
□ Wird die Aktion in den UndoStack gepusht bevor die DB-Operation?
□ Wird der UndoStack bei neuer Aktion korrekt geleert (Redo-Branch)?
□ Erzeugen Bulk-Operationen exakt einen UndoEntry?

── IMPORT/EXPORT ────────────────────────────────────────────────────────────────
□ Werden Checksummen im Bundle verifiziert?
□ Wird die Bundle-Version geprüft und kommuniziert?
□ Respektiert der Import die ConflictPolicy vollständig?

── FRONTEND / STORES ────────────────────────────────────────────────────────────
□ Wird der Store nur durch IPC-Calls mutiert — nie direkt?
□ Werden Derived Stores korrekt als derived() deklariert?
□ Wird die WorkspaceSession bei signifikanten UI-Änderungen (debounced) gespeichert?
□ Werden Notifications via notificationsStore.push() statt alert() gezeigt?
□ Sind alle fehlerhaften Zustände durch ErrorBanner oder Toast sichtbar?

── SECURITY ─────────────────────────────────────────────────────────────────────
□ Wird die QuickJS-Sandbox für JEDEN JS-Aufruf verwendet (kein eval im Frontend)?
□ Werden Markdown-Outputs immer durch DOMPurify sanitisiert?
□ Haben safety-critical Builtins failurePolicy = 'abort'?
□ Werden sensitive Clipboard-Inhalte nicht in Logs geschrieben?
```

---

## § 25 — DATA FLOWS (Vollständige User Journeys) [NEU — v2.0]

```
FLOW A: Clipboard-Eintrag → Snippet (Kernloop)
─────────────────────────────────────────────────
1. Nutzer kopiert Text in beliebiger App (Browser, IDE, Terminal)
2. wl-paste --watch stdout empfängt neuen Inhalt (push-basiert, kein polling)
3. SHA-256-Dedup → falls identisch: verwerfen
4. ClipboardEntry.create() → Option<ClipboardEntry>
5. SQLite: INSERT INTO clipboard_history
6. LRU-Trim: älteste unpinned Einträge bei Überschreitung max_entries löschen
7. Tauri-Event "clipboard:new_entry" → Frontend
8. clipboardStore aktualisieren (prepend + slice(0, 50) für UI)
9. Notification: "1 neuer Eintrag" (sofern Clipboard-View aktiv)
10. Nutzer öffnet Clipboard-Ansicht, wählt Eintrag
11. "Als Snippet speichern" → promote_clipboard_to_snippet IPC
12. UndoStack.push({ _type: 'snippet_create', ... })
13. Notification: "Snippet 'Titel' gespeichert" mit Undo-Aktion

FLOW B: Template ausfüllen und in Clipboard kopieren
─────────────────────────────────────────────────────
1. Nutzer öffnet Snippet mit isTemplate = true
2. TemplateRenderer.parse(content) → ParsedTemplate mit requiredVars + optionalVars
3. TemplateForm.svelte rendert automatisch Eingabefelder (required = rot markiert)
4. Nutzer füllt Felder aus (Live-Vorschau in TemplatePreview.svelte)
5. "Kopieren" → TemplateRenderer.render(content, context)
6. Result<DomainError, TemplateRenderResult>
7. Bei TEMPLATE_MISSING_VARIABLE + strict: Fehler anzeigen, kein Kopieren
8. Bei strict = false: unresolved Variablen bleiben als {{var}} erhalten
9. write_to_clipboard IPC → arboard::set_text() (arboard nutzt wl-clipboard intern)
10. update_snippet (usageCount++) IPC
11. INSERT INTO usage_history (action: 'copy')
12. UndoStack: kein Eintrag (Kopieren ist nicht undo-bar)
13. Toast: "Kopiert!" (1.2s)

FLOW C: Transform-Pipeline auf Snippet anwenden
─────────────────────────────────────────────────
1. Nutzer wählt Snippet + Pipeline
2. execute_pipeline IPC → QuickJS-Sandbox
3. Jeder Step: read input → run script → write output
4. PipelineExecutionResult mit StepResults
5. UI zeigt DiffViewer: original vs. transformierter Text
6. Nutzer bestätigt "Übernehmen"
7. update_snippet(patch: { content: result }) IPC
8. UndoStack.push({ _type: 'transform_apply', originalContent, transformedContent })
9. Toast: "Transformation abgeschlossen – Rückgängig möglich"

FLOW D: Bulk-Transformation mit Vorschau
─────────────────────────────────────────
1. Nutzer selektiert mehrere Snippets (Checkbox oder Shift+Klick)
2. SnippetBulkToolbar erscheint mit verfügbaren Aktionen
3. "Transformation anwenden" → BulkOperation{ _type: 'bulk_transform', saveResults: false }
4. execute_bulk_operation IPC
5. Tauri-Event "bulk:progress" → UI zeigt Fortschrittsbalken
6. BulkOperationResult.previews: Vorschau für jedes Snippet
7. Nutzer prüft Vorschau-Liste (DiffViewer je Snippet)
8. "Alle übernehmen" → BulkOperation{ saveResults: true }
9. Einzelner UndoEntry für gesamte Bulk-Operation
10. Toast: "N Snippets transformiert – Rückgängig möglich"

FLOW E: Import aus .tfbundle
─────────────────────────────
1. Datei → Import → Tauri FileSaveDialog
2. preview_import IPC → ImportPreviewDto (kompatibel? Anzahl Einträge?)
3. ImportPreviewTable.svelte zeigt Übersicht
4. Nutzer wählt ConflictPolicy + Zielort
5. import_data IPC
6. Tauri-Event "import:progress" → Fortschrittsanzeige
7. ImportResult: succeeded + skipped + conflicts + errors
8. Toast: "Import abgeschlossen: N Snippets, M Skripte importiert"
9. Kein UndoStack-Eintrag (Import-Undo via Bulk-Delete wenn nötig)
```

---

*TextForge Interface Specification v2.1*
*Ableitend aus: PromptStation-Architecture v0.1/v0.2 + TextForge v1.0/v2.0 + JS-Principles-System-Prompt v1.0*
*Zielplattform: KDE Plasma 6 / Wayland / Tauri 2.x / SvelteKit*
*Alle Typen, Invarianten, IPC-Signaturen und Architekturprinzipien sind unveränderliche Eigenschaften — nicht Richtlinien.*

---

## ANHANG A — Vollständige BuiltinId-Referenz mit Beschreibungen

| BuiltinId | Kategorie | TokenImpact | SafetyCritical | Beschreibung |
|---|---|---|---|---|
| trim | text | none | false | Führende/folgende Leerzeichen und Newlines entfernen |
| remove_empty_lines | text | low | false | Leerzeilen und Zeilen nur mit Whitespace löschen |
| collapse_whitespace | text | low | false | Mehrfaches Whitespace (inkl. Tabs) auf ein Leerzeichen reduzieren |
| normalize_whitespace | text | none | false | Alle Unicode-Whitespace-Varianten → normales Leerzeichen |
| remove_non_ascii | text | low | false | Alle Zeichen außerhalb ASCII (0–127) entfernen |
| normalize_unicode | text | none | false | NFC-Normalisierung: kombinierte Akzent-Zeichen vereinheitlichen |
| remove_accents | text | none | false | Diakritika entfernen (é→e, ü→u, ñ→n) |
| truncate | text | high | false | Text auf N Zeichen kürzen (Param: maxLength, ellipsis) |
| summary_cut | text | high | false | Anfang (N Zeichen) + Ende (N Zeichen), Mitte durch "…[N Zeichen entfernt]…" |
| first_n_lines | text | high | false | Erste N Zeilen behalten (Param: n) |
| last_n_lines | text | high | false | Letzte N Zeilen behalten (Param: n) |
| wrap_text | text | none | false | Word-Wrap bei N Zeichen (Param: width) |
| uppercase | text | none | false | Gesamten Text in Großbuchstaben |
| lowercase | text | none | false | Gesamten Text in Kleinbuchstaben |
| title_case | text | none | false | Jeden Wortanfang großschreiben |
| sentence_case | text | none | false | Ersten Buchstaben groß, Rest klein |
| alternating_case | text | none | false | Wechselnde Groß-/Kleinschreibung |
| rot13 | text | none | false | ROT13-Zeichenrotation |
| sort_lines | text | none | false | Zeilen alphabetisch sortieren (Param: caseSensitive) |
| sort_lines_desc | text | none | false | Zeilen umgekehrt alphabetisch |
| sort_lines_by_length | text | none | false | Zeilen nach Länge sortieren (kürzeste zuerst) |
| reverse_lines | text | none | false | Zeilenreihenfolge umkehren |
| unique_lines | text | medium | false | Doppelte Zeilen entfernen (behält erste Occurrence) |
| shuffle_lines | text | none | false | Zeilen zufällig mischen |
| add_line_numbers | text | low | false | "1. Zeile", "2. Zeile" voranstellen (Param: separator) |
| remove_line_numbers | text | none | false | Führende Nummern/Bullets entfernen |
| prefix_lines | text | none | false | Präfix vor jede nicht-leere Zeile (Param: prefix) |
| suffix_lines | text | none | false | Suffix an jede nicht-leere Zeile anfügen (Param: suffix) |
| indent | text | none | false | N Leerzeichen/Tabs einrücken (Param: n, char) |
| dedent | text | none | false | Gemeinsamen führenden Whitespace entfernen |
| join_lines | text | medium | false | Zeilen mit Trennzeichen verbinden (Param: separator) |
| reverse_text | text | none | false | Zeichenkette zeichenweise umkehren |
| wrap_markdown_block | code | none | false | ```sprache ... ``` umhüllen (Param: language) |
| strip_markdown | code | low | false | Markdown-Syntax entfernen, Klartext behalten |
| markdown_to_html | code | none | false | Markdown in HTML rendern (sanitized) |
| strip_html_tags | code | low | false | HTML-Tags entfernen, Textinhalt behalten |
| pretty_json | code | none | false | JSON mit 2-Space-Indent formatieren |
| minify_json | code | high | false | JSON-Whitespace entfernen |
| minify_code | code | high | false | Kommentare/Leerzeilen entfernen (Param: lang) |
| extract_code_blocks | code | high | false | Alle ```...``` Code-Blöcke extrahieren |
| extract_errors | code | high | false | Exceptions, Stack Traces, Error-Zeilen filtern |
| extract_json_keys | code | high | false | Alle JSON-Schlüssel als Zeilenliste |
| flatten_json | code | none | false | Verschachteltes JSON → flache dot.notation |
| xml_pretty | code | none | false | XML mit Einrückung formatieren |
| xml_minify | code | high | false | XML-Whitespace komprimieren |
| remove_comments | code | low | false | Code-Kommentare entfernen (Param: lang) |
| escape_json_string | code | none | false | String für JSON-Wert escapen |
| unescape_json_string | code | none | false | JSON-String-Escapes auflösen |
| base64_encode | format | none | false | Base64-Kodierung |
| base64_decode | format | none | false | Base64-Dekodierung |
| url_encode | format | none | false | URL-Prozent-Kodierung (vollständig) |
| url_encode_component | format | none | false | Nur Sonderzeichen URL-kodieren |
| url_decode | format | none | false | URL-Prozent-Dekodierung |
| html_entity_encode | format | none | false | & < > " ' → HTML-Entities |
| html_entity_decode | format | none | false | HTML-Entities → Klartextzeichen |
| hash_sha256 | format | none | false | SHA-256-Hash des Inhalts als Hex-String |
| camel_to_snake | format | none | false | camelCase → snake_case |
| snake_to_camel | format | none | false | snake_case → camelCase |
| snake_to_pascal | format | none | false | snake_case → PascalCase |
| to_slug | format | none | false | URL-freundlicher Slug (Kleinbuchstaben, Bindestriche) |
| to_kebab_case | format | none | false | Beliebig → kebab-case |
| to_constant_case | format | none | false | Beliebig → SCREAMING_SNAKE_CASE |
| csv_to_json | format | none | false | CSV mit Header-Zeile → JSON-Array |
| json_to_csv | format | none | false | JSON-Array → CSV mit Header |
| json_to_yaml | format | none | false | JSON → YAML |
| yaml_to_json | format | none | false | YAML → JSON |
| table_to_markdown | format | none | false | Leerzeichen-Tabelle → Markdown-Tabelle |
| align_columns | format | none | false | Spalten durch Padding ausrichten (Param: separator) |
| extract_emails | analysis | high | false | Alle E-Mail-Adressen extrahieren |
| extract_urls | analysis | high | false | Alle URLs extrahieren |
| extract_numbers | analysis | high | false | Alle Zahlen extrahieren (Param: includeDecimals) |
| extract_markdown_headings | analysis | high | false | Alle # Überschriften als Liste |
| extract_yaml_frontmatter | analysis | high | false | YAML-Frontmatter (zwischen ---) extrahieren |
| extract_json_values | analysis | high | false | Alle JSON-Werte (ohne Keys) extrahieren |
| estimate_tokens | analysis | none | false | Token-Schätzung als Kommentar anfügen |
| with_stats | analysis | none | false | Kurz-Statistiken (Zeichen, Wörter, Zeilen) anfügen |
| with_full_stats | analysis | none | false | Vollständige TextStats anfügen |
| count_occurrences | analysis | none | false | Anzahl Regex-Treffer zählen (Param: pattern) |
| redact_sensitive | security | low | **true** | IPs, E-Mails, API-Keys, Tokens maskieren |
| strip_pii | security | low | **true** | Alle PII vollständig entfernen (IPs, E-Mails, Tel, Karten) |
| fill_template | template | none | false | {{variable}} mit params-Werten füllen |
```
