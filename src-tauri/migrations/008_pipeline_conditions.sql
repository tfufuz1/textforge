-- § 4 Pipeline: failure_policy + condition_json (INVARIANT-F: append-only)
-- Fügt FailurePolicy und PipelineCondition-Unterstützung zu pipeline_steps hinzu

ALTER TABLE pipeline_steps ADD COLUMN failure_policy TEXT NOT NULL DEFAULT 'abort';
-- Erlaubte Werte: 'abort' | 'warn' | 'passthrough'

ALTER TABLE pipeline_steps ADD COLUMN condition_json TEXT;
-- JSON-Struktur: { "_type": "size_gt", "bytes": 1000 } etc.
-- Wenn NULL: kein Condition-Check, Schritt wird immer ausgeführt
