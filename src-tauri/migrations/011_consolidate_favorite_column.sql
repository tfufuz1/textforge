-- 011_consolidate_favorite_column.sql
-- Konsolidierung auf is_favorite als kanonische Spalte fuer snippets.
-- Die Spalte 'favorite' in 'snippets' ist deprecated.

UPDATE snippets SET is_favorite = favorite WHERE favorite = 1 AND is_favorite = 0;
