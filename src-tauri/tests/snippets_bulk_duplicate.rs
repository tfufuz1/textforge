use textforge::db::init_db;
use textforge::commands::snippets::duplicate_snippets_bulk;
use textforge::AppState;
use tempfile::NamedTempFile;
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_bulk_duplicate_200_snippets_performance_and_roundtrips() {
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

    // 1. Insert 200 original snippets
    let now = chrono::Utc::now().timestamp_millis();
    let mut ids = Vec::new();
    for i in 0..200 {
        let id = format!("snip-orig-{}", i);
        let title = format!("Prompt Component {:03}", i);
        let content = format!("Content for snippet {}", i);
        sqlx::query(
            "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&title)
        .bind(&content)
        .bind("plain_text")
        .bind("inbox")
        .bind(now)
        .bind(now)
        .execute(&db)
        .await
        .unwrap();
        ids.push(id);
    }

    // Measure time and execution of duplicate_snippets_bulk
    let start = std::time::Instant::now();
    let res = duplicate_snippets_bulk(ids.clone(), None, state.clone()).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(res.succeeded.len(), 200);
    assert_eq!(res.failed.len(), 0);
    println!("Duplicated 200 snippets in {:?}", elapsed);

    // Verify all 200 duplicates were inserted into DB
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM snippets WHERE title LIKE '%(Kopie)'")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count.0, 200);

    // Check undo stack
    let stack = state.undo_stack.lock().unwrap();
    assert_eq!(stack.undo_history.len(), 1);
}

#[tokio::test]
async fn test_bulk_duplicate_partial_success_with_invalid_id() {
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

    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("valid-1")
    .bind("Valid Snippet 1")
    .bind("Some content")
    .bind("plain_text")
    .bind("inbox")
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .unwrap();

    let ids = vec!["valid-1".to_string(), "invalid-non-existent-id".to_string()];
    let res = duplicate_snippets_bulk(ids, None, state.clone()).await.unwrap();

    assert_eq!(res.succeeded.len(), 1);
    assert_eq!(res.succeeded[0].title, "Valid Snippet 1 (Kopie)");
    assert_eq!(res.failed.len(), 1);
    assert_eq!(res.failed[0].id, "invalid-non-existent-id");
    assert_eq!(res.failed[0].error["code"], "SNIPPET_NOT_FOUND");
}

#[tokio::test]
async fn test_bulk_duplicate_with_target_folder() {
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

    let now = chrono::Utc::now().timestamp_millis();
    // Create target folder
    let target_folder_id = "target-folder-123".to_string();
    sqlx::query("INSERT INTO folders (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&target_folder_id)
        .bind("Target Folder")
        .bind(now)
        .execute(&db)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("snip-orig")
    .bind("Original Prompt")
    .bind("Content")
    .bind("plain_text")
    .bind("inbox")
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .unwrap();

    let res = duplicate_snippets_bulk(
        vec!["snip-orig".to_string()],
        Some(target_folder_id.clone()),
        state.clone(),
    )
    .await
    .unwrap();

    assert_eq!(res.succeeded.len(), 1);
    let duplicated = &res.succeeded[0];
    assert_eq!(duplicated.folder_id, Some(target_folder_id));
    assert_eq!(duplicated.location_type, "folder");
}

#[tokio::test]
async fn test_bulk_duplicate_multiple_same_snippet_unique_titles() {
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

    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("same-snippet")
    .bind("Base Prompt")
    .bind("Content")
    .bind("plain_text")
    .bind("inbox")
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .unwrap();

    // Duplicate the same snippet 4 times in one bulk call
    let ids = vec![
        "same-snippet".to_string(),
        "same-snippet".to_string(),
        "same-snippet".to_string(),
        "same-snippet".to_string(),
    ];
    let res = duplicate_snippets_bulk(ids, None, state.clone()).await.unwrap();

    assert_eq!(res.succeeded.len(), 4);
    let titles: Vec<String> = res.succeeded.into_iter().map(|s| s.title).collect();
    assert_eq!(titles, vec![
        "Base Prompt (Kopie)",
        "Base Prompt (Kopie 2)",
        "Base Prompt (Kopie 3)",
        "Base Prompt (Kopie 4)",
    ]);
}
