# TextForge — Interface-Spezifikation v2.2 (Addendum zu v2.1)
## Erweiterung: Reiter/Collections, Tag-Suche, Sequenz-Kombination, CopyQ-Parität

> **Geltungsbereich:** Dieses Dokument ist eine **additive Erweiterung** von `SPECIFICATION.1.md` (v2.1). Es verändert keine bestehenden Typen, sondern ergänzt neue Abschnitte und erweitert punktuell bestehende Interfaces um optionale bzw. rückwärtskompatible Felder. Alle v2.1-Typen bleiben gültig.
>
> **Anlass:** Nutzer-Feedback und Vergleich mit CopyQ (etabliertester Open-Source-Clipboard-Manager, GPL-3.0, `github.com/hluk/CopyQ`) zeigen drei strukturelle Lücken gegenüber dem Marktstandard:
> 1. Es gibt keine frei benennbaren **Reiter/Sammlungen** (CopyQ: "Tabs") zur Organisation jenseits von Ordnern/Inbox/Archiv.
> 2. Tags existieren als Filter-Kriterium, aber es fehlt eine **dedizierte, eigenständige Tag-Suche** mit Autovervollständigung, Tag-Verwaltung und Tag-Umbenennung/-Zusammenführung.
> 3. Es gibt keine Möglichkeit, **mehrere Elemente in einer selbst gewählten Reihenfolge zu kombinieren** (Snippets *und* Clipboard-Einträge gemischt) — das ist in CopyQ über "Multi-Paste"/Sequenzen abgedeckt und bei TextForge bisher nur rudimentär über Bulk-Operationen auf *gleichartige* Elemente vorhanden.
>
> Neue Abschnitte sind mit `[NEU — v2.2]` markiert. Geänderte Abschnitte mit `[GEÄNDERT — v2.2]`. Bei Konflikten mit v2.1 gilt v2.2.

---

## § 0.1 — EINORDNUNG IN DIE ENTWICKLUNGSREIHENFOLGE [NEU — v2.2]

```
Diese Erweiterung baut auf einer bereits funktionsfähigen PHASE 1–4 auf (siehe § 0 in v2.1).
Empfohlene Einordnung als PHASE 5:

PHASE 5 — Organisation & Kombination (§ 22 Collections, § 23 Tag-Management,
           § 24 Sequenz-Engine, § 25 CopyQ-Paritätsfeatures)
  □ CollectionTab-Entity + IPC + Frontend-Reiterleiste
  □ TagRegistry (zentrale Tag-Verwaltung, Rename/Merge/Delete)
  □ Erweiterte Tag-Suche (Autocomplete, Tag-Browser, Tag-Kombinatorik)
  □ Sequence-Entity: geordnete Kombination beliebiger Item-Referenzen
  □ Globale Suche über Snippets + Clipboard + Scripts + Pipelines gleichzeitig
  □ CopyQ-Paritätsfeatures: Notizen, Ignore-Regeln, Multi-Paste, MIME-Erhalt, CLI
  → Deliverable: Nutzer kann Elemente in Reitern organisieren, per Tag durchsuchen,
    beliebig kombinieren/anordnen und die App vollständig per Tastatur/CLI bedienen.

WICHTIG: Diese Phase setzt funktionierendes Undo/Redo (§ 9) und die Workspace-Session
(§ 16) voraus, da Reiter- und Sequenz-Zustand Teil der Session sind.
```

---

## § 22 — REITER / COLLECTIONS (Tabs) [NEU — v2.2]

### 22.1 Motivation und Abgrenzung zu bestehenden Konzepten

TextForge kennt in v2.1 bereits drei Organisationsformen, die aber unterschiedliche Zwecke erfüllen und **nicht** das ersetzen, was CopyQ "Tabs" nennt:

| Bestehendes Konzept (v2.1) | Zweck | Warum es keine "Reiter" sind |
|---|---|---|
| `SnippetLocation` (`inbox`/`archive`/`trash`/`folder`) | Genau ein exklusiver Aufbewahrungsort pro Snippet | Ein Snippet kann nur an einem Ort sein — keine Mehrfachzugehörigkeit |
| `Folder` (hierarchisch, `parentId`) | Baumstruktur für Snippets | Nur für Snippets, nicht für Clipboard-Einträge, Scripts oder Pipelines |
| `EditorTab` (§ 16, `openEditorTabs`) | Offene Bearbeitungsfenster in der UI | Rein UI-transient, keine persistente Sammlung von Inhalten |
| `TagName` (Tags) | Nicht-exklusive Kennzeichnung | Kein Container mit eigener Reihenfolge/Ansicht — nur Filterkriterium |

**Neu:** `CollectionTab` — ein benutzerdefinierter, beliebig benennbarer Reiter (analog CopyQ-Tabs), der als **Sichtfenster mit eigener Mitgliederliste** funktioniert. Ein Item (Snippet, Clipboard-Eintrag, Script oder Pipeline) kann in **null, einem oder mehreren** Reitern gleichzeitig erscheinen — das ist der zentrale Unterschied zu `Folder`/`SnippetLocation`, die exklusiv sind.

### 22.2 CollectionTab — Entity

