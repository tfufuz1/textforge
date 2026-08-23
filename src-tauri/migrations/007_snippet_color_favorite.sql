-- 007_snippet_color_favorite.sql
-- Gemäß INVARIANT-F: append-only, keine bestehenden Migrations-Dateien ändern

-- Füge color und favorite Felder zum snippets table hinzu
ALTER TABLE snippets ADD COLUMN color TEXT;
ALTER TABLE snippets ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
