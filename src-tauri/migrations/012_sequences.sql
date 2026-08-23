-- 012_sequences.sql
CREATE TABLE IF NOT EXISTS sequences (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  separator TEXT NOT NULL,
  favorite INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sequence_items (
  id TEXT PRIMARY KEY,
  sequence_id TEXT NOT NULL REFERENCES sequences(id) ON DELETE CASCADE,
  order_index INTEGER NOT NULL,
  ref_type TEXT NOT NULL,
  ref_id TEXT,
  literal_text TEXT,
  pipeline_id TEXT,
  prefix_override TEXT,
  suffix_override TEXT,
  enabled INTEGER NOT NULL DEFAULT 1
);
