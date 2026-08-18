CREATE TABLE IF NOT EXISTS snippets (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'plain_text',
  location_type TEXT NOT NULL DEFAULT 'inbox',
  location_folder_id TEXT,
  deleted_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  usage_count INTEGER NOT NULL DEFAULT 0,
  is_pinned INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS snippet_tags (
  snippet_id TEXT NOT NULL REFERENCES snippets(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  PRIMARY KEY (snippet_id, tag)
);

CREATE VIRTUAL TABLE IF NOT EXISTS snippets_fts USING fts5(
  title,
  content,
  content='snippets',
  content_rowid='rowid',
  tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS snip_fts_insert AFTER INSERT ON snippets
  BEGIN INSERT INTO snippets_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content); END;
CREATE TRIGGER IF NOT EXISTS snip_fts_update AFTER UPDATE ON snippets
  BEGIN
    DELETE FROM snippets_fts WHERE rowid = old.rowid;
    INSERT INTO snippets_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
  END;
CREATE TRIGGER IF NOT EXISTS snip_fts_delete AFTER DELETE ON snippets
  BEGIN DELETE FROM snippets_fts WHERE rowid = old.rowid; END;

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