```typescript
// [DDD] CollectionTab ist ein eigenständiges Aggregate, referenziert Items nur per ID
interface CollectionTab {
  readonly id:          CollectionTabId;
  readonly name:        string;              // 1–48 Zeichen, z. B. "Arbeit", "Screenshots", "Später lesen"
  readonly icon:        Option<string>;      // Emoji oder Icon-Name
  readonly color:       Option<string>;      // Hex-Farbe (#RRGGBB) für die Reiterleiste
  readonly sortOrder:   number;               // Position in der Reiterleiste
  readonly kind:        CollectionKind;
  readonly isPinned:    boolean;              // Immer sichtbar, auch bei vielen Reitern (Überlauf-Menü)
  readonly createdAt:   UnixMs;
  readonly updatedAt:   UnixMs;
  // Abgeleitet, nicht gespeichert:
  readonly itemCount:   number;
}

// [FP-Scala] Ein Reiter ist entweder eine manuell kuratierte Liste ODER
// eine gespeicherte Suche/Filter ("Smart Tab" — CopyQ nennt das sinngemäß
// über Tabs mit aktiven Filtern; hier als eigener Typ explizit gemacht)
type CollectionKind =
  | { readonly _type: 'manual' }                                   // Mitglieder explizit hinzugefügt/entfernt
  | { readonly _type: 'smart'; readonly filter: UnifiedItemFilter } // Mitglieder = Live-Query-Ergebnis
  | { readonly _type: 'clipboard_capture' };                        // Spezialfall: neue Clipboard-Einträge
                                                                     // landen automatisch hier (CopyQ-Verhalten:
                                                                     // "Kopieren in bestimmten Fenstern → in Tab X")

// Invarianten:
// INVARIANT-CT1: name.length ∈ [1, 48]
// INVARIANT-CT2: Mind. 1 CollectionTab existiert immer ("Alle" / Default, nicht löschbar, id = 'default')
// INVARIANT-CT3: kind === 'smart' → filter darf keine Seiteneffekte auslösen (reine Query)
// INVARIANT-CT4: Maximal 32 CollectionTabs (UI-Grenze, Reiterleiste bleibt bedienbar)

interface CollectionTabMember {
  readonly tabId:     CollectionTabId;
  readonly itemRef:   ItemRef;          // siehe § 24.2 — polymorphe Referenz auf Snippet/Clipboard/Script/Pipeline
  readonly addedAt:   UnixMs;
  readonly sortOrder: number;           // manuelle Reihenfolge innerhalb des Reiters (nur bei kind='manual' relevant)
}

const CollectionTab = {
  create: (draft: { name: string; icon?: string; color?: string; kind?: CollectionKind }): Result<DomainError, CollectionTab> => {
    if (draft.name.trim().length === 0) return Result.err({ code: 'EMPTY_TITLE' });
    if (draft.name.length > 48)         return Result.err({ code: 'TITLE_TOO_LONG', max: 48 });
    const now = Date.now() as UnixMs;
    return Result.ok({
      id: CollectionTabId.of(crypto.randomUUID()),
      name: draft.name.trim(),
      icon: Option.fromNullable(draft.icon),
      color: Option.fromNullable(draft.color),
      sortOrder: 0,
      kind: draft.kind ?? { _type: 'manual' },
      isPinned: false,
      createdAt: now, updatedAt: now,
      itemCount: 0,
    });
  },
  // [v2.2] Reihenfolge zweier Reiter tauschen (Drag&Drop in der Reiterleiste)
  reorder: (tabs: ReadonlyArray<CollectionTab>, fromIndex: number, toIndex: number): ReadonlyArray<CollectionTab> => {
    const arr = [...tabs];
    const [moved] = arr.splice(fromIndex, 1);
    arr.splice(toIndex, 0, moved);
    return arr.map((t, i) => ({ ...t, sortOrder: i }));
  },
} as const;
```

### 22.3 IPC-Commands für Collections [NEU — v2.2]

```rust
#[tauri::command]
pub async fn list_collection_tabs(state: State<'_, AppState>) -> Result<Vec<CollectionTabDto>, String>;

#[tauri::command]
pub async fn create_collection_tab(draft: CreateCollectionTabDto, state: State<'_, AppState>) -> Result<CollectionTabDto, String>;

#[tauri::command]
pub async fn update_collection_tab(id: String, patch: CollectionTabPatchDto, state: State<'_, AppState>) -> Result<CollectionTabDto, String>;

#[tauri::command]
pub async fn delete_collection_tab(id: String, state: State<'_, AppState>) -> Result<(), String>;
// Löscht NUR den Reiter (Mitgliedschaften), nie die referenzierten Items selbst.

#[tauri::command]
pub async fn reorder_collection_tabs(orderedIds: Vec<String>, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
pub async fn add_item_to_tab(tabId: String, itemRef: ItemRefDto, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
pub async fn remove_item_from_tab(tabId: String, itemRef: ItemRefDto, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
pub async fn reorder_tab_members(tabId: String, orderedItemRefs: Vec<ItemRefDto>, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
pub async fn list_tab_members(tabId: String, page: u32, pageSize: u32, state: State<'_, AppState>) -> Result<PagedResultDto<UnifiedItemListItemDto>, String>;
// Bei kind='smart': führt filter live gegen die Datenbank aus (kein Materialize).
// Bei kind='manual'/'clipboard_capture': liest collection_tab_members, sortiert nach sortOrder.

#[tauri::command]
pub async fn move_items_between_tabs(fromTabId: String, toTabId: String, itemRefs: Vec<ItemRefDto>, keepInSource: bool, state: State<'_, AppState>) -> Result<(), String>;
```

### 22.4 Frontend: Reiterleiste

```
Neue Komponenten (Ergänzung zu § 20.2):
├── components/collections/                    # [NEU v2.2]
│   ├── CollectionTabBar.svelte     # Horizontale Reiterleiste, Drag&Drop-Reorder, Überlauf-Menü ab 8 Reitern
│   ├── CollectionTabView.svelte    # Zeigt Mitglieder des aktiven Reiters (manual/smart/clipboard_capture)
│   ├── CollectionTabEditor.svelte  # Name/Icon/Farbe/Kind bearbeiten
│   ├── AddToTabMenu.svelte         # Kontextmenü-Eintrag: "Zu Reiter hinzufügen ▸"
│   └── SmartTabFilterEditor.svelte # Filter-Builder für kind='smart' (nutzt UnifiedItemFilter § 23.4)

Neue Route:
routes/
└── collections/
    └── [tabId]/+page.svelte   # Reiter-Detailansicht: Mitgliederliste + Reihenfolge editierbar
```

Verhaltensspezifikation (analog CopyQ, an TextForge-Domänenmodell angepasst):

- Die Reiterleiste ist horizontal oben in der Hauptansicht sichtbar, unmittelbar über der Element-Liste — **nicht** eine zusätzliche Sidebar-Ebene, um mit einem Klick zwischen Sammlungen zu wechseln (CopyQ-Kernverhalten: "instant switch between categories").
- `Strg+1` bis `Strg+9` springen zu Reiter 1–9 (siehe § 26 Shortcuts-Erweiterung).
- Rechtsklick auf ein beliebiges Listenelement (Snippet, Clipboard-Eintrag, Script, Pipeline) öffnet `AddToTabMenu` mit Checkbox-Liste aller vorhandenen Reiter plus "Neuer Reiter …".
- Der Default-Reiter `'default'` (nicht löschbar, INVARIANT-CT2) zeigt weiterhin die bisherige unveränderte Snippet-Liste — v2.1-Verhalten bleibt für Nutzer ohne konfigurierte Reiter identisch.
- `kind: 'clipboard_capture'`-Reiter erlauben pro Reiter eine Zuordnungsregel (`sourceApp`-Muster), sodass z. B. alles aus dem Terminal automatisch in einen "Shell"-Reiter läuft — direktes Pendant zu CopyQs fensterbasierter Tab-Zuordnung.

### 22.5 SQLite-Schema-Ergänzung [NEU — v2.2]

