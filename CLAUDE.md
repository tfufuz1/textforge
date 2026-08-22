# CLAUDE.md — TextForge

> Kanonische Arbeitsanweisung für Claude und alle LLM-Agenten in diesem Repository.
> Bei Widersprüchen zwischen dieser Datei und der Spec gilt: **Spec ist Wahrheit, CLAUDE.md ist Werkzeug.**
> Spezifikation: `SPECIFICATION.1.md`

---

## 1. Projekt auf einen Blick

TextForge ist ein **persönliches Text-Transformations-Tool** — kein SaaS, kein Team-Produkt.
Einzel-Nutzer, Einzelrechner. Das beeinflusst jeden Trade-off: Korrektheit vor Durchsatz,
Lesbarkeit vor Micro-Optimierung, explizite Typen vor cleveren Abstraktionen.

| Eigenschaft | Wert |
|---|---|
| Stack | Tauri 2.x (Rust) · SvelteKit (TypeScript) · SQLite (sqlx) |
| Plattform | KDE Plasma 6 · Wayland · Linux |
| Clipboard | `wl-paste --watch` subprocess (kein X11, kein Polling) |
| Sandbox | QuickJS via `rquickjs` — kein eval im Frontend |
| Spec-Datei | `SPECIFICATION.1.md` |
| Tests | Vitest (Domain Core) · Rust `#[test]` (IPC-Commands) |

---

## 2. Bevor du anfängst — Pflicht-Lesungen

**Lies immer zuerst den relevanten Spec-Abschnitt, bevor du Code schreibst.**
Die Spec ist die einzige Quelle der Wahrheit. Nicht das, was du aus dem Kontext schließt.

```
Neue Funktion / neues Feature     → §-Abschnitt in Spec + § 24 Checkliste
Neuer IPC-Command                 → § 18 + § 19 (DTO-Strukturen)
Neue SQLite-Tabelle / -Spalte     → § 17 (Schema) + INVARIANT-F
Neuer Svelte-Store                → § 20.1 (Store-Topologie)
Clipboard-Änderungen              → § 8 komplett
Fehlerbehandlung                  → § 2.4 (DomainError)
Neue Builtin-Transformation       → Anhang A (BuiltinId-Referenz)
```

---

## 3. Entwicklungsreihenfolge — nicht überspringen

Implementiere in dieser Reihenfolge. Fange keine Phase an, solange die vorherige
kein funktionsfähiges Deliverable produziert.

```
PHASE 1 — Clipboard-Kern                     (Spec § 8, § 2.5, § 5.2, § 17, § 18.2)
  Deliverable: Live-Clipboard-Verlauf sichtbar, durchsuchbar, pinnable

PHASE 2 — Snippet aus Clipboard              (Spec § 2.1, § 18.1 promote_*, § 9)
  Deliverable: Ein-Klick Clipboard → Snippet mit Undo

PHASE 3 — Snippet-Bearbeitung               (Spec § 2.1, § 3 Script, § 4, § 6)
  Deliverable: Snippets bearbeiten, transformieren, als Template nutzen

PHASE 4 — Erweiterte Features               (Spec § 10, § 13, § 14, § 15, § 16)
  Deliverable: Import/Export, Bulk-Ops, Diff-Viewer
```

Wenn du mitten in Phase 1 gebeten wirst, eine Pipeline zu bauen: **ablehnen und
darauf hinweisen, dass Phase 1 noch nicht abgeschlossen ist.**

---

## 4. Architektur-Regeln — nie verletzen

Diese Regeln kommen aus `§ 0` der Spec. Sie sind keine Empfehlungen.

### 4.1 Schichten-Trennung

```
Domain Core  →  Application  →  Effect Shell  →  Frontend (IPC)
```

- **Domain Core** (`src/lib/domain/`): Null Dependencies. Kein Import aus `effect/`, `ipc/`, `stores/`.
  Nur pure Funktionen, `Option<A>`, `Result<E,A>`, Validierungen.
- **Effect Shell** (`src-tauri/src/`): Genau ein Einstiegspunkt pro Seiteneffekt
  (eine Clipboard-Schreib-Funktion, eine SQLite-Connection, eine QuickJS-Instanz).
