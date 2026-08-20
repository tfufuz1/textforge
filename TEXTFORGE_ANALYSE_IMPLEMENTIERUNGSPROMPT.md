# TextForge — Lückenanalyse & Implementierungsprompt für den leitenden Entwickler

> **Analysierte Quelle:** `github.com/tfufuz1/textforge` (vollständiger Klon, Commit-Stand vom Analysedatum)
> **Analysemethode:** Statischer Code-Review aller Rust- und TypeScript-Quellen gegen `SPECIFICATION.1.md` (3.239 Zeilen) und `CLAUDE.md`, ergänzt durch tatsächliche Ausführung der SQL-Migrationskette und der Vitest-Suite.
> **Zielgruppe:** Leitender Entwickler, der die nächste Implementierungs-Iteration plant und priorisiert.

---

## 0. Wie dieser Bericht zu lesen ist

Die Spezifikation (`SPECIFICATION.1.md`) enthält an einigen Stellen bereits `[Status: ...]`-Marker, die von einem früheren Analyse-Durchlauf stammen. **Diese Marker sind streckenweise veraltet und teils falsch** — sowohl in beide Richtungen:

- An mehreren Stellen ist der Code **weiter** als der Marker vermuten lässt (z. B. Template-Filter, Import-Konfliktpolicies, FTS5-Suche in der Zwischenablage sind vollständig implementiert, obwohl als "Partially Implemented" markiert).
- An anderen Stellen fehlen Marker **komplett**, obwohl die Funktion nicht implementiert ist (z. B. `bulk_transform`/`bulk_export` sind reine `NOT_IMPLEMENTED`-Stubs, ohne jeden Hinweis in der Spec).

Dieser Bericht verlässt sich **nicht** auf die Statusmarker, sondern auf tatsächliche Code-Verifikation. Jeder Befund unten wurde durch Lesen der relevanten Datei(en) bestätigt; die schwerwiegendsten Befunde wurden zusätzlich empirisch reproduziert (SQL-Migration tatsächlich ausgeführt, Vitest-Suite tatsächlich laufen lassen).

---

## 1. P0 — Blocker: App ist in diesem Zustand nicht startfähig

### 1.1 Migrationskonflikt zerstört jede Neuinstallation ⛔

**Befund (empirisch verifiziert):** `migrations/002_v2_extensions.sql` und `migrations/007_snippet_color_favorite.sql` fügen beide unabhängig voneinander eine Spalte `snippets.color` per `ALTER TABLE ... ADD COLUMN` hinzu:

```sql
-- 002_v2_extensions.sql, Zeile 4
ALTER TABLE snippets ADD COLUMN color TEXT CHECK(color GLOB '#??????' OR color IS NULL);

-- 007_snippet_color_favorite.sql, Zeile 5
ALTER TABLE snippets ADD COLUMN color TEXT;
```

Ich habe die komplette Migrationskette (001–008) sequenziell gegen eine leere SQLite-Datenbank ausgeführt. Ergebnis:

```
--- 007_snippet_color_favorite.sql ---
FEHLER: duplicate column name: color
```

Da `src-tauri/src/db/mod.rs::init_db()` `sqlx::migrate!("./migrations").run(&pool).await?` aufruft und `main.rs` das Ergebnis mit `.expect("Failed to initialize db")` entgegennimmt, **stürzt die Anwendung beim allerersten Start auf jedem System ab**, auf dem noch keine (bereits fehlerhaft/zufällig migrierte) Datenbank existiert. Das ist mit hoher Wahrscheinlichkeit der Grund, warum `src-tauri/textforge_dev.db` im Repository mit 0 Byte vorliegt.

Zusätzlich enthält `007_snippet_color_favorite.sql` ein **redundantes zweites Favoriten-Feld** (`favorite`), obwohl `is_favorite` bereits in `002_v2_extensions.sql` existiert — zwei konkurrierende Spalten für dasselbe Konzept.

**Auftrag an den leitenden Entwickler:**
1. Neue Migration `009_fix_duplicate_color_column.sql` anlegen (INVARIANT-F beachten — 002/007 dürfen nicht verändert werden).
2. Da `ALTER TABLE ... DROP COLUMN` in älteren SQLite-Versionen eingeschränkt ist: prüfen, welche der beiden `color`-Definitionen (mit oder ohne `CHECK`-Constraint) kanonisch sein soll, den Konflikt aus 007 durch Umbenennung/Entfernen der redundanten Anweisung in einer neuen Migration bereinigen (z. B. `007` faktisch nie erfolgreich angewendet — daher kann 009 einfach das tun, was 007 *zusätzlich* wollte: `favorite`-Spalte hinzufügen, **ohne** `color` erneut anzulegen).
3. `favorite` vs. `is_favorite`: auf ein Feld konsolidieren, den ungenutzten Namen als deprecated markieren oder per Migration zusammenführen.
4. Diesen Fix mit einem Frisch-Installations-Test absichern (siehe 1.3).

### 1.2 Einziger Rust-Integrationstest ist unabhängig davon ebenfalls defekt ⛔