```sql
-- 0XX_collection_tabs.sql (append-only, INVARIANT-F beachten)

CREATE TABLE IF NOT EXISTS collection_tabs (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  icon         TEXT,
  color        TEXT CHECK(color GLOB '#??????' OR color IS NULL),
  sort_order   INTEGER NOT NULL DEFAULT 0,
  kind         TEXT NOT NULL DEFAULT 'manual',   -- 'manual' | 'smart' | 'clipboard_capture'
  kind_config  TEXT,                              -- JSON: UnifiedItemFilter bei 'smart', sourceApp-Muster bei 'clipboard_capture'
  is_pinned    INTEGER NOT NULL DEFAULT 0,
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collection_tab_members (
  tab_id       TEXT NOT NULL REFERENCES collection_tabs(id) ON DELETE CASCADE,
  item_kind    TEXT NOT NULL,   -- 'snippet' | 'clipboard' | 'script' | 'pipeline'
  item_id      TEXT NOT NULL,
  added_at     INTEGER NOT NULL,
  sort_order   INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (tab_id, item_kind, item_id)
);

CREATE INDEX IF NOT EXISTS idx_tab_members_tab   ON collection_tab_members(tab_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_tab_members_item  ON collection_tab_members(item_kind, item_id);

-- Default-Reiter wird bei Migration einmalig angelegt (INVARIANT-CT2):
INSERT OR IGNORE INTO collection_tabs (id, name, sort_order, kind, is_pinned, created_at, updated_at)
VALUES ('default', 'Alle', 0, 'manual', 1, strftime('%s','now')*1000, strftime('%s','now')*1000);
```

---

## § 23 — TAG-SYSTEM & TAG-BASIERTE SUCHE [ERWEITERT — v2.2]

### 23.1 Ist-Zustand in v2.1 und Lücke

In v2.1 sind Tags bereits Teil von `Snippet.tags`, `SnippetFilter.tags`/`tagsMode` und `ClipboardFilter.tags`. Das reicht als **Filterkriterium**, aber es fehlt:

1. Eine zentrale **Tag-Registry** — Tags existieren nur implizit als Strings auf Items verstreut; es gibt keine Liste "alle jemals benutzten Tags", keine Nutzungszahl, keine Umbenennung, kein Zusammenführen ähnlicher Tags (z. B. "Todo" und "todo" versehentlich als zwei verschiedene Tags).
2. Eine **eigenständige Tag-Such-UI** mit Autovervollständigung während der Eingabe, statt nur Chip-Auswahl aus bereits gefilterten Ergebnissen.
3. **Tag-übergreifende Suche über alle Item-Typen** (Snippets *und* Clipboard-Einträge *und* Pipelines *und* Scripts gemeinsam) — bisher sind `SnippetFilter.tags` und `ClipboardFilter.tags` getrennte, inkompatible Felder.

### 23.2 TagRegistry — Entity [NEU — v2.2]

```typescript
interface TagInfo {
  readonly name:        TagName;
  readonly color:       Option<string>;         // Optionale visuelle Kennzeichnung, wie CopyQ-Labels
  readonly usageCount:  number;                  // Über alle Item-Typen hinweg — Derived
  readonly lastUsedAt:  UnixMs;                  // Derived
  readonly createdAt:   UnixMs;
}

const TagRegistry = {
  // [FP-Scala] Normalisierung ist eine reine, totale Funktion — Groß/Kleinschreibung
  // und Whitespace-Varianten dürfen nicht zu Tag-Duplikaten führen
  normalize: (raw: string): Result<DomainError, TagName> => {
    const trimmed = raw.trim().toLowerCase().replace(/\s+/g, '-');
    if (trimmed.length === 0)  return Result.err({ code: 'EMPTY_TAG' });
    if (trimmed.length > 32)   return Result.err({ code: 'TAG_TOO_LONG', max: 32 });
    if (!/^[\p{L}\p{N}_-]+$/u.test(trimmed)) return Result.err({ code: 'INVALID_TAG_CHARS' });
    return Result.ok(TagName.of(trimmed));
  },
} as const;
```

### 23.3 IPC-Commands für Tag-Verwaltung [NEU — v2.2]

```rust
#[tauri::command]
pub async fn list_all_tags(state: State<'_, AppState>) -> Result<Vec<TagInfoDto>, String>;
// Aggregiert Tags über snippets, clipboard_history, scripts, pipelines hinweg (UNION + GROUP BY).

#[tauri::command]
pub async fn suggest_tags(prefix: String, limit: u32, state: State<'_, AppState>) -> Result<Vec<TagInfoDto>, String>;
// Autocomplete: Präfix-Suche über tag-Index, sortiert nach usage_count DESC.

#[tauri::command]
pub async fn rename_tag(oldName: String, newName: String, state: State<'_, AppState>) -> Result<TagRenameResultDto, String>;
// Ersetzt oldName durch newName auf ALLEN Item-Typen. Falls newName bereits existiert: Merge (siehe merge_tags).

#[tauri::command]
pub async fn merge_tags(sourceTags: Vec<String>, targetTag: String, state: State<'_, AppState>) -> Result<TagMergeResultDto, String>;
// Führt mehrere Tags zu einem zusammen (z. B. "todo" + "Todo" + "to-do" → "todo").
// Erzeugt EINEN UndoEntry vom Typ 'tag_merge' mit vollständiger Rückabbildung.

#[tauri::command]
pub async fn delete_tag_everywhere(tagName: String, state: State<'_, AppState>) -> Result<u32, String>;
// Entfernt den Tag von allen Items, löscht aber keine Items. Rückgabe: Anzahl betroffener Items.

#[tauri::command]
pub async fn set_tag_color(tagName: String, color: Option<String>, state: State<'_, AppState>) -> Result<(), String>;
```

### 23.4 UnifiedItemFilter — Item-Typ-übergreifende Suche [NEU — v2.2]

