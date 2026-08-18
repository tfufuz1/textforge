use textforge::db::init_db;
use textforge::commands::clipboard::*;
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
}