- **Frontend-Stores**: Werden ausschließlich durch IPC-Aufrufe mutiert.
  Kein `$store = ...` direkt in Komponenten außerhalb des Store-Moduls.

### 4.2 Die acht Invarianten (INVARIANT-A bis H)

| ID | Regel |
|---|---|
| A | Domain Core importiert nie aus `effect/` oder `ipc/` |
| B | Kein DTO verlässt `ipc/` ohne Konvertierung in Domain-Typ |
| C | Stores nur durch IPC-Calls mutieren — kein direktes `$store =` |
| D | Jede Mutation erzeugt ein neues Objekt (`...spread` / `structuredClone`) |
| E | Fehler sind `Result<DomainError, T>` — niemals `throw()` in Business-Logik |
| F | SQLite-Schema ist append-only — nie bestehende Migrations-Dateien ändern |
| G | QuickJS-Sandbox: kein Netzwerk, kein Filesystem, kein Clipboard |
| H | Alle Timestamps sind `UnixMs` (UTC, Millisekunden) — keine Zeitzonenstrings |

Wenn eine Anforderung eine dieser Invarianten verletzt: **Invariante beibehalten,
Alternative vorschlagen, Abweichung explizit benennen.**

### 4.3 Typregeln

```typescript
// ✓ RICHTIG
const result: Result<DomainError, Snippet> = Snippet.create(draft);

// ✗ FALSCH — throw in Business-Logik
function createSnippet(draft) {
  if (!draft.title) throw new Error('No title'); // VERBOTEN
}

// ✓ RICHTIG — Option statt null
const sourceApp: Option<string> = Option.fromNullable(detected);

// ✗ FALSCH — null zurückgeben
function detectApp(): string | null { ... } // VERBOTEN im Domain Core

// ✓ RICHTIG — Branded Types für IDs
const id: SnippetId = SnippetId.of(crypto.randomUUID());

// ✗ FALSCH — rohe Strings als IDs
function getSnippet(id: string) { ... } // VERBOTEN — muss SnippetId sein
```

---

## 5. Clipboard — Wayland-spezifisch

Das ist die häufigste Fehlerquelle. KDE Plasma 6 läuft auf **Wayland, nicht X11**.

### Was verwendet wird

```rust
// RICHTIG: wl-paste --watch subprocess
tokio::process::Command::new("wl-paste")
    .arg("--watch")
    .arg("--no-newline")
    .stdout(Stdio::piped())
    .spawn()

// Fallback wenn wl-paste nicht verfügbar:
arboard::Clipboard::get_text()  // 500ms Polling
```

### Was NICHT verwendet wird

```rust
// FALSCH — kein X11 mehr
x11rb::connect()                    // existiert nicht mehr im Projekt
xfixes_select_selection_input()     // existiert nicht mehr
xdotool / xprop                     // NICHT auf Wayland verfügbar
```

### Quell-App-Erkennung (Wayland)

```rust
// Reihenfolge: KWin D-Bus → qdbus6 subprocess → procfs → None
// Alle Strategien sind graceful — None bei Fehler, kein Panic

// Abhängigkeiten auf dem System:
// sudo apt install wl-clipboard kde-cli-tools
```

### Wichtige Clipboard-Invarianten

- SHA-256-Dedup: Identischer Hash → verwerfen (kein Doppeleintrag)
- LRU-Limit: `settings.clipboard.max_entries` (default 500) — nur unpinned Einträge löschen
- Mindestlänge: `settings.clipboard.min_length` (default 3 Zeichen)
- Subprocess-Cleanup: `child.kill()` beim App-Ende — kein verwaister `wl-paste`-Prozess

---

## 6. SQLite-Schema-Regeln

**INVARIANT-F ist absolut.** Bestehende Migrations-Dateien werden nie geändert.

```
migrations/
├── 001_initial.sql          ← NIEMALS ANFASSEN
├── 002_v2_extensions.sql    ← NIEMALS ANFASSEN
└── 003_*.sql                ← neue Features kommen hierher
```

### Schema-Änderungen — so geht es richtig

