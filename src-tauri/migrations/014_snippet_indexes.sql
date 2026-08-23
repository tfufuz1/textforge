-- Indexes for snippets table to optimize list_snippets queries
CREATE INDEX IF NOT EXISTS idx_snippets_loc_pinned_updated ON snippets(location_type, is_pinned DESC, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_snippets_loc_pinned_created ON snippets(location_type, is_pinned DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_snippets_loc_pinned_usage ON snippets(location_type, is_pinned DESC, usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_snippets_loc_pinned_title ON snippets(location_type, is_pinned DESC, title ASC);
CREATE INDEX IF NOT EXISTS idx_snippets_folder ON snippets(location_folder_id) WHERE location_folder_id IS NOT NULL;
