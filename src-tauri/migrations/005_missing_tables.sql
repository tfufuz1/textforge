-- 005_missing_tables.sql
-- Folders table referenced by snippets location_folder_id
CREATE TABLE IF NOT EXISTS folders (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  parent_id TEXT REFERENCES folders(id) ON DELETE SET NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  icon TEXT NOT NULL DEFAULT 'folder',
  color TEXT NOT NULL DEFAULT '#64748b',
  created_at INTEGER NOT NULL
);

-- Script versions history
CREATE TABLE IF NOT EXISTS script_versions (
  id TEXT PRIMARY KEY,
  script_id TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
  version INTEGER NOT NULL,
  js_code TEXT,
  regex_pattern TEXT,
  regex_replacement TEXT,
  regex_flags TEXT NOT NULL DEFAULT 'g',
  parameters_json TEXT NOT NULL DEFAULT '{}',
  change_note TEXT NOT NULL DEFAULT '',
  saved_at INTEGER NOT NULL,
  UNIQUE(script_id, version)
);

CREATE INDEX IF NOT EXISTS idx_scrver_script ON script_versions(script_id, version DESC);

-- Trigger: keep max 20 versions per script
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

-- Template variables extracted from snippets
CREATE TABLE IF NOT EXISTS template_variables (
  snippet_id TEXT NOT NULL REFERENCES snippets(id) ON DELETE CASCADE,
  variable TEXT NOT NULL,
  has_default INTEGER NOT NULL DEFAULT 0,
  default_val TEXT NOT NULL DEFAULT '',
  is_required INTEGER NOT NULL DEFAULT 0,
  occurrences INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (snippet_id, variable)
);

-- Alter scripts & pipelines for v2 fields if missing
ALTER TABLE scripts ADD COLUMN current_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE scripts ADD COLUMN color TEXT NOT NULL DEFAULT '#6366f1';
ALTER TABLE pipelines ADD COLUMN is_template INTEGER NOT NULL DEFAULT 0;

-- Default Settings Insertions
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('clipboard.max_entries', '500', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('clipboard.min_length', '1', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('clipboard.enabled', 'true', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('undo.max_stack_size', '50', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('ui.theme', 'dark', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('ui.font_size', '14', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('ui.diff_mode', 'split', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('template.default_var_syntax', 'mustache', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('export.pretty_json', 'true', 1700000000);
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('session.restore_on_start', 'true', 1700000000);