**Befund (empirisch verifiziert):** `src-tauri/tests/clipboard.rs` versucht:

```rust
sqlx::query(
    "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at, size_bytes) VALUES (?, ?, ?, ?, ?, ?)"
)
```

`size_bytes` ist in `003_clipboard.sql` als `GENERATED ALWAYS AS (length(content)) VIRTUAL` deklariert — eine berechnete Spalte, in die SQLite kein `INSERT` erlaubt. Ich habe das gegen eine reale SQLite-Instanz reproduziert:

```
INSERT FEHLER: cannot INSERT into generated column "size_bytes"
```

**Auftrag:** `size_bytes` aus der `INSERT`-Spaltenliste in `tests/clipboard.rs` entfernen. Das ist unabhängig vom Migrationsfix in 1.1 — beide Bugs müssen behoben werden, damit der Test grün wird.

### 1.3 Fehlender CI-Smoke-Test für Frischinstallation

Es existiert kein Test, der `init_db()` gegen eine brandneue, leere Datei aufruft und lediglich prüft, dass die Migration ohne Fehler durchläuft. Genau dieser Fall (neuer Nutzer, neue Maschine) ist der am häufigsten durchlaufene Pfad der gesamten Anwendung und war zum Zeitpunkt der Analyse ungetestet.

**Auftrag:** Test `test_fresh_database_migration()` ergänzen, der `init_db(&tempfile)` aufruft und nur `assert!(result.is_ok())` prüft — kein Fixture, keine Vorbedingungen. Diesen Test in eine CI-Pipeline aufnehmen, die bei jedem PR läuft.

---

## 2. P1 — Kernfunktionen sind UI-seitig sichtbar, aber wirkungslos ("Potemkinsche Features")

Diese Kategorie ist besonders tückisch: Die Buttons existieren, die Stores existieren, die Backend-Logik sieht auf den ersten Blick fertig aus — aber die Verbindungskette ist an einer Stelle unterbrochen, sodass das Feature aus Nutzersicht nie funktioniert.

### 2.1 Undo/Redo ist komplett funktionslos

`src-tauri/src/commands/undo.rs` enthält eine überraschend vollständige `execute_action_recursive()`-Funktion, die für alle 12 spezifizierten `UndoAction`-Varianten (Snippet, Script, Pipeline, Folder, Bulk, Transform) die inverse Operation korrekt gegen die DB ausführt.

**Aber:** Ich habe *jeden* Command in `snippets.rs`, `bulk.rs` und `transform.rs` durchsucht — **keine einzige Stelle** außerhalb von `undo.rs` referenziert `state.undo_stack` oder erzeugt einen `UndoEntryDto`. Der Stack bleibt für die gesamte Laufzeit der App leer. Der Frontend-Store bestätigt das:

```typescript
// src/lib/stores/undo.ts
export function pushUndoAction(_action: any, _description?: string) {
  refreshUndoState();   // <- ruft NUR eine Statusabfrage auf, pusht nichts
}
```

`pushUndoAction` wird zudem im gesamten Frontend nur an **einer** Stelle aufgerufen (`ClipboardEntryActions.svelte`), und selbst dort passiert wegen der Store-Implementierung nichts Persistentes.

**Konsequenz:** `Strg+Z` kann in der aktuellen App **niemals** etwas rückgängig machen, unabhängig davon, welche Aktion zuvor ausgeführt wurde.

**Auftrag:**
1. Neuen IPC-Command `push_undo_entry(entry: UndoEntryDto)` einführen (oder: jeder mutierende Command in `snippets.rs`/`bulk.rs`/`transform.rs` befüllt `state.undo_stack` direkt vor dem `Ok(...)`-Return).
2. Am Beispiel `update_snippet`, `create_snippet`, `trash_snippet`, `execute_bulk_operation` beginnen — das sind die am häufigsten genutzten mutierenden Pfade.
3. Nach jeder Mutation im Frontend-Store (`snippets.ts`, `scripts.ts`, `pipelines.ts`) tatsächlich `refreshUndoState()` aufrufen, damit die UI (`canUndo`/`canRedo`) den echten Zustand zeigt.
4. `CLAUDE.md` §12 Checkliste „UndoStack.push() VOR der DB-Operation?" ernst nehmen — aktuell wird diese Checkliste offenbar nicht durchgesetzt.

### 2.2 Skript-Versions-Wiederherstellung ist über die UI defekt

`ScriptVersionHistory.svelte` ruft auf:

```typescript
await restoreScriptVersion(scriptId, version);   // version: number, z.B. 1, 2, 3
```

Der Rust-Command erwartet aber:

```rust
pub async fn restore_script_version(
    script_id: String,
    version_id: String,   // UUID der script_versions-Zeile, NICHT die Versionsnummer
    ...
) {
    ... WHERE id = ? AND script_id = ? ...
}
```

Da niemals die tatsächliche `id` der Versionszeile übergeben wird, schlägt jeder Restore-Versuch mit `"Script version not found"` fehl. Das UI-Feature "Version wiederherstellen" ist vollständig verdrahtet, aber **nie funktionsfähig**.

