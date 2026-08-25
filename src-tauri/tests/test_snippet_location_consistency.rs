use textforge::db::init_db;
use textforge::commands::snippets::{create_snippet, list_snippets, CreateSnippetDto, SnippetFilterDto};
use textforge::AppState;
use tempfile::NamedTempFile;
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_snippet_location_consistency() {
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

    // 1. Create a snippet without folder_id (default location)
    let draft = CreateSnippetDto {
        title: "Test Inbox Snippet".to_string(),
        content: "Some content".to_string(),
        content_type: None,
        tags: None,
        folder_id: None,
    };
    let created = create_snippet(draft, state.clone()).await.unwrap();
    assert_eq!(created.location_type, "inbox");

    // 2. Query list_snippets with location_type = "inbox"
    let filter = SnippetFilterDto {
        location_type: Some("inbox".to_string()),
        ..Default::default()
    };
    let res = list_snippets(Some(filter), state.clone()).await.unwrap();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].id, created.id);
}
