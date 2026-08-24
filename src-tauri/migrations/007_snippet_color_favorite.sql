-- 007_snippet_color_favorite.sql
-- Gemäß INVARIANT-F: append-only, keine bestehenden Migrations-Dateien ändern

-- Füge color und favorite Felder zum snippets table hinzu
-- Füge favorite Feld zum snippets table hinzu (color ist bereits in 002_v2_extensions.sql vorhanden)
ALTER TABLE snippets ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