```typescript
// [PragProg] DRY: Statt getrennter SnippetFilter/ClipboardFilter-Suchpfade
// existiert jetzt ein gemeinsamer Filter für "durchsuche alles". SnippetFilter und
// ClipboardFilter (§ 5.1, § 5.2) bleiben für item-typ-spezifische Ansichten bestehen
// und unverändert — UnifiedItemFilter ist eine zusätzliche, orthogonale Fähigkeit.
interface UnifiedItemFilter {
  readonly searchQuery:  Option<string>;
  readonly itemKinds:    ReadonlyArray<ItemKind>;   // leer = alle: snippet, clipboard, script, pipeline
  readonly tags:         ReadonlyArray<TagName>;
  readonly tagsMode:     'all' | 'any' | 'none';    // [v2.2] 'none' neu: "keine dieser Tags" — CopyQ-Exclude-Pattern
  readonly collectionTabId: Option<CollectionTabId>; // Auf einen Reiter eingeschränkt
  readonly dateRange:    Option<DateRangeFilter>;
  readonly contentTypes: ReadonlyArray<ContentType>;
  readonly sortBy:       'relevance' | 'updatedAt' | 'title';
  readonly sortDir:      'asc' | 'desc';
}

type ItemKind = 'snippet' | 'clipboard' | 'script' | 'pipeline';

const UnifiedItemFilter = {
  default: (): UnifiedItemFilter => ({
    searchQuery: Option.none(), itemKinds: [], tags: [], tagsMode: 'all',
    collectionTabId: Option.none(), dateRange: Option.none(), contentTypes: [],
    sortBy: 'relevance', sortDir: 'desc',
  }),
  byTag: (tag: TagName): Partial<UnifiedItemFilter> => ({ tags: [tag], tagsMode: 'any' }),
  byTags: (tags: ReadonlyArray<TagName>, mode: 'all' | 'any' = 'all'): Partial<UnifiedItemFilter> => ({ tags, tagsMode: mode }),
} as const;
```

### 23.5 IPC: Globale Suche [NEU — v2.2]

```rust
#[tauri::command]
pub async fn search_all_items(filter: UnifiedItemFilterDto, page: u32, pageSize: u32, state: State<'_, AppState>)
    -> Result<PagedResultDto<UnifiedItemListItemDto>, String>;
// Implementierung: UNION ALL über snippets_fts, clipboard_fts, scripts_fts (§ 24.6 neu),
// pipelines (LIKE-Fallback, da Pipelines i. d. R. wenige sind) — Ergebnisse tragen
// je einen `matchScore` und werden gemeinsam nach `sortBy` sortiert.
// Query-Highlighting: Rückgabe enthält `highlightedSnippet: string` mit <mark>-Tags
// um Treffer, analog CopyQ "filtering with matched text highlighting".

interface UnifiedItemListItemDto {
  itemKind:            ItemKind;
  id:                  string;
  title:               string;
  preview:             string;
  highlightedPreview:  string;      // [v2.2] <mark>Treffer</mark>-markiert
  tags:                string[];
  contentType:         string | null;   // null bei Script/Pipeline
  updatedAt:            number;
  matchScore:          number | null;
}
```

### 23.6 Frontend: Tag-Browser & globale Suche

```
Neue Komponenten:
├── components/tags/                          # [NEU v2.2]
│   ├── TagBrowser.svelte        # Alle Tags als Cloud/Liste, Klick = Filter, Größe ~ usageCount
│   ├── TagAutocomplete.svelte   # Ersetzt/erweitert TagInput.svelte um Live-Vorschläge (suggest_tags)
│   ├── TagManageDialog.svelte   # Umbenennen, Zusammenführen, Farbe setzen, Löschen
│   └── TagChip.svelte           # Einzelnes Tag mit Farbe + Klick-Aktionen (Filter/Entfernen)
│
├── components/search/                        # [NEU v2.2]
│   ├── GlobalSearchBar.svelte   # Immer sichtbar (Kopfzeile), durchsucht ALLE Item-Typen
│   ├── GlobalSearchResults.svelte # Gruppiert nach ItemKind, mit Highlighting
│   └── SearchResultRow.svelte   # Eine Trefferzeile inkl. highlightedPreview
```

Verhaltensspezifikation:

- `Strg+F` fokussiert je nach Kontext entweder die lokale Listen-Suche (bestehendes v2.1-Verhalten) oder — bei erneutem `Strg+F` innerhalb 1s — die `GlobalSearchBar` (Doppel-Trigger, CopyQ-inspiriert: **F3** öffnet dort die globale Suche unabhängig vom Kontext).
- Tag-Chips sind überall klickbar identisch eingefärbt (Farbe aus `TagInfo.color`, falls gesetzt) — konsistente visuelle Sprache analog CopyQ-Labels.
- `TagBrowser` unterstützt Mehrfachauswahl mit AND/OR/NOT-Umschalter (`tagsMode`) direkt in der UI, keine separate Modal nötig.

### 23.7 Schema-Ergänzung

```sql
-- 0XX_tag_registry.sql
CREATE TABLE IF NOT EXISTS tag_colors (
  tag_name    TEXT PRIMARY KEY,   -- normalisierter Tagname (siehe TagRegistry.normalize)
  color       TEXT CHECK(color GLOB '#??????' OR color IS NULL),
  created_at  INTEGER NOT NULL
);
-- usageCount/lastUsedAt werden zur Laufzeit per UNION-Query über
-- snippet_tags, clipboard_tags, script_tags (neu), pipeline_tags (neu) berechnet,
-- NICHT redundant gespeichert (INVARIANT-Konsistenz: keine Doppelquelle der Wahrheit).

CREATE TABLE IF NOT EXISTS script_tags (
  script_id  TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
  tag        TEXT NOT NULL,
  PRIMARY KEY (script_id, tag)
);
CREATE TABLE IF NOT EXISTS pipeline_tags (
  pipeline_id TEXT NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
  tag         TEXT NOT NULL,
  PRIMARY KEY (pipeline_id, tag)
);
```

---

## § 24 — SEQUENZ-ENGINE: ELEMENTE IN REIHENFOLGE KOMBINIEREN [NEU — v2.2]

### 24.1 Motivation

Bisher (v2.1) gibt es Bulk-Operationen (§ 13) und Pipelines (§ 4) — beide wirken aber **auf einen einzelnen Text bzw. auf mehrere gleichartige Snippets unabhängig voneinander**. Es fehlt die Fähigkeit, **mehrere unterschiedliche Elemente — Snippets, Clipboard-Einträge, sogar Template-Ausgaben — in einer selbst gewählten Reihenfolge zu einem einzigen Ergebnistext zu kombinieren.** Das ist das CopyQ-Pendant zu "Multi-Paste" (mehrere Items nacheinander einfügen) — TextForge erweitert das Konzept jedoch um Persistenz, Trennzeichen-Kontrolle und optionale Pipeline-Anwendung pro Element.

Typische Anwendungsfälle:
- Mehrere Code-Snippets in fester Reihenfolge zu einer Datei zusammensetzen.
- Eine E-Mail aus wiederverwendbaren Textbausteinen (Begrüßung + Hauptteil + Signatur) zusammenstellen, wobei jeder Baustein ein eigenes Snippet bleibt.
- Mehrere zuletzt kopierte Clipboard-Einträge in der Reihenfolge des Kopierens (oder einer manuell geänderten Reihenfolge) als einen zusammenhängenden Text exportieren.

