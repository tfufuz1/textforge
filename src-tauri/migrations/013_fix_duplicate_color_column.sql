-- 013_fix_duplicate_color_column.sql
-- Gemäß INVARIANT-F: append-only, keine bestehenden Migrations-Dateien (001-012) verändern.
--
-- Hinweis zur Spalte 'color':
-- Spalte 'snippets.color' wurde bereits in 002_v2_extensions.sql angelegt.
-- In dieser Migration wird 'color' daher NICHT erneut hinzugefügt.
--
-- Konsolidierung der Favoriten-Spalten:
-- 'is_favorite' (aus 002_v2_extensions.sql) ist die kanonische Spalte.
-- 'favorite' (aus 007_snippet_color_favorite.sql) ist deprecated.
-- Vorhandene Daten aus 'favorite' werden nach 'is_favorite' migriert.

UPDATE snippets SET is_favorite = favorite WHERE favorite = 1 AND is_favorite = 0;
