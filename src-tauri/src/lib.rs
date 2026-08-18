pub mod clipboard;
pub mod commands;
pub mod db;
pub mod sandbox;

use std::sync::Mutex;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub undo_stack: commands::undo::SharedUndoStack,
}