### 24.2 ItemRef — polymorphe Referenz [NEU — v2.2]

```typescript
// [FP-Scala] Discriminated Union statt (kind: string, id: string) — verhindert
// unmögliche Zustände (z. B. kind='snippet' mit einer Clipboard-UUID)
type ItemRef =
  | { readonly _type: 'snippet';   readonly id: SnippetId }
  | { readonly _type: 'clipboard'; readonly id: ClipboardEntryId }
  | { readonly _type: 'script_output'; readonly scriptId: ScriptId; readonly cachedOutput: Option<string> }
  | { readonly _type: 'literal';   readonly text: string };  // Freitext-Baustein direkt in der Sequenz

const ItemRef = {
  resolveContent: async (ref: ItemRef, deps: ResolveDeps): Promise<Result<DomainError, string>> => {
    switch (ref._type) {
      case 'snippet':   return deps.getSnippetContent(ref.id);
      case 'clipboard': return deps.getClipboardContent(ref.id);
      case 'script_output': return ref.cachedOutput._tag === 'Some'
        ? Result.ok(ref.cachedOutput.value)
        : deps.runScriptForOutput(ref.scriptId);
      case 'literal':   return Result.ok(ref.text);
    }
  },
} as const;
```

### 24.3 Sequence — Entity

```typescript
interface Sequence {
  readonly id:          SequenceId;
  readonly name:        string;                          // 1–128 Zeichen
  readonly items:        ReadonlyArray<SequenceItem>;      // Reihenfolge = items[].order
  readonly separator:    SequenceSeparator;
  readonly tags:         ReadonlyArray<TagName>;
  readonly favorite:     boolean;
  readonly createdAt:    UnixMs;
  readonly updatedAt:    UnixMs;
  // Ergebnis-Cache (optional, nicht Wahrheit — jederzeit neu berechenbar):
  readonly lastRenderedAt: Option<UnixMs>;
}

interface SequenceItem {
  readonly id:            string;              // stabile ID innerhalb der Sequenz (für Reorder/Undo)
  readonly order:         number;
  readonly ref:           ItemRef;
  readonly pipelineId:    Option<PipelineId>;   // Optional: Element wird vor Einfügen transformiert
  readonly prefixOverride: Option<string>;      // Element-spezifisches Präfix (überschreibt separator.prefix)
  readonly suffixOverride: Option<string>;
  readonly enabled:       boolean;              // Kann deaktiviert werden ohne Entfernen (wie PipelineStep.enabled)
}

type SequenceSeparator =
  | { readonly _type: 'none' }
  | { readonly _type: 'newline';       readonly count: number }        // N Leerzeilen zwischen Elementen
  | { readonly _type: 'custom';        readonly text: string }         // z. B. "\n---\n"
  | { readonly _type: 'numbered_list' }                                // "1. ", "2. " automatisch
  | { readonly _type: 'markdown_section' };                            // "## <SnippetTitel>\n\n<Inhalt>"

// Invarianten:
// INVARIANT-SQ1: name.length ∈ [1, 128]
// INVARIANT-SQ2: items.length ∈ [1, 100]  — Obergrenze gegen versehentliche Mammut-Sequenzen
// INVARIANT-SQ3: items[].order ist eindeutig und lückenlos 0..n-1 nach jeder Mutation

const Sequence = {
  create: (draft: { name: string; items?: ReadonlyArray<Omit<SequenceItem, 'order'>>; separator?: SequenceSeparator }): Result<DomainError, Sequence> => { /* ... */ },

  // [Immutability] Reihenfolge ändern erzeugt neues Objekt, reindiziert order lückenlos
  reorderItems: (seq: Sequence, fromIndex: number, toIndex: number): Sequence => {
    const items = [...seq.items];
    const [moved] = items.splice(fromIndex, 1);
    items.splice(toIndex, 0, moved);
    return { ...seq, items: items.map((it, i) => ({ ...it, order: i })), updatedAt: Date.now() as UnixMs };
  },

  addItem: (seq: Sequence, ref: ItemRef, atIndex?: number): Result<DomainError, Sequence> => {
    if (seq.items.length >= 100) return Result.err({ code: 'TOO_MANY_TAGS', max: 100 }); // Wiederverwendung des Fehlercodes-Musters; ggf. eigener Code 'SEQUENCE_FULL'
    /* ... Einfügen an atIndex ?? Ende, order neu berechnen ... */
    return Result.ok(seq /* aktualisiert */);
  },

  removeItem: (seq: Sequence, itemId: string): Sequence => ({
    ...seq,
    items: seq.items.filter(it => it.id !== itemId).map((it, i) => ({ ...it, order: i })),
    updatedAt: Date.now() as UnixMs,
  }),
} as const;
```

### 24.4 Rendering — reine Funktion + Effect-Shell-Ausführung

```typescript
// Reine Domain-Funktion: nimmt bereits AUFGELÖSTE Inhalte entgegen (Strings),
// keine IPC/DB-Zugriffe hier — das Auflösen von ItemRef.resolveContent geschieht
// in der Application-Schicht (transform.rs / sequences.rs), NICHT im Domain Core.
const renderSequence = (
  items: ReadonlyArray<{ content: string; item: SequenceItem }>,
  separator: SequenceSeparator,
): string => {
  const parts = items.filter(x => x.item.enabled).map(({ content, item }, idx) => {
    const prefix = item.prefixOverride._tag === 'Some' ? item.prefixOverride.value : separatorPrefixFor(separator, idx);
    const suffix = item.suffixOverride._tag === 'Some' ? item.suffixOverride.value : '';
    return `${prefix}${content}${suffix}`;
  });
  return parts.join(separatorJoinFor(separator));
};
```

### 24.5 IPC-Commands [NEU — v2.2]

