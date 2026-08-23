use textforge::db::init_db;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_fresh_database_migration() {
    let tmp = NamedTempFile::new().unwrap();
    let result = init_db(tmp.path()).await;
    assert!(result.is_ok(), "Database initialization on fresh file failed: {:?}", result.err());
}

#[tokio::test]
async fn test_clipboard_operations() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    // Insert dummy entry directly
    // Note: size_bytes is a GENERATED ALWAYS AS ... VIRTUAL column in 003_clipboard.sql.
    // Excluded from INSERT column list since SQLite rejects INSERTs into generated columns.
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind("Hello TextForge Clipboard")
    .bind("hash123")
    .bind("plain_text")
    .bind(now)
    .execute(&db)
    .await
    .unwrap();

    // Verify row count
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_history")
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(count, 1);

    // Test Pinning
    sqlx::query("UPDATE clipboard_history SET is_pinned = 1 WHERE id = ?")
        .bind(&id)
        .execute(&db)
        .await
        .unwrap();

    let (pinned,): (i64,) = sqlx::query_as("SELECT is_pinned FROM clipboard_history WHERE id = ?")
        .bind(&id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(pinned, 1);

    // Test FTS5 Trigger on Insert
    let (fts_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_fts WHERE content MATCH 'TextForge'")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(fts_count, 1);

    // Test Deletion
    sqlx::query("DELETE FROM clipboard_history WHERE id = ?")
        .bind(&id)
        .execute(&db)
        .await
        .unwrap();

    let (count_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_history")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count_after, 0);
}

#[tokio::test]
async fn test_clipboard_lru_trim_unpinned() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    // Insert pinned entry (oldest)
    let pinned_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at, is_pinned) VALUES (?, ?, ?, ?, ?, 1)"
    )
    .bind(&pinned_id)
    .bind("Pinned Item")
    .bind("hash_pinned")
    .bind("plain_text")
    .bind(1000)
    .execute(&db)
    .await
    .unwrap();

    // Insert 5 unpinned entries
    for i in 1..=5 {
        let unpinned_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at, is_pinned) VALUES (?, ?, ?, ?, ?, 0)"
        )
        .bind(&unpinned_id)
        .bind(format!("Unpinned Item {}", i))
        .bind(format!("hash_unpinned_{}", i))
        .bind("plain_text")
        .bind(1000 + i * 100)
        .execute(&db)
        .await
        .unwrap();
    }

    // Total 6 items. Simulate max_entries = 3. We need to trim (6 - 3 = 3) oldest unpinned items.
    let max_entries = 3;
    let (total_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_history")
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(total_count, 6);

    let to_delete = total_count - max_entries;
    sqlx::query("DELETE FROM clipboard_history WHERE id IN (SELECT id FROM clipboard_history WHERE is_pinned = 0 ORDER BY captured_at ASC LIMIT ?)")
        .bind(to_delete)
        .execute(&db)
        .await
        .unwrap();

    // Verify pinned entry is preserved
    let (pinned_exists,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_history WHERE id = ?")
        .bind(&pinned_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(pinned_exists, 1);

    // Remaining count should be 3 (1 pinned + 2 newest unpinned)
    let (remaining_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_history")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(remaining_count, 3);
}

#[tokio::test]
async fn test_clipboard_fts5_search_filter() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let id1 = uuid::Uuid::new_v4().to_string();
    let id2 = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id1)
    .bind("Special keyword alpha_test_snippet")
    .bind("hash_search_1")
    .bind("plain_text")
    .bind(2000)
    .execute(&db)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id2)
    .bind("Another text without the target word")
    .bind("hash_search_2")
    .bind("plain_text")
    .bind(2100)
    .execute(&db)
    .await
    .unwrap();

    // Query FTS5 for "alpha_test_snippet"
    let formatted = textforge::commands::clipboard::format_fts5_query("alpha_test_snippet");
    let (matches,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM clipboard_history WHERE rowid IN (SELECT rowid FROM clipboard_fts WHERE content MATCH ?)"
    )
    .bind(&formatted)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(matches, 1);
}

