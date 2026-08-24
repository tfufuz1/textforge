-- collection_tabs: Benutzerdefinierten Reiter für Snippet-Sammlungen
CREATE TABLE IF NOT EXISTS collection_tabs (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  icon        TEXT,
  color       TEXT,
  sort_order  INTEGER NOT NULL DEFAULT 0,
  kind        TEXT NOT NULL DEFAULT 'manual', -- 'manual' | 'smart' | 'clipboard_capture'
  kind_config TEXT,                            -- JSON für Smart-Tab-Kriterien
  is_pinned   INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collection_tab_members (
  tab_id    TEXT NOT NULL REFERENCES collection_tabs(id) ON DELETE CASCADE,
  item_kind TEXT NOT NULL, -- 'snippet' | 'clipboard'
  item_id   TEXT NOT NULL,
  added_at  INTEGER NOT NULL,
  PRIMARY KEY (tab_id, item_kind, item_id)
);

CREATE INDEX IF NOT EXISTS idx_tab_members_tab ON collection_tab_members(tab_id);

-- automation_rules: Trigger-basierte Automatisierungsregeln
CREATE TABLE IF NOT EXISTS automation_rules (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  enabled    INTEGER NOT NULL DEFAULT 1,
  trigger    TEXT NOT NULL,    -- JSON: AutomationTrigger
  condition  TEXT,             -- optionale JSON-Bedingung
  script_id  TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_automation_enabled ON automation_rules(enabled, sort_order);

-- Tag-Registry: cross-entity Tags für Snippets, Scripts, Pipelines
CREATE TABLE IF NOT EXISTS script_tags (
  script_id TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
  tag       TEXT NOT NULL,
  PRIMARY KEY (script_id, tag)
);

CREATE TABLE IF NOT EXISTS pipeline_tags (
  pipeline_id TEXT NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
  tag         TEXT NOT NULL,
  PRIMARY KEY (pipeline_id, tag)
);

CREATE TABLE IF NOT EXISTS tag_colors (
  tag_name   TEXT PRIMARY KEY,
  color      TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

-- folders: updated_at-Spalte fehlt in Migration 005, wird hier nachgereicht
ALTER TABLE folders ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;
UPDATE folders SET updated_at = created_at WHERE updated_at = 0;