```rust
#[tauri::command]
pub async fn list_sequences(state: State<'_, AppState>) -> Result<Vec<SequenceDto>, String>;

#[tauri::command]
pub async fn create_sequence(draft: CreateSequenceDto, state: State<'_, AppState>) -> Result<SequenceDto, String>;

#[tauri::command]
pub async fn update_sequence(id: String, patch: SequencePatchDto, state: State<'_, AppState>) -> Result<SequenceDto, String>;

#[tauri::command]
pub async fn delete_sequence(id: String, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
pub async fn reorder_sequence_items(id: String, fromIndex: u32, toIndex: u32, state: State<'_, AppState>) -> Result<SequenceDto, String>;

#[tauri::command]
pub async fn render_sequence(id: String, state: State<'_, AppState>) -> Result<SequenceRenderResultDto, String>;
// Löst alle ItemRefs auf (DB-Reads), wendet ggf. pipelineId pro Element an
// (Wiederverwendung von run_pipeline, siehe P1-Fix "BulkTransform" in der Lückenanalyse —
// dieselbe extrahierte Funktion wird hier ein drittes Mal genutzt: Pipeline, Bulk, Sequenz),
// rendert über renderSequence(), gibt Endergebnis + Einzelergebnisse zurück.

#[tauri::command]
pub async fn quick_combine(itemRefs: Vec<ItemRefDto>, separator: SequenceSeparatorDto, state: State<'_, AppState>) -> Result<String, String>;
// Ad-hoc-Kombination OHNE eine Sequence-Entity anzulegen — für den schnellen
// "markiere 3 Elemente in der Liste → Rechtsklick → Kombinieren" Anwendungsfall.
// Persistiert nichts; das Ergebnis kann der Nutzer per "Als Sequenz speichern" später sichern.

interface SequenceRenderResultDto {
  finalOutput:   string;
  itemResults:   { itemId: string; resolvedPreview: string; error: string | null }[];
  warnings:      string[];   // z. B. "Element X wurde gelöscht, wurde übersprungen"
}
```

### 24.6 Frontend: Sequenz-Builder

```
Neue Komponenten:
├── components/sequences/                       # [NEU v2.2]
│   ├── SequenceList.svelte          # Alle gespeicherten Sequenzen
│   ├── SequenceBuilder.svelte       # Drag&Drop-Reihenfolge (analog PipelineEditor, aber
│   │                                 # für heterogene ItemRefs statt PipelineSteps)
│   ├── SequenceItemCard.svelte      # Einzelnes Element: Vorschau + Prefix/Suffix + Pipeline-Wahl
│   ├── SeparatorPicker.svelte       # Trennzeichen-Auswahl (none/newline/custom/numbered/markdown)
│   ├── SequencePreview.svelte       # Live-Vorschau des Endergebnisses (debounced render_sequence)
│   └── QuickCombineBar.svelte       # Erscheint bei Mehrfachauswahl in JEDER Liste (Snippet, Clipboard, gemischt)
```

Verhaltensspezifikation:

- `QuickCombineBar` erscheint sowohl in der Snippet-Liste als auch in der Clipboard-History als auch in der neuen `GlobalSearchResults`-Ansicht — Mehrfachauswahl über Item-Typ-Grenzen hinweg ist explizit möglich (Checkbox-Selektion bleibt beim Wechsel der Ansicht erhalten, gehalten in einem neuen `selectionStore: writable<ItemRef[]>`).
- Drag&Drop-Reihenfolge im `SequenceBuilder` nutzt dieselbe Interaktionslogik wie `PipelineEditor.svelte` (Wiederverwendung der bestehenden Drag-Drop-Utility, kein Duplikat).
- Nach `render_sequence` kann das Ergebnis wahlweise (a) in die Zwischenablage kopiert, (b) als neues Snippet gespeichert, oder (c) direkt als neuer Clipboard-Eintrag geschrieben werden — drei Buttons in `SequencePreview.svelte`.

### 24.7 Schema-Ergänzung

```sql
-- 0XX_sequences.sql
CREATE TABLE IF NOT EXISTS sequences (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  separator     TEXT NOT NULL,        -- JSON: SequenceSeparator
  favorite      INTEGER NOT NULL DEFAULT 0,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  last_rendered_at INTEGER
);

CREATE TABLE IF NOT EXISTS sequence_items (
  id              TEXT PRIMARY KEY,
  sequence_id     TEXT NOT NULL REFERENCES sequences(id) ON DELETE CASCADE,
  order_index     INTEGER NOT NULL,
  ref_type        TEXT NOT NULL,      -- 'snippet' | 'clipboard' | 'script_output' | 'literal'
  ref_id          TEXT,               -- NULL bei 'literal'
  literal_text    TEXT,               -- NULL außer bei 'literal'
  pipeline_id     TEXT REFERENCES pipelines(id) ON DELETE SET NULL,
  prefix_override TEXT,
  suffix_override TEXT,
  enabled         INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_seq_items_seq ON sequence_items(sequence_id, order_index);

CREATE TABLE IF NOT EXISTS sequence_tags (
  sequence_id TEXT NOT NULL REFERENCES sequences(id) ON DELETE CASCADE,
  tag         TEXT NOT NULL,
  PRIMARY KEY (sequence_id, tag)
);
```

---

## § 25 — COPYQ-PARITÄTSFEATURES [NEU — v2.2]

Diese Features schließen die verbleibenden funktionalen Lücken gegenüber CopyQ, die nicht bereits durch § 22–24 abgedeckt sind. Jeder Punkt referenziert die konkrete CopyQ-Fähigkeit als Begründung.

### 25.1 Notizen an Items (CopyQ: "Add notes to items")

```typescript
// Erweiterung bestehender Typen (additiv, rückwärtskompatibel):
interface Snippet {
  // ... alle v2.1-Felder unverändert ...
  readonly note: Option<string>;   // [NEU v2.2] Freitext-Anmerkung, max 2000 Zeichen, getrennt vom content
}
interface ClipboardEntry {
  // ... alle v2.1-Felder unverändert ...
  readonly note: Option<string>;   // [NEU v2.2]
}
```
Notizen sind **nicht** Teil der FTS5-Volltextsuche über `content` (separates Feld), erscheinen aber optional als eigene durchsuchbare Spalte (`notes_fts`), damit "Suche in Notizen" gezielt an/ausschaltbar ist, ohne Haupttreffer zu verwässern.

**Auftrag:** `note`-Feld per Migration ergänzen (`ALTER TABLE snippets ADD COLUMN note TEXT`, analog für `clipboard_history`), `NoteEditor.svelte` als ausklappbares Feld im Editor, optionales `notes_fts`-Virtual-Table.

### 25.2 Ignore-Regeln für Clipboard-Erfassung (CopyQ: "Ignore clipboard copied from some windows or containing some text")

```typescript
interface ClipboardIgnoreRule {
  readonly id:        IgnoreRuleId;
  readonly enabled:   boolean;
  readonly matchType: 'source_app' | 'content_regex' | 'content_type';
  readonly pattern:   string;          // App-Name, Regex oder ContentType-Wert je nach matchType
  readonly createdAt: UnixMs;
}
```
Wird vom `ClipboardMonitor` vor dem Einfügen in `clipboard_history` geprüft (Effect-Shell, nicht Domain Core — reine Filterfunktion `shouldIgnore(entry, rules): boolean` bleibt aber testbar im Domain Core). Typischer Anwendungsfall: Passwortmanager-Fenster oder Inhalte, die dem Muster eines TOTP-Codes entsprechen, nie in der History landen zu lassen.