#[tokio::test]
async fn test_compose_clipboard_entries_to_snippet() {
    use std::sync::Mutex;
    use tauri::Manager;
    use textforge::commands::clipboard::{compose_clipboard_entries_to_snippet, SnippetLocationDto};
    use textforge::AppState;

    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
    });
    let state = app.state::<AppState>();

    let id1 = "clip-1".to_string();
    let id2 = "clip-2".to_string();
    let id3 = "clip-3".to_string();

    sqlx::query("INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id1).bind("Du bist ein erfahrener Softwarearchitekt.").bind("hash_1").bind("plain_text").bind(100)
        .execute(&db).await.unwrap();

    sqlx::query("INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id2).bind("Erstelle eine saubere API-Spezifikation für {{service_name}}.").bind("hash_2").bind("plain_text").bind(200)
        .execute(&db).await.unwrap();

    sqlx::query("INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id3).bind("Antworte ausschließlich im Markdown-Format.").bind("hash_3").bind("plain_text").bind(300)
        .execute(&db).await.unwrap();

    // Pass in custom order [id3, id1, id2] to verify order preservation
    let entry_ids = vec![id3.clone(), id1.clone(), id2.clone()];
    let location = SnippetLocationDto { _type: "inbox".to_string(), folder_id: None };

    let snippet_id = compose_clipboard_entries_to_snippet(
        entry_ids,
        Some("\n\n---\n\n".to_string()),
        None,
        location,
        state.clone(),
    )
    .await
    .unwrap();

    // Verify created snippet in DB
    #[derive(sqlx::FromRow)]
    struct SnipRow {
        title: String,
        content: String,
        is_template: i64,
    }

    let snip: SnipRow = sqlx::query_as("SELECT title, content, is_template FROM snippets WHERE id = ?")
        .bind(&snippet_id)
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(snip.title, "Antworte ausschließlich im Markdown-Format.");
    assert_eq!(
        snip.content,
        "Antworte ausschließlich im Markdown-Format.\n\n---\n\nDu bist ein erfahrener Softwarearchitekt.\n\n---\n\nErstelle eine saubere API-Spezifikation für {{service_name}}."
    );
    assert_eq!(snip.is_template, 1);

    // Verify promoted_to_snippet_id is set for all 3 clipboard entries
    let promoted_ids: Vec<(Option<String>,)> = sqlx::query_as("SELECT promoted_to_snippet_id FROM clipboard_history WHERE id IN ('clip-1', 'clip-2', 'clip-3')")
        .fetch_all(&db)
        .await
        .unwrap();

    assert_eq!(promoted_ids.len(), 3);
    for (p_id,) in promoted_ids {
        assert_eq!(p_id, Some(snippet_id.clone()));
    }

    // Verify undo stack
    let stack = state.undo_stack.lock().unwrap();
    assert_eq!(stack.undo_history.len(), 1);
    assert!(stack.undo_history[0].description.contains("zusammengestellt"));
}

#[tokio::test]
async fn test_promote_clipboard_entries_bulk() {
    use std::sync::Mutex;
    use tauri::Manager;
    use textforge::commands::clipboard::{promote_clipboard_entries_bulk, SnippetLocationDto};
    use textforge::AppState;

    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
    });
    let state = app.state::<AppState>();

    let id1 = "bulk-clip-1".to_string();
    let id2 = "bulk-clip-2".to_string();
    let id3 = "bulk-clip-3".to_string();

    sqlx::query("INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id1).bind("Snippet Item Alpha").bind("hash_b1").bind("plain_text").bind(100)
        .execute(&db).await.unwrap();

    sqlx::query("INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id2).bind("Snippet Item Beta").bind("hash_b2").bind("plain_text").bind(200)
        .execute(&db).await.unwrap();

    sqlx::query("INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id3).bind("Snippet Item Gamma").bind("hash_b3").bind("plain_text").bind(300)
        .execute(&db).await.unwrap();

    let entry_ids = vec![id1.clone(), id2.clone(), id3.clone()];
    let location = SnippetLocationDto { _type: "inbox".to_string(), folder_id: None };

    let created_ids = promote_clipboard_entries_bulk(entry_ids, location, state.clone()).await.unwrap();

    assert_eq!(created_ids.len(), 3);

    // Verify 3 distinct snippets created in DB
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM snippets WHERE id IN (?, ?, ?)")
        .bind(&created_ids[0]).bind(&created_ids[1]).bind(&created_ids[2])
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(count, 3);

    // Verify promoted_to_snippet_id
    let (p1,): (Option<String>,) = sqlx::query_as("SELECT promoted_to_snippet_id FROM clipboard_history WHERE id = ?")
        .bind(&id1).fetch_one(&db).await.unwrap();
    assert_eq!(p1, Some(created_ids[0].clone()));

    let (p2,): (Option<String>,) = sqlx::query_as("SELECT promoted_to_snippet_id FROM clipboard_history WHERE id = ?")
        .bind(&id2).fetch_one(&db).await.unwrap();
    assert_eq!(p2, Some(created_ids[1].clone()));

    // Verify single bulk undo entry
    let stack = state.undo_stack.lock().unwrap();
    assert_eq!(stack.undo_history.len(), 1);
    assert!(stack.undo_history[0].description.contains("3 Snippets aus Zwischenablage erstellt"));
}
