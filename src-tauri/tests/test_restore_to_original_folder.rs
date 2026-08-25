use textforge::db::init_db;
use textforge::commands::snippets::{
    create_folder, create_snippet, get_snippet, restore_snippet, trash_snippet, CreateSnippetDto
};
use textforge::AppState;
use tempfile::NamedTempFile;
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_restore_to_original_folder() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
        clipboard_config: std::sync::RwLock::new(textforge::clipboard::ClipboardMonitorConfig::default()),
    });
    let state = app.state::<AppState>();

    // 1. Create a folder
    let folder = create_folder("Work".to_string(), None, Some("folder".to_string()), state.clone()).await.unwrap();

    // 2. Create a snippet inside that folder
    let draft = CreateSnippetDto {
        title: "Folder Snippet".to_string(),
        content: "Content in folder".to_string(),
        content_type: None,
        tags: None,
        folder_id: Some(folder.id.clone()),
    };
    let created = create_snippet(draft, state.clone()).await.unwrap();
    assert_eq!(created.location_type, "folder");
    assert_eq!(created.folder_id, Some(folder.id.clone()));

    // 3. Move snippet to trash
    trash_snippet(created.id.clone(), state.clone()).await.unwrap();
    let trashed = get_snippet(created.id.clone(), state.clone()).await.unwrap();
    assert_eq!(trashed.location_type, "trash");

    // 4. Restore snippet
    restore_snippet(created.id.clone(), state.clone()).await.unwrap();
    let restored = get_snippet(created.id.clone(), state.clone()).await.unwrap();
    assert_eq!(restored.location_type, "folder");
    assert_eq!(restored.folder_id, Some(folder.id.clone()));
}