**IPC:** `list_ignore_rules`, `create_ignore_rule`, `update_ignore_rule`, `delete_ignore_rule`.
**Schema:** neue Tabelle `clipboard_ignore_rules`.

### 25.3 Multi-Paste / "Paste as plain text" (CopyQ: Systemweite Shortcuts, "paste as plain text")

Zusätzlich zur Sequenz-Engine (§ 24, für **gespeicherte** Kombinationen) unterstützt TextForge zwei sofortige Einfüge-Modi ohne vorherige Sequenzerstellung:

```rust
#[tauri::command]
pub async fn copy_to_clipboard_as_plain_text(itemRef: ItemRefDto, state: State<'_, AppState>) -> Result<(), String>;
// Entfernt Formatierung/Markdown-Syntax vor dem Schreiben in die System-Zwischenablage
// (nutzt bereits vorhandenes Builtin 'strip_markdown'/'strip_html_tags' intern).

#[tauri::command]
pub async fn paste_sequence_stepwise_next(sequenceId: String, state: State<'_, AppState>) -> Result<StepPasteResultDto, String>;
// CopyQ-Pendant zu "copy next/previous item": schreibt EIN Element der Sequenz nach dem
// anderen in die Zwischenablage bei wiederholtem Tastendruck, statt alles auf einmal zu kombinieren.
```

### 25.4 MIME-Type-/Formaterhalt (CopyQ: "Store text, HTML, images and any other custom format")

`ContentType` (§ 2.3) klassifiziert bereits nach Inhalt, speichert aber keine **Herkunfts-MIME-Information** aus der System-Zwischenablage. Erweiterung:

```typescript
interface ClipboardEntry {
  // ... v2.1-Felder unverändert ...
  readonly sourceMimeTypes: ReadonlyArray<string>;  // [NEU v2.2] z. B. ["text/html", "text/plain"] — alle vom OS angebotenen Formate
  readonly hasRichFormatting: boolean;               // [NEU v2.2] Derived aus sourceMimeTypes.includes('text/html')
}
```
Beim Schreiben zurück in die Zwischenablage (`clipboard write-back`, § 8.4) kann der Nutzer wählen: Original-Formatierung (falls `hasRichFormatting`) oder reiner Text — deckt CopyQs Kernversprechen "Formatierung bleibt erhalten oder wird gezielt entfernt" ab, ohne dass TextForge HTML-Inhalte selbst rendern/speichern muss (nur die MIME-Typ-Liste wird gehalten, der Rich-Content bleibt transient im OS-Clipboard-Buffer beim Zurückschreiben).

### 25.5 Kommandozeilen-Schnittstelle (CopyQ: "advanced command-line interface", `copyq add`, `copyq read 0`)

```
Neuer Tauri-Command-Line-Modus (analog CopyQ, aber an Tauri-IPC statt eigenem Server gebunden):

textforge-cli add "Text"                    → Fügt Text als neuen Clipboard-Eintrag hinzu
textforge-cli read <index|id>               → Gibt Inhalt eines Eintrags auf stdout aus
textforge-cli list --tag <tag>              → Listet IDs/Titel gefiltert nach Tag
textforge-cli tab <name> add <id>           → Fügt Item zu einem CollectionTab hinzu (§ 22)
textforge-cli sequence render <id>          → Rendert eine Sequenz (§ 24) und gibt Ergebnis aus
textforge-cli --help
```
Technisch: separates schlankes Rust-Binary im selben Cargo-Workspace (`src-tauri/src/bin/textforge-cli.rs`), das über eine lokale Unix-Domain-Socket-Verbindung mit der laufenden Tauri-App spricht (App muss laufen — identisch zu CopyQs Einschränkung: *"The main application must be running to be able to issue commands using the command line"*). Fällt die App nicht, degradiert der CLI-Befehl auf direkten SQLite-Zugriff (read-only Befehle wie `read`/`list` funktionieren dann auch ohne laufende App).

**Neue Invariante:** INVARIANT-I: CLI-Schreibbefehle (`add`, `tab … add`) erzeugen dieselben Undo-Einträge wie die entsprechende GUI-Aktion — keine Sonderpfade, die Undo umgehen.

### 25.6 Vim-artige Tastaturnavigation in Listen (CopyQ: "simple Vim-like editor with keyboard shortcuts")

Ergänzung zu § 11 (Shortcuts, v2.1) bzw. § 26 (Shortcut-Erweiterung, unten):

```
j / k          — Nächstes / vorheriges Element (Listen-Navigation)
g g / G        — Zum ersten / letzten Element springen
/              — Lokale Listen-Suche fokussieren (zusätzlich zu Strg+F)
d d            — Aktuelles Element löschen (mit Undo, wie jede andere Löschung)
y y            — Aktuelles Element in Zwischenablage kopieren ("yank")
Leertaste      — Mehrfachauswahl-Checkbox toggeln (für QuickCombineBar, § 24.6)
```
Dieser Modus ist **standardmäßig deaktiviert** (Einstellung `keyboard.vim_mode: boolean`, default `false`) und nur aktiv, wenn kein Eingabefeld fokussiert ist — verhindert Konflikte mit normaler Texteingabe.

---

## § 26 — SHORTCUT- UND SESSION-ERWEITERUNGEN [GEÄNDERT — v2.2]

### 26.1 Neue Shortcuts (Ergänzung der Tabelle aus § 11, v2.1)

```
Strg+1 .. Strg+9      — Zu CollectionTab 1–9 springen                         [NEU v2.2]
Strg+Shift+T          — Neuen CollectionTab erstellen                         [NEU v2.2]
Strg+Shift+F          — Globale Suche über alle Item-Typen fokussieren        [NEU v2.2]
F3                    — Globale Suche öffnen (CopyQ-Parität)                  [NEU v2.2]
Strg+G                — Ausgewählte Elemente kombinieren (QuickCombineBar)    [NEU v2.2]
Strg+Shift+G          — Kombination als neue Sequenz speichern                [NEU v2.2]
Alt+T                 — Tag-Browser öffnen/schließen                          [NEU v2.2]
Strg+Shift+V          — Als reiner Text einfügen (paste as plain text)        [NEU v2.2]
```

### 26.2 WorkspaceSession — Erweiterung [GEÄNDERT — v2.2]

