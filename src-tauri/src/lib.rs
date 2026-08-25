pub mod automation;
pub mod clipboard;
pub mod commands;
pub mod db;
pub mod sandbox;

use std::sync::Mutex;

pub type SharedRegexCache = Mutex<lru::LruCache<(String, String), regex::Regex>>;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub undo_stack: commands::undo::SharedUndoStack,
    pub regex_cache: SharedRegexCache,
    pub clipboard_config: std::sync::RwLock<crate::clipboard::ClipboardMonitorConfig>,
}