**Auftrag:** `ScriptVersion`-Domain-Typ und `listScriptVersions()`-Rückgabe müssen die `id` jeder Version mitführen; `ScriptVersionHistory.svelte` muss diese `id` (nicht `version`) an `restoreScriptVersion()` übergeben. Signatur in `ipc/scripts.ts` entsprechend auf `restoreScriptVersion(scriptId: string, versionId: string)` korrigieren.

### 2.3 Workspace-Session wird nie geladen oder gespeichert

Backend-IPC (`get_workspace_session`, `save_workspace_session`) und die zugehörige Wrapper-Datei `src/lib/ipc/session.ts` existieren vollständig. **Kein einziger** Ort im Frontend importiert diese Datei. Ergebnis: Ansicht, Filter, Sidebar-Breite, zuletzt geöffnetes Snippet — der komplette in § 16 spezifizierte UI-Zustand geht bei jedem Neustart verloren, obwohl das Backend dafür bereit ist.

**Auftrag:**
1. In `+layout.svelte`: beim Mount `getWorkspaceSession()` aufrufen und `activeSessionStore` initialisieren.
2. Debounced Autosave (2s, wie in § 16 gefordert) bei Tab-Wechsel, Filteränderung, Snippet-Öffnen etc. einbauen — z. B. über einen `derived`-Store mit `debounce`-Utility, der `saveWorkspaceSession()` aufruft.
3. `WorkspaceSession`-Domain-Typ (`session.ts`) um die fehlenden Felder `sidebarWidth`, `previewMode`, `filterState`, `openEditorTabs`, `savedAt` erweitern (siehe 3.6).

### 2.4 Bulk-Transform und Bulk-Export sind Stubs

```rust
// src-tauri/src/commands/bulk.rs
BulkOperationDto::BulkTransform { snippet_ids, .. } => {
    for id in snippet_ids {
        failed.push(BulkOperationFailedDto { id: id.clone(), error: json!({ "code": "NOT_IMPLEMENTED" }) });
    }
}
BulkOperationDto::BulkExport { snippet_ids, .. } => {
    for id in snippet_ids {
        failed.push(BulkOperationFailedDto { id: id.clone(), error: json!({ "code": "NOT_IMPLEMENTED" }) });
    }
}
```

Von 7 spezifizierten Bulk-Operationstypen sind 2 — darunter der vermutlich meistgenutzte (`bulk_transform`, Pipeline auf mehrere Snippets gleichzeitig anwenden) — reine Fehlerantworten. Kein Statusmarker in der Spec weist darauf hin.

**Auftrag:**
1. `BulkTransform`: Für jede `snippet_id` den Pipeline-Run wiederverwenden (`run_pipeline`-Logik aus `transform.rs` extrahieren und als gemeinsame Funktion nutzen), Ergebnis abhängig von `saveResults` entweder persistieren oder als `previews` zurückgeben.
2. `BulkExport`: Kann auf die Export-Logik aus `import_export.rs` zurückgreifen, eingeschränkt auf die übergebenen `snippet_ids`.
3. Für beide: Einen einzigen `UndoEntry` vom Typ `bulk_operation` erzeugen (siehe 2.1) — die Spec verlangt das explizit, das Datenmodell (`BulkOperation` in `UndoActionDto`) existiert bereits.

### 2.5 Export unterstützt nur `.tfbundle` — fünf von sechs Formaten fehlen

```rust
pub async fn export_data(...) -> Result<ExportResultDto, String> {
    if request.format != "tfbundle" {
        return Err("Only tfbundle export format is supported".to_string());
    }
    ...
}
```

`markdown`, `text`, `json`, `json_array`, `csv` sind spezifiziert, aber nicht implementiert. Zusätzlich: Der Formatname im Code (`"tfbundle"`) weicht vom Spec-Bezeichner (`'bundle'`) ab — Frontend und Dokumentation müssen konsistent sein.

**Auftrag:**
1. Formatnamen zwischen Spec, DTO und Rust-Match-Arm vereinheitlichen (Empfehlung: `'bundle'` als kanonischer Wert, da so in § 10.2 spezifiziert; Rust-Code entsprechend anpassen).
2. Je Format eine kleine, unabhängige Export-Funktion ergänzen: `text`/`markdown` sind trivial (Snippet-Content roh schreiben), `json`/`json_array` ist Serialisierung der bereits vorhandenen DTOs, `csv` benötigt einen einfachen CSV-Writer für Snippet-Metadaten (Titel, Tags, erstellt/aktualisiert, Ordner).

---

## 3. P2 — Spezifikationsabweichungen mit funktionaler Auswirkung

### 3.1 Tauri-Events: Nur 1 von 5 spezifizierten Events wird emittiert

Ich habe alle `.emit(...)`-Aufrufe im Rust-Code und alle `listen(...)`-Aufrufe im Frontend gesucht:

| Event (Spec § 18.5) | Backend emittiert? | Frontend hört zu? |
|---|---|---|
| `clipboard:new_entry` | Ja (aber Payload ist nur `id: String`, nicht `ClipboardEntryListItemDto` wie spezifiziert) | Ja |
| `pipeline:step_started` | Nein | Nein |
| `pipeline:step_complete` | Nein | Nein |
| `bulk:progress` | Nein | Nein |
| `import:progress` | Nein | Nein |

**Auswirkung:** Bei langlaufenden Pipelines, Bulk-Operationen (sobald 2.4 behoben ist) und Bundle-Importen gibt es keinerlei Fortschrittsanzeige — die UI wirkt bei größeren Operationen eingefroren.

**Auftrag:**
1. `clipboard:new_entry`-Payload auf das vollständige `ClipboardEntryListItemDto` erweitern, statt nur die `id` zu senden — spart dem Frontend einen zusätzlichen `get_clipboard_entry`-Roundtrip.
2. In `run_pipeline` (transform.rs) nach jedem Step `app_handle.emit("pipeline:step_started"/"pipeline:step_complete", ...)` ergänzen (State muss dazu als `AppHandle`, nicht nur `AppState`, in die Funktion gereicht werden — Signatur-Anpassung im Command nötig).
3. Analog für `execute_bulk_operation` und `import_data`.

### 3.2 Keyboard Shortcuts: Kein zentrales Registry vorhanden

Spec § 11 definiert ca. 20 Shortcuts über 4 Kontexte (global, snippet_list, snippet_editor, script_editor) plus Command Palette. Tatsächlich vorhanden im gesamten Frontend:

- `SnippetEditorLayout.svelte`: ein lokaler `keydown`-Handler für **ausschließlich** `Strg+Shift+M` (Preview-Toggle).
- `SnippetList.svelte`: `Enter`-Handler auf einem einzelnen Listenelement.
- Keine `ShortcutMap`, kein globaler Listener, keine Command Palette (`Strg+Shift+P`), kein Undo/Redo-Shortcut, kein `Strg+N`, kein `Strg+F`.

**Auftrag:**
1. Neues Modul `src/lib/shortcuts/registry.ts` gemäß Spec-Datenmodell (`Shortcut`, `ShortcutContext`) anlegen — eine einzige Quelle der Wahrheit, wie in CLAUDE.md § "DRY" gefordert.
2. Einen globalen `keydown`-Listener in `+layout.svelte` registrieren, der kontextabhängig (aktive Route/Fokus) das passende Set aus der Registry matched und die zugehörige `action`-ID dispatcht.
3. Command Palette als neue Komponente (`CommandPalette.svelte`) — kann zunächst als einfache Fuzzy-Filter-Liste über bekannte Aktionen implementiert werden, MVP ohne Erweiterbarkeit.
4. Undo/Redo-Shortcuts direkt an `performUndo()`/`performRedo()` aus `stores/undo.ts` anbinden (funktioniert erst sinnvoll nach Fix 2.1).

### 3.3 Clipboard-Monitor: Konfigurierbarkeit, Prozess-Cleanup, Quell-App-Erkennung

- **`ClipboardMonitorConfig` wird ignoriert:** `start_monitor(app_handle: AppHandle)` nimmt gar keine Config entgegen; `min_content_length` (`< 3`), `max_entries` (`500`) sind im Code hartkodiert. Die entsprechenden `settings`-Werte (`clipboard.min_length`, `clipboard.max_entries`, `clipboard.dedup_window_ms`) werden zwar in der DB gepflegt, aber nie gelesen.
- **`child.kill()` fehlt vollständig:** Der `wl-paste --watch`-Subprozess wird beim App-Ende nirgends terminiert. Bei wiederholten Dev-Restarts sammeln sich verwaiste `wl-paste`-Prozesse an.
- **`dedup_window_ms` wird nicht verwendet:** Dedup erfolgt ausschließlich über einen globalen `UNIQUE`-Constraint auf `content_hash` — ein zeitbasiertes Fenster für "kurz hintereinander kopierter identischer Text" existiert nicht (unschädlich, aber weicht von der Spezifikation ab).
- **`try_procfs_active()` (Fallback-Strategie 3 der Quell-App-Erkennung) ist ein Stub:**
  ```rust
  async fn try_procfs_active() -> Option<String> {
      None   // liefert immer None
  }
  ```
  Wenn sowohl KWin-D-Bus (nativ) als auch `qdbus6` fehlschlagen, bleibt die Quell-App-Erkennung ergebnislos, obwohl ein dritter Fallback spezifiziert und als Funktionsname bereits vorhanden ist.
- **LRU-Trim ohne Trigger-Absicherung:** Der Trim erfolgt applikationsseitig nach jedem Insert (`DELETE ... ORDER BY captured_at ASC LIMIT ?`), nicht über einen DB-Trigger. `DELETE ... LIMIT` ist zudem in Standard-SQLite ohne das Compile-Flag `SQLITE_ENABLE_UPDATE_DELETE_LIMIT` **nicht gültige Syntax** — das sollte gegen die tatsächlich gelinkte `rusqlite`/`sqlx`-SQLite-Variante geprüft werden, da dies sonst bei jedem Insert über dem Limit einen stillen Laufzeitfehler produziert.