```typescript
interface WorkspaceSession {
  // ... alle v2.1-Felder unverändert (activeView, lastActiveSnippetId, sidebarWidth,
  //     previewMode, filterState, openEditorTabs, savedAt, ...) ...

  // [NEU v2.2]
  readonly activeCollectionTabId: Option<CollectionTabId>;
  readonly collectionTabOrder:    ReadonlyArray<CollectionTabId>;   // Für den Fall, dass Reihenfolge
                                                                     // abweichend von sortOrder lokal gehalten wird
  readonly globalSearchOpen:      boolean;
  readonly selectedItemRefs:      ReadonlyArray<ItemRef>;           // Persistiert Mehrfachauswahl über Neustart
                                                                     // hinweg NICHT standardmäßig (Privacy — siehe unten),
                                                                     // nur wenn `session.persist_selection` = true in Settings
  readonly vimModeActive:         boolean;
}
```

> **Privacy-Hinweis (analog bestehender v2.1-Regel "Session darf nie personenbezogene Daten enthalten"):** `selectedItemRefs` enthält nur IDs, niemals Inhalte — konsistent mit der bestehenden Invariante aus § 16.

---

## § 27 — ERGÄNZUNG SnippetFilter / ClipboardFilter [GEÄNDERT — v2.2]

Rein additive, rückwärtskompatible Feldergänzungen an bestehenden v2.1-Filtertypen (keine bestehenden Felder verändert):

```typescript
interface SnippetFilter {
  // ... alle v2.1-Felder unverändert ...
  readonly tagsMode:        'all' | 'any' | 'none';        // [GEÄNDERT v2.2] 'none' neu hinzugefügt
  readonly collectionTabId: Option<CollectionTabId>;        // [NEU v2.2]
}

interface ClipboardFilter {
  // ... alle v2.1-Felder unverändert ...
  readonly tagsMode:        'all' | 'any' | 'none';         // [NEU v2.2] Feld existierte in v2.1 nicht — jetzt ergänzt
  readonly collectionTabId: Option<CollectionTabId>;        // [NEU v2.2]
}
```

---

## § 28 — AKTUALISIERTE FRONTEND-DATEISTRUKTUR [GEÄNDERT — v2.2]

Ergänzung zu § 20.2 (v2.1) — nur neue Einträge, bestehende Struktur bleibt unverändert:

```
src/lib/
├── domain/
│   ├── collection-tab.ts        # CollectionTab, CollectionKind, ItemRef       [NEU v2.2]
│   ├── tag-registry.ts          # TagRegistry.normalize, TagInfo              [NEU v2.2]
│   ├── sequence.ts              # Sequence, SequenceItem, renderSequence      [NEU v2.2]
│   └── unified-filter.ts        # UnifiedItemFilter, ItemKind                 [NEU v2.2]
│
├── ipc/
│   ├── collections.ts           # list/create/update/delete_collection_tab... [NEU v2.2]
│   ├── tags.ts                  # list_all_tags, suggest_tags, rename_tag...  [NEU v2.2]
│   ├── sequences.ts             # list/create/render_sequence, quick_combine  [NEU v2.2]
│   └── global-search.ts         # search_all_items                            [NEU v2.2]
│
├── stores/
│   ├── collections.ts           # collectionTabsStore, activeTabStore         [NEU v2.2]
│   ├── tags.ts                  # tagRegistryStore, tagBrowserFilterStore     [NEU v2.2]
│   ├── sequences.ts             # sequencesStore, activeSequenceDraft         [NEU v2.2]
│   └── selection.ts             # selectionStore: writable<ItemRef[]>         [NEU v2.2]
│                                 # (typübergreifende Mehrfachauswahl, siehe § 24.6)
│
└── components/
    ├── collections/  (§ 22.4)
    ├── tags/         (§ 23.6)
    ├── sequences/    (§ 24.6)
    └── search/       (§ 23.6)
```

Neue Routen:

```
routes/
├── collections/[tabId]/+page.svelte    [NEU v2.2]
├── sequences/
│   ├── +page.svelte                    [NEU v2.2]  # Liste aller Sequenzen
│   └── [id]/+page.svelte               [NEU v2.2]  # Sequence-Builder
└── search/+page.svelte                 [NEU v2.2]  # Volle globale Suchergebnis-Seite (GlobalSearchBar
                                                       # öffnet für Kurzergebnisse ein Dropdown, "Alle anzeigen"
                                                       # navigiert hierher)
```

---

## § 29 — MIGRATIONS-CHECKLISTE FÜR DIE UMSETZUNG [NEU — v2.2]

```
Empfohlene Migrationsreihenfolge (jede Datei append-only, INVARIANT-F):

010_collection_tabs.sql        — § 22.5
011_tag_registry.sql           — § 23.7 (inkl. script_tags, pipeline_tags)
012_sequences.sql              — § 24.7
013_notes.sql                  — § 25.1 (note-Spalte auf snippets + clipboard_history)
014_clipboard_ignore_rules.sql — § 25.2
015_clipboard_mime_types.sql   — § 25.4 (source_mime_types als JSON-Array-Spalte)
016_scripts_fts.sql            — Nachrüstung aus der Lückenanalyse (§ 3.1 dort), wird
                                  durch die neue script_tags-Tabelle in 011 ohnehin relevant
                                  und sollte im selben Arbeitsschritt mit erledigt werden

Jede Migration MUSS von einem Fresh-Install-Test begleitet werden (siehe Lückenanalyse,
Abschnitt "Fresh-Install-Smoke-Test" — dieser Test muss künftig bei JEDER neuen Migration
mitlaufen, nicht nur einmalig nachgerüstet werden).
```

---

## § 30 — ZUSAMMENFASSUNG: WAS DIESES ADDENDUM LÖST

| Nutzeranforderung | Gelöst durch |
|---|---|
| "Mehrere Elemente in Reihenfolge kombinieren" | § 24 Sequenz-Engine (`Sequence`, `SequenceItem`, `renderSequence`, `quick_combine`) |
| "Tag-basierte Suche" | § 23 TagRegistry + `UnifiedItemFilter.tags`/`tagsMode` inkl. `'none'`-Ausschluss, Autocomplete via `suggest_tags` |
| "Reiter-Organisation" | § 22 `CollectionTab` (manual/smart/clipboard_capture), Reiterleiste, Drag&Drop |
| "Durchsuchen der Elemente" | § 23.5 `search_all_items` — Item-Typ-übergreifende Suche mit Highlighting, plus bestehende FTS5-Suchen bleiben erhalten |
| "CopyQ-Features berücksichtigen" | § 25: Notizen, Ignore-Regeln, Multi-Paste/Plain-Text-Paste, MIME-Erhalt, CLI, Vim-Navigation |
| "Viel besser werden" | Durchgängige Undo-Integration (INVARIANT-I), Privacy-konforme Session-Erweiterung, keine Breaking Changes an v2.1 — additive, risikoarme Evolution statt Rewrite |