```sql
-- ✓ RICHTIG: neue Datei, neue Spalte hinzufügen
-- migrations/003_my_feature.sql
ALTER TABLE snippets ADD COLUMN my_field TEXT;

-- ✗ FALSCH: bestehende Migration bearbeiten
-- 002_v2_extensions.sql öffnen und ändern → VERBOTEN
```

### FTS5-Trigger nach Schema-Änderung prüfen

Wenn du `snippets`, `scripts` oder `clipboard_history` änderst:
FTS5-Trigger in der gleichen Migration explizit neu erstellen (`DROP TRIGGER IF EXISTS` + `CREATE TRIGGER`).

---

## 7. IPC-Konventionen

Jeder Tauri-Command folgt diesem Muster:

```rust
// src-tauri/src/commands/my_module.rs

#[tauri::command]
pub async fn my_command(
    param:  MyParamDto,          // DTOs kommen rein (§ 19)
    state:  State<'_, AppState>,
) -> Result<MyResultDto, String> {  // DTOs gehen raus — niemals Domain-Typen
    let domain_val = MyDomainType::from_dto(param)  // DTO → Domain
        .map_err(|e| DomainError::describe(&e))?;

    let result = state.db.do_something(domain_val).await
        .map_err(|e| e.to_string())?;

    Ok(MyResultDto::from(result))  // Domain → DTO
}
```

- **Fehler**: `Result<_, String>` nach außen (Tauri-Konvention), `Result<DomainError, T>` intern
- **DTOs**: Alle in `§ 19` definiert — keine ad-hoc Structs in Commands
- **State**: `AppState` enthält DB-Pool + Undo-Stack + Clipboard-Config
- **Keine Business-Logik in Commands** — nur Orchestrierung: DTO-Konversion, DB-Aufruf, Event-Emit

---

## 8. Frontend-Store-Regeln

```typescript
// ✓ RICHTIG: Store nur durch IPC mutieren
async function addSnippet(draft: SnippetDraft) {
  const dto = await invoke<SnippetDto>('create_snippet', { draft: toDto(draft) });
  snippetsStore.update(ss => [fromDto(dto), ...ss]);
}

// ✗ FALSCH: Store direkt setzen
snippetsStore.set([...get(snippetsStore), newSnippet]);  // VERBOTEN außerhalb ipc/

// ✓ RICHTIG: Derived Store
const filteredSnippets = derived([snippetsStore, filterStore], applyFilter);

// ✗ FALSCH: Derived Store manuell setzen
filteredSnippets.set([...]);  // VERBOTEN — derived Stores sind readonly
```

---

## 9. QuickJS-Sandbox-Regeln

Jede JavaScript-Ausführung läuft durch die QuickJS-Sandbox. **Kein eval() im Frontend.**

```typescript
// ✓ RICHTIG: Immer über IPC
const result = await invoke<ScriptResultDto>('execute_script', {
  scriptId: script.id,
  input: content,
  params: paramValues,
});

// ✗ FALSCH: Direkte Ausführung im Frontend
const fn = new Function('input', 'utils', script.jsCode);  // VERBOTEN
const result = fn(content, utils);                          // SICHERHEITSLÜCKE
```

Sandbox-Limits (aus `§ 22 SETTINGS_SCHEMA`):
- Timeout: 3000ms (default)
- Output-Limit: 512 KB
- Input-Limit: 2 MB

---

## 10. Fehlerbehandlung

### Domain Core

```typescript
// Immer Result<DomainError, T> — nie throw
function parseTag(raw: string): Result<DomainError, TagName> {
  return /^[a-z0-9_\-]{1,32}$/.test(raw.trim())
    ? Result.ok(raw.trim() as TagName)
    : Result.err({ code: 'INVALID_TAG', raw });
}
```

### Frontend

```typescript
// Fehler via notificationsStore — nie alert() oder console.error allein
Result.fold(
  result,
  (err) => notificationsStore.push(Notifications.transformError(err)),
  (val) => snippetsStore.update(...)
);
```

### Neue DomainError-Varianten

Wenn ein neuer Fehlerfall entsteht: **immer in `§ 2.4` der Spec eintragen und
in `DomainError.describe()` einen deutschen Text hinzufügen.** Kein undokumentierter Fehlercode.

---

