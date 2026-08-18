CREATE TABLE IF NOT EXISTS clipboard_history (
  id               TEXT PRIMARY KEY,
  content          TEXT NOT NULL,
  content_hash     TEXT NOT NULL UNIQUE,
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

CREATE INDEX IF NOT EXISTS idx_clip_captured  ON clipboard_history(captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_clip_pinned    ON clipboard_history(is_pinned, captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_clip_type      ON clipboard_history(content_type);
CREATE INDEX IF NOT EXISTS idx_clip_source    ON clipboard_history(source_app);

CREATE TABLE IF NOT EXISTS clipboard_tags (
  entry_id TEXT NOT NULL,
  tag      TEXT NOT NULL,
  PRIMARY KEY (entry_id, tag)
);

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
