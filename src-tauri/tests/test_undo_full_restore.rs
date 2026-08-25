use textforge::db::init_db;
use textforge::commands::snippets::{
    create_folder, create_snippet, get_snippet, update_snippet, CreateSnippetDto, UpdateSnippetDto
};
use textforge::commands::undo::{undo, redo};
use textforge::AppState;
use tempfile::NamedTempFile;
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_undo_restores_all_snippet_fields() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
    });
    let state = app.state::<AppState>();

    // 1. Create initial folder and snippet
    let folder1 = create_folder("Folder 1".to_string(), None, Some("folder".to_string()), state.clone()).await.unwrap();
    let folder2 = create_folder("Folder 2".to_string(), None, Some("folder".to_string()), state.clone()).await.unwrap();

    let create_draft = CreateSnippetDto {
        title: "Original Title".to_string(),
        content: "Original Content {{var}}".to_string(),
        content_type: Some("markdown".to_string()),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        folder_id: Some(folder1.id.clone()),
    };
    let initial_snippet = create_snippet(create_draft, state.clone()).await.unwrap();

    // Set initial pinned, favorite, color
    let update_initial = UpdateSnippetDto {
        title: None,
        content: None,
        content_type: None,
        tags: None,
        is_pinned: Some(true),
        is_favorite: Some(true),
        color: Some("#FF0000".to_string()),
        folder_id: None,
    };
    let original = update_snippet(initial_snippet.id.clone(), update_initial, state.clone()).await.unwrap();
    assert_eq!(original.title, "Original Title");
    assert_eq!(original.content, "Original Content {{var}}");
    assert_eq!(original.content_type, "markdown");
    assert_eq!(original.tags, vec!["tag1".to_string(), "tag2".to_string()]);
    assert_eq!(original.folder_id, Some(folder1.id.clone()));
    assert_eq!(original.is_pinned, true);
    assert_eq!(original.is_favorite, true);
    assert_eq!(original.color, Some("#FF0000".to_string()));
    assert_eq!(original.is_template, true);

    // 2. Perform an update changing ALL fields
    let update_draft = UpdateSnippetDto {
        title: Some("Updated Title".to_string()),
        content: Some("Updated Content".to_string()),
        content_type: Some("plain_text".to_string()),
        tags: Some(vec!["tag3".to_string()]),
        is_pinned: Some(false),
        is_favorite: Some(false),
        color: Some("#00FF00".to_string()),
        folder_id: Some(folder2.id.clone()),
    };
    let updated = update_snippet(original.id.clone(), update_draft, state.clone()).await.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.content, "Updated Content");
    assert_eq!(updated.content_type, "plain_text");
    assert_eq!(updated.tags, vec!["tag3".to_string()]);
    assert_eq!(updated.is_pinned, false);
    assert_eq!(updated.is_favorite, false);
    assert_eq!(updated.color, Some("#00FF00".to_string()));

    // 3. Undo the update
    undo(state.clone()).await.unwrap();

    // 4. Verify snippet is restored to original state
    let restored = get_snippet(original.id.clone(), state.clone()).await.unwrap();
    assert_eq!(restored.title, "Original Title");
    assert_eq!(restored.content, "Original Content {{var}}");
    assert_eq!(restored.content_type, "markdown");
    assert_eq!(restored.tags, vec!["tag1".to_string(), "tag2".to_string()]);
    assert_eq!(restored.folder_id, Some(folder1.id.clone()));
    assert_eq!(restored.is_pinned, true);
    assert_eq!(restored.is_favorite, true);
    assert_eq!(restored.color, Some("#FF0000".to_string()));
    assert_eq!(restored.is_template, true);
}

#[tokio::test]
async fn test_redo_create_restores_tags() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
    });
    let state = app.state::<AppState>();

    // 1. Create a snippet with tags
    let draft = CreateSnippetDto {
        title: "New Snippet".to_string(),
        content: "Some content".to_string(),
        content_type: Some("plain_text".to_string()),
        tags: Some(vec!["rust".to_string(), "tauri".to_string()]),
        folder_id: None,
    };
    let created = create_snippet(draft, state.clone()).await.unwrap();
    assert_eq!(created.tags, vec!["rust".to_string(), "tauri".to_string()]);

    // 2. Undo snippet creation (deletes the snippet)
    undo(state.clone()).await.unwrap();
    assert!(get_snippet(created.id.clone(), state.clone()).await.is_err());

    // 3. Redo snippet creation (re-creates the snippet with tags)
    redo(state.clone()).await.unwrap();

    // 4. Verify snippet and tags are restored
    let redone = get_snippet(created.id.clone(), state.clone()).await.unwrap();
    assert_eq!(redone.title, "New Snippet");
    assert_eq!(redone.content, "Some content");
    assert_eq!(redone.tags, vec!["rust".to_string(), "tauri".to_string()]);
}
