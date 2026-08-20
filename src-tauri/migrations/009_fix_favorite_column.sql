-- 009_fix_favorite_column.sql
-- Synchronize favorite values to is_favorite to unify snippets favorite column

UPDATE snippets SET is_favorite = favorite WHERE favorite = 1 AND is_favorite = 0;
UPDATE snippets SET favorite = is_favorite WHERE is_favorite = 1 AND favorite = 0;