## 11. Was du NICHT tun sollst

Diese Aktionen erfordern explizite Rückfrage beim Nutzer, bevor du fortfährst:

| Aktion | Grund |
|---|---|
| Bestehende Migration-Datei bearbeiten | INVARIANT-F — Datenverlust möglich |
| `eval()` / `new Function()` im Frontend | Sicherheitslücke, INVARIANT-G |
| X11/xdotool/x11rb hinzufügen | Plattform ist Wayland-only |
| `throw` in Domain-Core-Funktion | INVARIANT-E — bricht Fehlermodell |
| Store direkt setzen (nicht via IPC) | INVARIANT-C — erzeugt inkonsistenten Zustand |
| Domain-Typ direkt als IPC-Rückgabe | INVARIANT-B — DTO-Grenze muss eingehalten werden |
| Phase 3+ Feature implementieren wenn Phase 1/2 unfertig | Scope-Creep-Prävention |
| Neuen `BuiltinId` ohne Anhang-A-Eintrag | DRY-Verletzung |

---

## 12. Vor jedem Commit — Checkliste (§ 24)

Führe diese Checks durch, bevor du Code als fertig deklarierst:

```
DOMAIN CORE
□ Gibt die Funktion Result<DomainError, T> statt zu werfen?
□ Gibt es null-Rückgaben? → Option<T>
□ Wird ein Argument mutiert? → neue Kopie
□ Sind alle Timestamps UnixMs (UTC)?
□ Ist die Funktion ohne DB/QuickJS/Clipboard testbar?

ENTITY-INVARIANTEN
□ Wurden alle INVARIANT-* der betroffenen Entity geprüft?
□ updatedAt bei jeder Mutation aktualisiert?
□ FTS5-Trigger nach Schema-Änderung noch intakt?

CLIPBOARD (bei § 8-Änderungen)
□ SHA-256-Dedup korrekt angewendet?
□ LRU-Limit enforced (nur unpinned löschen)?
□ wl-paste subprocess bei App-Ende sauber beendet?
□ Mindestlänge geprüft?

UNDO/REDO
□ UndoStack.push() VOR der DB-Operation?
□ Redo-Stack bei neuer Aktion geleert?
□ Bulk-Operationen erzeugen genau EINEN UndoEntry?

FRONTEND
□ Store nur durch IPC mutiert?
□ Derived Stores korrekt als derived() deklariert?
□ Notifications via notificationsStore.push() statt alert()?

SECURITY
□ Kein eval() / new Function() im Frontend?
□ Markdown-Output durch DOMPurify sanitisiert?
□ Clipboard-Inhalt nicht in Logs?
```

---

## 13. Projektstruktur auf einen Blick

```
textforge/
├── CLAUDE.md                        ← diese Datei
├── docs/
│   └── textforge-interface-spec-v2.1.md  ← autoritative Spec
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                  ← Tauri-Setup, App-State
│   │   ├── commands/                ← IPC-Command-Handler (§ 18)
│   │   │   ├── snippets.rs
│   │   │   ├── clipboard.rs         ← PHASE 1 — zuerst
│   │   │   ├── scripts.rs
│   │   │   ├── pipelines.rs
│   │   │   ├── transform.rs
│   │   │   ├── bulk.rs
│   │   │   ├── undo.rs
│   │   │   ├── import_export.rs
│   │   │   └── settings.rs
│   │   ├── clipboard/               ← Wayland-Monitor (§ 8)
│   │   │   ├── mod.rs               ← start_monitor(), MonitorConfig
│   │   │   └── source_app.rs        ← KWin D-Bus → PID → /proc
│   │   ├── db/                      ← sqlx-Queries, Migrations
│   │   │   └── migrations/
│   │   │       ├── 001_initial.sql  ← NICHT ANFASSEN
│   │   │       ├── 002_v2_extensions.sql  ← NICHT ANFASSEN
│   │   │       └── 003_*.sql        ← neue Features
│   │   └── sandbox/                 ← QuickJS-Wrapper (§ 21)
│   └── Cargo.toml
└── src/
    └── lib/
        ├── domain/                  ← Pure Core (§ 1, § 2, § 3, § 4, § 5, § 6)
        │   ├── adts.ts              ← Option, Result, NonEmptyArray
        │   ├── snippet.ts
        │   ├── clipboard-entry.ts   ← PHASE 1 — zuerst
        │   ├── script.ts
        │   ├── pipeline.ts
        │   ├── template.ts
        │   ├── text-stats.ts
        │   ├── undo.ts
        │   ├── filter.ts
        │   └── errors.ts            ← DomainError (erschöpfend)
        ├── ipc/                     ← invoke()-Wrapper (§ 18)
        │   ├── clipboard.ts         ← PHASE 1 — zuerst
        │   └── ...
        ├── stores/                  ← Svelte-Stores (§ 20.1)
        │   ├── clipboard.ts         ← PHASE 1 — zuerst
        │   └── ...
        └── components/
            ├── clipboard/           ← PHASE 1 — zuerst
            │   ├── ClipboardHistory.svelte
            │   ├── ClipboardEntry.svelte
            │   ├── ClipboardFilter.svelte
            │   └── ClipboardEntryActions.svelte
            └── ...
```

