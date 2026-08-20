use textforge::db::init_db;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_clipboard_operations() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    // Insert dummy entry directly
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at, size_bytes) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind("Hello TextForge Clipboard")
    .bind("hash123")
    .bind("plain_text")
    .bind(now)
    .bind(25)
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
