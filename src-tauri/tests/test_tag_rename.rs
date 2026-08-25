use tempfile::NamedTempFile;
use textforge::db::init_db;
use textforge::AppState;
use textforge::commands::tags::rename_tag;
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_rename_tag_no_data_loss_when_both_exist() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.expect("Failed to init db");

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
        clipboard_config: std::sync::RwLock::new(textforge::clipboard::ClipboardMonitorConfig::default()),
    });
    let state = app.state::<AppState>();

    // Setup: Snippet hat BEIDE Tags "foo" und "bar"
    let snippet_id = "snippet-both-tags-1";
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at)
         VALUES (?, 'Test Title', 'Test Content', 'plain_text', 'inbox', ?, ?)"
    )
    .bind(snippet_id).bind(now).bind(now)
    .execute(&db).await.unwrap();

    sqlx::query("INSERT INTO snippet_tags (snippet_id, tag) VALUES (?, 'foo'), (?, 'bar')")
        .bind(snippet_id).bind(snippet_id)
        .execute(&db).await.unwrap();

    // rename_tag("foo" -> "bar")
    let res = rename_tag("foo".to_string(), "bar".to_string(), state.clone())
        .await
        .expect("rename_tag failed");

    // Erwartet: affected_items = 0 (since snippet already had "bar", old "foo" tag was cleaned up)
    assert_eq!(res.affected_items, 0);

    // Erwartet: Snippet hat noch genau ["bar"] (kein Datenverlust, kein Duplikat)
    let tags: Vec<(String,)> = sqlx::query_as("SELECT tag FROM snippet_tags WHERE snippet_id = ?")
        .bind(snippet_id)
        .fetch_all(&db)
        .await
        .unwrap();

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].0, "bar");
}

#[tokio::test]
async fn test_rename_tag_updates_affected_count_correctly() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.expect("Failed to init db");

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
        clipboard_config: std::sync::RwLock::new(textforge::clipboard::ClipboardMonitorConfig::default()),
    });
    let state = app.state::<AppState>();

    // 3 Snippets mit "foo", 1 hat zusätzlich "bar"
    let now = chrono::Utc::now().timestamp_millis();
    for i in 1..=3 {
        let id = format!("snippet-{}", i);
        sqlx::query(
            "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at)
             VALUES (?, 'Title', 'Content', 'plain_text', 'inbox', ?, ?)"
        )
        .bind(&id).bind(now).bind(now)
        .execute(&db).await.unwrap();

        sqlx::query("INSERT INTO snippet_tags (snippet_id, tag) VALUES (?, 'foo')")
            .bind(&id)
            .execute(&db).await.unwrap();
    }

    // Snippet 3 hat zusätzlich "bar"
    sqlx::query("INSERT INTO snippet_tags (snippet_id, tag) VALUES ('snippet-3', 'bar')")
        .execute(&db).await.unwrap();

    // rename "foo" -> "bar"
    let res = rename_tag("foo".to_string(), "bar".to_string(), state.clone())
        .await
        .expect("rename_tag failed");

    // affected_items: 2 (die 2, bei denen rename möglich war; die 1 mit beiden wird bereinigt)
    assert_eq!(res.affected_items, 2);

    // Verify database state: all 3 snippets now have tag "bar" and none have "foo"
    let count_foo: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM snippet_tags WHERE tag = 'foo'")
        .fetch_one(&db).await.unwrap();
    assert_eq!(count_foo.0, 0);

    let count_bar: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM snippet_tags WHERE tag = 'bar'")
        .fetch_one(&db).await.unwrap();
    assert_eq!(count_bar.0, 3);
}
