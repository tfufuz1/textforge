INSERT OR IGNORE INTO settings (key, value, updated_at)
VALUES ('clipboard.dedup_window_ms', '500', 1700000000);

-- UNIQUE Constraint auf content_hash entfernen, damit zeitfensterbasierte Duplikate erlaubt sind
CREATE TABLE IF NOT EXISTS clipboard_history_new (
  id               TEXT PRIMARY KEY,
  content          TEXT NOT NULL,
  content_hash     TEXT NOT NULL,
  content_type     TEXT NOT NULL DEFAULT 'plain_text',
  source_app       TEXT,
  captured_at      INTEGER NOT NULL,
  size_bytes       INTEGER GENERATED ALWAYS AS (length(content)) VIRTUAL,
  line_count       INTEGER GENERATED ALWAYS AS (
                     length(content) - length(replace(content, char(10), '')) + 1
                   ) VIRTUAL,
  is_pinned        INTEGER NOT NULL DEFAULT 0,
  promoted_to_snippet_id TEXT
);

INSERT INTO clipboard_history_new (id, content, content_hash, content_type, source_app, captured_at, is_pinned, promoted_to_snippet_id)
SELECT id, content, content_hash, content_type, source_app, captured_at, is_pinned, promoted_to_snippet_id FROM clipboard_history;

DROP TABLE clipboard_history;
ALTER TABLE clipboard_history_new RENAME TO clipboard_history;

CREATE INDEX IF NOT EXISTS idx_clip_captured  ON clipboard_history(captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_clip_pinned    ON clipboard_history(is_pinned, captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_clip_type      ON clipboard_history(content_type);
CREATE INDEX IF NOT EXISTS idx_clip_source    ON clipboard_history(source_app);
CREATE INDEX IF NOT EXISTS idx_clip_hash      ON clipboard_history(content_hash);

DROP TRIGGER IF EXISTS clip_fts_insert;
DROP TRIGGER IF EXISTS clip_fts_update;
DROP TRIGGER IF EXISTS clip_fts_delete;

CREATE TRIGGER clip_fts_insert AFTER INSERT ON clipboard_history
  BEGIN INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content); END;
CREATE TRIGGER clip_fts_update AFTER UPDATE ON clipboard_history
  BEGIN
    DELETE FROM clipboard_fts WHERE rowid = old.rowid;
    INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content);
  END;
CREATE TRIGGER clip_fts_delete AFTER DELETE ON clipboard_history
  BEGIN DELETE FROM clipboard_fts WHERE rowid = old.rowid; END;
