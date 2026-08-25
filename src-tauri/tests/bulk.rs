use textforge::db::init_db;
use tempfile::NamedTempFile;
use textforge::AppState;
use textforge::commands::undo::UndoStack;
use textforge::commands::bulk::{BulkOperationDto, BulkProgressPayload, execute_bulk_operation};
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_bulk_progress_payload_serialization() {
    let payload = BulkProgressPayload {
        completed: 5,
        total: 10,
        current_id: "snip-123".to_string(),
    };

    let json = serde_json::to_value(&payload).unwrap();
    assert_eq!(json["completed"], 5);
    assert_eq!(json["total"], 10);
    assert_eq!(json["currentId"], "snip-123");
}

#[tokio::test]
async fn test_bulk_transform_pipeline_error_partial_success() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
        clipboard_config: std::sync::RwLock::new(textforge::clipboard::ClipboardMonitorConfig::default()),
    });

    let state = app.state::<AppState>();
    let app_handle = app.app_handle().clone();

    let now = chrono::Utc::now().timestamp_millis();
    // Insert 3 snippets
    for i in 1..=3 {
        sqlx::query(
            "INSERT INTO snippets (id, title, content, content_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(format!("id-{}", i))
        .bind(format!("Title {}", i))
        .bind(format!("Content {}", i))
        .bind("text/plain")
        .bind(now)
        .bind(now)
        .execute(&db)
        .await
        .unwrap();
    }

    // Insert dummy script and pipeline with step
    sqlx::query("INSERT INTO scripts (id, name, description, script_type, category, js_code, regex_pattern, regex_replacement, regex_flags, color, parameters_json, tags_json, created_at, updated_at) VALUES ('script-1', 'Script 1', '', 'js', 'custom', 'invalid_js_code_syntax {{{{', NULL, NULL, 'g', '#000', '[]', '[]', 0, 0)")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO pipelines (id, name, description, created_at, updated_at) VALUES ('bad_pipeline', 'Bad Pipe', '', 0, 0)")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO pipeline_steps (id, pipeline_id, script_id, step_order, label, enabled, failure_policy) VALUES ('step-1', 'bad_pipeline', 'script-1', 0, 'Step 1', 1, 'abort')")
        .execute(&db)
        .await
        .unwrap();

    let op = BulkOperationDto::BulkTransform {
        snippet_ids: vec!["id-1".to_string(), "id-2".to_string(), "id-3".to_string()],
        pipeline_id: "bad_pipeline".to_string(),
        save_results: true,
    };

    let res = execute_bulk_operation(app_handle, op, state).await;
    assert!(res.is_ok());

    let res_dto = res.unwrap();
    assert_eq!(res_dto.succeeded.len(), 0);
    assert_eq!(res_dto.failed.len(), 3);
    assert_eq!(res_dto.failed[0].error["code"], "PIPELINE_ERROR");

    // Check DB content remains unchanged (original content)
    let (content1,): (String,) = sqlx::query_as("SELECT content FROM snippets WHERE id = 'id-1'")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(content1, "Content 1");
}

#[tokio::test]
async fn test_bulk_export_progress_and_transaction() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
        clipboard_config: std::sync::RwLock::new(textforge::clipboard::ClipboardMonitorConfig::default()),
    });

    let state = app.state::<AppState>();
    let app_handle = app.app_handle().clone();

    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, created_at, updated_at) VALUES ('exp-1', 'Export Title', 'Export Content', 'text/plain', ?, ?)"
    )
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .unwrap();

    let export_file = NamedTempFile::new().unwrap();
    let export_path = export_file.path().to_string_lossy().to_string();

    let op = BulkOperationDto::BulkExport {
        snippet_ids: vec!["exp-1".to_string(), "exp-2".to_string()],
        format: "json".to_string(),
        output_path: export_path.clone(),
    };

    let res = execute_bulk_operation(app_handle, op, state).await;
    assert!(res.is_ok());

    let res_dto = res.unwrap();
    assert_eq!(res_dto.succeeded, vec!["exp-1"]);
    assert_eq!(res_dto.failed.len(), 1);
    assert_eq!(res_dto.failed[0].id, "exp-2");
    assert_eq!(res_dto.failed[0].error["code"], "SNIPPET_NOT_FOUND");
}