**Auftrag:**
1. `start_monitor` Signatur auf `start_monitor(config: ClipboardMonitorConfig, app_handle: AppHandle)` ändern (wie in Spec § 8.2 vorgesehen), Config beim App-Start aus `get_all_settings()` befüllen.
2. In `main.rs`, im `Drop`-Pfad oder via Tauri `on_window_event(CloseRequested)`: `child.kill().await` auf den gehaltenen `Child`-Handle aufrufen. Dazu muss der `Child`-Handle aus `try_wl_paste_monitor` in den `AppState` wandern (aktuell wird er nur lokal im `tokio::spawn` gehalten und ist danach unerreichbar).
3. `try_procfs_active()` entweder echt implementieren (z. B. `/proc/*/stat` nach fokussiertem TTY/Session scannen) oder den Kommentar/die Doku anpassen, dass Strategie 3 aktuell nicht existiert, um falsche Erwartungen zu vermeiden.
4. LRU-Trim-Query gegen die tatsächliche SQLite-Build-Konfiguration verifizieren; im Zweifel auf eine portable Variante ohne `DELETE ... LIMIT` umstellen (Subquery mit `IN (SELECT id FROM ... ORDER BY ... LIMIT ...)`, wie es an anderer Stelle im Code bereits korrekt gemacht wird, siehe `restore_script_version`-Umfeld).

### 3.4 QuickJS-Sandbox: potenzieller Panic + fehlendes Input-Limit

```rust
const MAX_OUTPUT_BYTES: usize = 512 * 1024;
...
if res.len() > MAX_OUTPUT_BYTES {
    res.truncate(MAX_OUTPUT_BYTES);   // operiert auf String, nicht auf &[u8]
}
```

`String::truncate()` in Rust **panict**, wenn der übergebene Index nicht auf einer UTF-8-Zeichengrenze liegt. Bei einem Skript-Output, dessen 512-KB-Grenze zufällig mitten in einem Mehrbyte-Unicode-Zeichen liegt (z. B. Emoji, kyrillische/asiatische Zeichen), stürzt der gesamte Skript-Ausführungspfad ab.

Zusätzlich: Das spezifizierte **2-MB-Input-Limit** (`sandbox.input_limit_bytes`) ist nirgends im Code geprüft — beliebig große Eingaben werden anstandslos an QuickJS übergeben.

**Auftrag:**
1. `truncate` durch eine char-boundary-sichere Variante ersetzen, z. B.:
   ```rust
   if res.len() > MAX_OUTPUT_BYTES {
       let mut end = MAX_OUTPUT_BYTES;
       while !res.is_char_boundary(end) { end -= 1; }
       res.truncate(end);
   }
   ```
