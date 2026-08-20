use tauri::Manager;
use std::path::PathBuf;
use std::sync::Mutex;
use textforge::{AppState, clipboard, commands, db};


// STATUS: Implemented (Phase 1)
/// Attempt to migrate old `app.db` from CWD into the proper app data directory.
/// This is a one-time, best-effort operation — failures are silently ignored.
fn migrate_legacy_db(target: &std::path::Path) {
    if target.exists() {
        return; // new DB already exists, nothing to migrate
    }
    let legacy = PathBuf::from("app.db");
    if legacy.exists() {
        if let Err(e) = std::fs::copy(&legacy, target) {
            eprintln!("Note: could not migrate legacy app.db: {}", e);
        } else {
            eprintln!("Migrated legacy app.db → {}", target.display());
        }
    }
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // STATUS: Implemented (Phase 1 - SQLite Database Initialization)
                // Initialize DB in platform-appropriate app data directory
                let db_dir = app_handle.path().app_data_dir()
                    .expect("Failed to resolve app data directory");
                std::fs::create_dir_all(&db_dir).ok();
                let db_path = db_dir.join("textforge.db");

                // One-time migration of legacy app.db from CWD
                migrate_legacy_db(&db_path);

                let db = db::init_db(&db_path).await.expect("Failed to initialize db");
                
                app_handle.manage(AppState {
                    db: db.clone(),
                    undo_stack: Mutex::new(commands::undo::UndoStack::new()),
                });

                // STATUS: Implemented (Phase 1 - Wayland/arboard Clipboard Monitor)
                // Start monitor
                if let Err(e) = clipboard::start_monitor(app_handle.clone()).await {
                    eprintln!("Clipboard monitor error: {:?}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // STATUS: Implemented (Phase 1/2 - Clipboard IPC Commands)
            commands::clipboard::list_clipboard_history,
            commands::clipboard::get_clipboard_entry,
            commands::clipboard::pin_clipboard_entry,
            commands::clipboard::promote_clipboard_to_snippet,
            commands::clipboard::delete_clipboard_entry,
            commands::clipboard::clear_clipboard_history,
            commands::clipboard::read_clipboard_now,
            commands::clipboard::write_to_clipboard,

            // STATUS: Implemented (Phase 2/3 - Snippets, Folders, & Templates IPC Commands)
            commands::snippets::list_snippets,
            commands::snippets::get_snippet,
            commands::snippets::create_snippet,
            commands::snippets::update_snippet,
            commands::snippets::duplicate_snippet,
            commands::snippets::trash_snippet,
            commands::snippets::restore_snippet,
            commands::snippets::delete_snippet_permanently,
            commands::snippets::empty_trash,
            commands::snippets::list_all_tags,
            commands::snippets::list_folders,
            commands::snippets::create_folder,
            commands::snippets::rename_folder,
            commands::snippets::delete_folder,
            commands::snippets::compute_text_stats,
            commands::snippets::parse_template,
            commands::snippets::render_template,
            commands::snippets::compute_diff,
            commands::snippets::save_script_version,
            commands::snippets::list_script_versions,
            commands::snippets::restore_script_version,

            // STATUS: Implemented (Phase 4 - Bulk Operations IPC Commands)
            commands::bulk::execute_bulk_operation,

            // STATUS: Implemented (Phase 2 - Undo/Redo Engine IPC Commands)
            commands::undo::undo,
            commands::undo::redo,
            commands::undo::get_undo_state,
            commands::undo::push_undo_entry,

            // STATUS: Implemented (Phase 3 - JavaScript Sandbox & Pipeline Transformation IPC Commands)
            commands::transform::execute_script,
            commands::transform::execute_builtin,
            commands::transform::run_pipeline,
            commands::transform::list_scripts,
            commands::transform::get_script,
            commands::transform::create_script,
            commands::transform::update_script,
            commands::transform::delete_script,
            commands::transform::list_pipelines,
            commands::transform::get_pipeline,
            commands::transform::create_pipeline,
            commands::transform::update_pipeline,
            commands::transform::delete_pipeline,
            commands::transform::add_pipeline_step,
            commands::transform::remove_pipeline_step,
            commands::transform::reorder_pipeline_steps,
            commands::transform::toggle_pipeline_step,

            // STATUS: Implemented (Phase 4 - Import/Export IPC Commands)
            commands::import_export::export_data,
            commands::import_export::import_data,
            commands::import_export::preview_import,

            // STATUS: Implemented (Phase 1/4 - Settings, Sessions, & Stats IPC Commands)
            commands::settings::get_all_settings,
            commands::settings::set_setting,
            commands::settings::get_workspace_session,
            commands::settings::save_workspace_session,
            commands::settings::get_database_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