---

## 14. Systemvoraussetzungen prüfen

Beim ersten Start (oder nach einer frischen Installation) prüfen:

```bash
# Wayland-Clipboard
which wl-paste        # muss vorhanden sein
wl-paste --version    # wl-clipboard >= 1.0

# KDE D-Bus für Quell-App-Erkennung
which qdbus6          # in kde-cli-tools

# Falls nicht vorhanden:
sudo apt install wl-clipboard kde-cli-tools
```

Der ClipboardMonitor prüft `wl-paste` beim Start und fällt auf arboard-Polling zurück,
wenn es nicht verfügbar ist — **kein Panic, kein Fehler** (graceful degradation).

---

## 15. Häufige Fragen

**F: Ich brauche eine neue Setting-Option. Wo kommt die hin?**
`§ 22 SETTINGS_SCHEMA` in der Spec aktualisieren, dann in `002_v2_extensions.sql`
(oder neue Migration) als `INSERT OR IGNORE INTO settings VALUES (...)`.

**F: Ich will eine neue Svelte-Komponente erstellen. Welche Stores darf sie nutzen?**
Nur `import`-Stores aus `src/lib/stores/`. Keine direkten `invoke()`-Calls in Komponenten —
das geht über den Store-Wrapper in `src/lib/ipc/`.

**F: Wie teste ich eine Domain-Core-Funktion?**
```bash
npx vitest run src/lib/domain/my-function.test.ts
```
Domain-Core-Tests brauchen kein Tauri, kein SQLite, keinen Browser.

**F: Wie teste ich einen Rust IPC-Command?**
```bash
cargo test -p textforge -- commands::my_module
```
Commands ohne `AppState` (reine Berechnungen) als Unit-Test,
Commands mit DB als Integration-Test mit `sqlx::test`.

**F: Welche Fehlercodes darf ich verwenden?**
Nur Codes aus `DomainError` in `§ 2.4`. Neuen Code braucht: Spec-Eintrag + `describe()`-Fall.

**F: Darf ich `console.log` in Produktion lassen?**
Im Domain Core: nein. In Komponenten für Entwicklungs-Debugging: nur mit `import.meta.env.DEV`-Guard.
Clipboard-Inhalt darf **niemals** in irgendeinem Log erscheinen.

**F: Wie aktualisiere ich die Spec?**
Spec-Änderungen (nicht nur Code) über `SPECIFICATION.1.md` —
mit einem klaren `[GEÄNDERT — vX.Y]`-Marker am betroffenen Abschnitt.

---

## 16. Kommunikationsstil

- **Kurze Zusammenfassung vor dem Code**: Was du vorhast und warum (max. 3 Sätze).
- **Spec-Referenz nennen**: „Gemäß § 8.2 verwendet der Monitor wl-paste --watch..."
- **Abweichungen explizit benennen**: Wenn du von der Spec abweichst, sagst du es und begründest es.
- **Keine spekulativen Implementierungen**: Wenn die Spec unklar ist, frag zuerst.
- **Invarianten nicht still brechen**: Wenn eine Anforderung INVARIANT-X verletzt, sagst du es laut.