2. Vor dem Aufruf von `run_script_in_sandbox` eine Längenprüfung auf `input_text.len()` gegen das (aus Settings zu ladende) Input-Limit ergänzen, mit klarer `DomainError`-Variante bei Überschreitung.
3. `SCRIPT_TIMEOUT` und `MAX_OUTPUT_BYTES` von Compile-Time-Konstanten auf zur Laufzeit aus `settings` gelesene Werte umstellen — aktuell hat das komplette `sandbox.*`-Einstellungsschema (5 Einträge inkl. „Large Mode" für 10-MB-Inputs) keinerlei Wirkung auf den Code.

### 3.5 SQLite-Schema: fehlende Foreign Keys in `003_clipboard.sql`

Im Gegensatz zu `001_initial.sql` (wo `snippet_tags.snippet_id` korrekt `REFERENCES snippets(id) ON DELETE CASCADE` nutzt), fehlen in `003_clipboard.sql` die entsprechenden Constraints:

```sql
-- IST-Zustand (003_clipboard.sql)
promoted_to_snippet_id TEXT,          -- kein REFERENCES
...
CREATE TABLE clipboard_tags (
  entry_id TEXT NOT NULL,             -- kein REFERENCES
  ...
);
```

Obwohl `PRAGMA foreign_keys = ON` gesetzt ist, greift ohne die `REFERENCES`-Klausel keine referentielle Integrität. Folge: Wird ein Snippet gelöscht, bleibt `promoted_to_snippet_id` als toter Verweis stehen; wird ein Clipboard-Eintrag gelöscht, verbleiben verwaiste `clipboard_tags`-Zeilen.

**Auftrag:** Neue Migration ergänzen, die (a) verwaiste Zeilen einmalig bereinigt und (b) — da SQLite `ALTER TABLE ... ADD CONSTRAINT` nicht unterstützt — entweder per Trigger-basierter Nachbildung der Kaskadierung arbeitet, oder (pragmatischer) die betroffenen Tabellen per `CREATE TABLE ... AS SELECT` + Umbenennung mit korrektem Schema neu aufbaut (Standard-SQLite-Pattern für nachträgliche FK-Ergänzung).

### 3.6 Domain-Modelle sind schmaler als die Spec an mehreren Stellen

| Bereich | Spec fordert | Tatsächlich vorhanden |
|---|---|---|
| `text-stats.ts` (TS-Domain) | `uniqueWordCount`, `sentenceCount`, `avgWordLength`, `longestWord`, `avgLineLength`, `longestLine`, wählbares `TokenizerModel`, Flesch-Kincaid, `topWords` | Nur 8 Basisfelder, hartkodierte Token-Formel `wordCount * 1.3` |
| `diff.ts` (TS-Domain) | `similarity: number`, `unchanged`-Zähler | Fehlen beide |
| `session.ts` | `sidebarWidth`, `previewMode`, `filterState`, `openEditorTabs`, `savedAt` | Fehlen alle fünf |
| `notifications.ts` | Vordefinierte Factories (`snippetSaved`, `snippetCopied`, `transformComplete`, `transformError`, `undoAvailable`, `importComplete`) inkl. `NotificationAction`/Handler | Nur generische `info/success/warning/error`, keine Action-Buttons |
| DiffViewer.svelte | 3 Modi: `unified`, `split`, `inline` | Nur `unified`, `split` |

**Wichtiger Kontrast:** Die **Rust-seitige** `compute_text_stats`-Implementierung ist deutlich vollständiger als die TS-Domain (inkl. `topWords`, `avgSentenceLength`, Flesch-Kincaid) — allerdings mit einer **mathematisch unsauberen** FK-Formel:

```rust
let flesch_kincaid_grade = if word_count > 0 && sentence_count > 0 {
    let grade = 0.39 * (word_count as f32 / sentence_count as f32) + 11.8 * 1.5 - 15.59;
    // "1.5" ist ein hartkodierter Platzhalter für "durchschnittliche Silben pro Wort" —
    // die echte FK-Formel benötigt eine echte Silbenzählung, nicht eine Konstante.
    Some(grade.max(0.0))
} else { None };
```

Da die UI (`SnippetStats.svelte`) den Rust-Wert über IPC bezieht, ist der angezeigte Lesbarkeits-Wert systematisch falsch (er ist für jeden Text bei gleicher Wort/Satz-Ratio identisch, unabhängig von der tatsächlichen Wortkomplexität).

**Auftrag:**
1. TS-Domain-Version von `text-stats.ts` entweder auf Rust-Parität bringen (unwahrscheinlich sinnvoll, da toter Code — siehe 4.1) oder als reine Typdefinition ohne eigene Berechnungslogik führen, um Doppelpflege zu vermeiden.
2. Rust-seitige FK-Formel durch eine echte, einfache Silbenschätzung ersetzen (Standard-Heuristik: Vokalgruppen pro Wort zählen) — Genauigkeit muss nicht perfekt sein, aber die Formel sollte nicht mit einer Konstante rechnen, die den Wortinhalt komplett ignoriert.
3. `NotificationAction`/`handler`-Feld in `AppNotification` ergänzen und die sechs spezifizierten Factory-Funktionen implementieren — das ist Voraussetzung dafür, dass eine "Rückgängig"-Aktion direkt im Toast nach Fix 2.1 sinnvoll angeboten werden kann.
4. `DiffLineDto`/`DiffResult` um `similarity` ergänzen (Rust: `similar::TextDiff` liefert das Verhältnis bereits über `.ratio()`).
5. `inline`-Diff-Modus in `DiffViewer.svelte` ergänzen (zeichenweise Hervorhebung — `similar`-Crate unterstützt das über `TextDiff::from_chars` bereits).

---

## 4. P3 — Technische Schulden, tote Module, Test-Lücken

### 4.1 Totes und fehlerhaftes Modul: `src/lib/ipc/text-analysis.ts`

Dieses Modul wird **von keiner einzigen Datei im gesamten Frontend importiert** (verifiziert per Grep über den kompletten `src/`-Baum). Es dupliziert Funktionalität aus `ipc/snippets.ts`, aber mit **falschen Parameternamen**, die bei Verwendung sofort scheitern würden:

```typescript
// ipc/text-analysis.ts — würde bei Aufruf fehlschlagen:
export async function computeTextStats(text: string): Promise<TextStats> {
  return invoke('compute_text_stats', { text });   // Rust erwartet "content", nicht "text"
}
export async function renderTemplate(templateText: string, variablesJson: string): Promise<string> {
  return invoke('render_template', { templateText, variablesJson });  // Rust erwartet "content", "context", "strict"
}
```

**Auftrag:** Datei komplett entfernen. Die korrekte, tatsächlich genutzte Implementierung liegt bereits in `ipc/snippets.ts` (`computeTextStats(content)`, `parseTemplate(content)`, `renderTemplate(content, context, strict)`).

### 4.2 Fehlende `scripts_fts`-Volltextsuche

CLAUDE.md § 6 listet `snippets`, `scripts`, `clipboard_history` als FTS5-relevante Tabellen. Tatsächlich existiert nur `snippets_fts` und `clipboard_fts`. `list_scripts()` hat keinerlei Suchparameter — jede Skriptsuche im UI (sofern vorhanden) muss aktuell rein clientseitig über die vollständig geladene Liste erfolgen.

**Auftrag:** Falls eine serverseitige Skriptsuche gewünscht ist (bei wachsender Skriptbibliothek relevant): neue Migration mit `scripts_fts`-Virtual-Table + Triggern nach dem Muster von `001_initial.sql`, `list_scripts` um optionalen `search`-Parameter erweitern.

### 4.3 `isSafetyCritical`-Flag ohne jede Wirkung

Anhang A markiert `redact_sensitive` und `strip_pii` als `SafetyCritical: true`. Das Feld existiert im TS-Domain-Typ (`script.ts`), wird aber:
- nirgends im Rust-Backend abgebildet,
- nirgends im Frontend zur Anzeige einer Warnung/Bestätigung vor destruktiver Anwendung genutzt.

**Auftrag:** Vor Ausführung eines als `isSafetyCritical` markierten Builtins/Skripts (in der UI-Schicht, z. B. `ScriptTester.svelte` / Transform-Trigger-Komponenten) einen Bestätigungsdialog einblenden, der klarmacht, dass der Vorgang potenziell destruktiv ist (z. B. PII-Entfernung kann bei False Positives legitime Daten zerstören).

### 4.4 Testabdeckung: 10 von 14 Domain-Core-Dateien ohne Unit-Test

Vorhandene Tests (alle grün, 47 Assertions, `npx vitest run` erfolgreich verifiziert):

```
✓ src/lib/domain/template.test.ts       (29 Tests)
✓ src/lib/domain/script.test.ts         (11 Tests)
✓ src/lib/domain/undo.test.ts           (4 Tests)
✓ src/lib/domain/clipboard-entry.test.ts (3 Tests)
```

Fehlend, obwohl CLAUDE.md „ohne DB/QuickJS/Browser testbar" ausdrücklich für den Domain Core fordert:

```
adts.ts        ← das fundamentale Result/Option-Typsystem, auf dem alles aufbaut
snippet.ts     ← zentrale Kernentität der Anwendung
pipeline.ts
filter.ts
diff.ts
session.ts
notifications.ts
text-stats.ts
import-export.ts
errors.ts      ← DomainError.describe() sollte auf Vollständigkeit/Erschöpfung geprüft werden
```

**Auftrag:** Reihenfolge nach Kritikalität: `adts.ts` zuerst (Fundament), dann `snippet.ts`, dann die übrigen. Ziel: Jede öffentliche Funktion mindestens ein Happy-Path- und ein Error-Path-Testfall.

### 4.5 Fehlendes npm-`test`-Script

`package.json` enthält kein `"test"`-Skript, obwohl `README.md` `npx vitest run` als offiziellen Testbefehl dokumentiert. Für CI-Konsistenz und um Onboarding zu vereinfachen:

**Auftrag:** In `package.json` ergänzen:
```json
"scripts": {
  "test": "vitest run",
  "test:watch": "vitest"
}
```

### 4.6 Referenzierte Spezifikationsdatei existiert nicht

`CLAUDE.md` verweist kanonisch und wiederholt auf `docs/textforge-interface-spec-v2.1.md` — dieser Pfad existiert im Repository nicht. Die tatsächliche Spezifikation liegt als `SPECIFICATION.1.md` im Projekt-Root.

**Auftrag:** Entweder (a) `SPECIFICATION.1.md` nach `docs/textforge-interface-spec-v2.1.md` verschieben und alle Referenzen in `CLAUDE.md`/`README.md` beibehalten, oder (b) alle Pfadverweise in `CLAUDE.md` auf den tatsächlichen Speicherort korrigieren. Für ein LLM-gestütztes Projekt, dessen zentrale Arbeitsanweisung explizit "Spec ist Wahrheit" postuliert, ist ein kaputter Verweis auf die Spec selbst ein besonders hohes Risiko für Fehlimplementierungen durch nachfolgende Bearbeiter (menschlich wie KI-Agent).

---

## 5. Positive Befunde — was bereits solide ist

Um ein ausgewogenes Bild zu zeichnen:

- **Alle 81 in Anhang A spezifizierten Builtin-Transformationen sind 1:1 im Rust-Code vorhanden** (`commands/builtins.rs`), inklusive korrekter Kategorisierung. Das ist der umfangreichste Einzelbereich der Spezifikation und vollständig umgesetzt.
- **Template-Engine** (Filter, Conditionals, Loops, Spezialvariablen) ist sowohl in Rust als auch TypeScript funktional weit vollständiger, als die (veralteten) Spec-Statusmarker vermuten lassen.
- **QuickJS-Sandbox-Grundgerüst** (Timeout via `tokio::time::timeout`, `console.log`-Capture, umfangreiches `utils.*`-Prelude mit ca. 40 Hilfsfunktionen) ist durchdacht und robust gegen Panics in der Skriptausführung selbst (abgesehen vom in 3.4 genannten Truncate-Bug).
- **Import mit Konfliktpolicies** (`skip`/`overwrite`/`rename`) inkl. SHA-256-Checksummenprüfung ist entgegen dem Spec-Marker bereits vollständig implementiert.
- **FTS5-Volltextsuche** für Snippets und Clipboard-Verlauf ist korrekt mit Triggern abgesichert und wird in den Such-Queries tatsächlich genutzt (kein Bypass, wie ein Marker vermuten ließ).
- **Append-only-Migrationsdisziplin** (INVARIANT-F) wird über 8 Migrationsdateien hinweg eingehalten — mit der einen kritischen Ausnahme aus 1.1, die vermutlich ein Merge-/Reihenfolgefehler und kein bewusstes Invarianten-Brechen war.
- Die 47 vorhandenen Frontend-Domain-Tests sind sauber geschrieben und laufen fehlerfrei.

---

## 6. Priorisierte Umsetzungsreihenfolge (Empfehlung)

```
SPRINT 0 — App lauffähig machen (P0)
  □ 1.1  Migrationskonflikt 002/007 beheben (neue Migration 009)
  □ 1.2  tests/clipboard.rs: size_bytes aus INSERT entfernen
  □ 1.3  Frischinstallations-Smoke-Test in CI aufnehmen
  → Deliverable: `cargo test` grün, App startet auf leerem System

SPRINT 1 — Kernversprechen der App einlösen (P1)
  □ 2.1  Undo/Redo tatsächlich befüllen (mind. Snippet-CRUD + Bulk)
  □ 2.2  restore_script_version Frontend/Backend-Mismatch fixen
  □ 2.3  Workspace-Session an Frontend anschließen
  □ 2.4  bulk_transform / bulk_export implementieren
  □ 2.5  Export-Formate jenseits tfbundle ergänzen
  → Deliverable: Alle in der UI sichtbaren Buttons tun tatsächlich das, was sie versprechen

SPRINT 2 — Spec-Konformität (P2)
  □ 3.1  Fehlende Tauri-Events (Pipeline/Bulk/Import-Progress)
  □ 3.2  Zentrales Keyboard-Shortcut-Registry + Command Palette
  □ 3.3  Clipboard-Monitor: Config nutzen, Subprozess-Cleanup, procfs-Fallback
  □ 3.4  Sandbox: Truncate-Panic fixen, Input-Limit + Settings-Anbindung
  □ 3.5  Foreign Keys in clipboard-Migration nachrüsten
  □ 3.6  Domain-Modelle (TextStats, Diff, Session, Notifications) auf Spec-Umfang bringen
  → Deliverable: Verhalten entspricht durchgängig SPECIFICATION.1.md

SPRINT 3 — Aufräumen (P3)
  □ 4.1  ipc/text-analysis.ts entfernen
  □ 4.2  scripts_fts ergänzen (falls Suche gewünscht)
  □ 4.3  isSafetyCritical-Bestätigungsdialog
  □ 4.4  Domain-Core-Testabdeckung auf 100% der Dateien
  □ 4.5  npm test-Script ergänzen
  □ 4.6  docs/-Pfad-Referenz in CLAUDE.md korrigieren
  → Deliverable: Kein totes/inkonsistentes Modul mehr im Repository
```

**Wichtiger Hinweis zur Vorgehensweise:** CLAUDE.md schreibt eine strikte Phasenreihenfolge vor ("Wenn du mitten in Phase 1 gebeten wirst, eine Pipeline zu bauen: ablehnen"). Diese Analyse zeigt, dass der Code faktisch bereits **alle vier Phasen parallel** angefasst hat, ohne dass Phase 1 (Clipboard-Kern) vollständig fehlerfrei abgeschlossen wurde (siehe 3.3). Für die weitere Arbeit empfiehlt sich, Sprint 0 als "Phase 1 nachträglich abschließen" zu behandeln, bevor an Sprint 2/3-Themen aus späteren Phasen weitergearbeitet wird — im Sinne der eigenen Projektregeln.

---

## 7. Offene Rückfragen an den Projektinhaber (vor Sprint 1 zu klären)

1. **`docs/textforge-interface-spec-v2.1.md` vs. `SPECIFICATION.1.md`**: Welche Datei ist tatsächlich kanonisch? Gibt es eine neuere Version außerhalb dieses Repo-Snapshots?
2. **`favorite` vs. `is_favorite`** (Migration 007 vs. 002): Welches Feld soll dauerhaft bestehen bleiben?
3. **Export-Formatname**: `'bundle'` (Spec) vs. `"tfbundle"` (Code) — welcher Bezeichner ist die Zielbenennung für die IPC-Schnittstelle?
4. **Undo-Scope**: Soll Undo/Redo für *alle* zwölf spezifizierten Aktionstypen gleichzeitig angebunden werden, oder ist eine stufenweise Einführung (zuerst Snippet-CRUD, dann Bulk, dann Scripts/Pipelines) gewünscht?
