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
