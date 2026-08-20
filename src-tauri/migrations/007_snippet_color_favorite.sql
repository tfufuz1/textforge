-- 007_snippet_color_favorite.sql
-- Gemäß INVARIANT-F: append-only, keine bestehenden Migrations-Dateien ändern

-- Füge color und favorite Felder zum snippets table hinzu
-- Füge favorite Feld zum snippets table hinzu (color existiert bereits aus 002_v2_extensions.sql)
ALTER TABLE snippets ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
