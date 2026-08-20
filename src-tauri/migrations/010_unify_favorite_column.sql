-- 010_unify_favorite_column.sql
-- Synchronize values between favorite and is_favorite columns for backward compatibility

UPDATE snippets SET is_favorite = favorite WHERE favorite = 1 AND is_favorite = 0;
UPDATE snippets SET favorite = is_favorite WHERE is_favorite = 1 AND favorite = 0;
