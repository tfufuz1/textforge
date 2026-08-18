-- 006_scripts_v2_fields.sql
ALTER TABLE scripts ADD COLUMN parameters_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE scripts ADD COLUMN tags_json TEXT NOT NULL DEFAULT '[]';
