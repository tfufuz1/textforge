use textforge::db::init_db;
use textforge::commands::snippets::{list_snippets, SnippetFilterDto};
use textforge::AppState;
use tempfile::NamedTempFile;
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_list_snippets_pagination_and_preview() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.unwrap();

    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
    });
    let state = app.state::<AppState>();

    // Insert 120 snippets into the database
    let now = chrono::Utc::now().timestamp_millis();
    for i in 0..120 {
        let id = format!("snip-{}", i);
        let title = format!("Snippet Title {:03}", i);
        // Create a long content (300 chars) for snippet 0 to test preview truncation
        let content = if i == 0 {
            "A".repeat(300)
        } else {
            format!("Short content for snippet {}", i)
        };

        sqlx::query(
            "INSERT INTO snippets (id, title, content, content_type, location_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&title)
        .bind(&content)
        .bind("plain_text")
        .bind("inbox")
        .bind(now + i as i64)
        .bind(now + i as i64)
        .execute(&db)
        .await
        .unwrap();
    }

    // 1. First page (limit 50, offset 0)
    let filter1 = SnippetFilterDto {
        limit: Some(50),
        offset: Some(0),
        ..Default::default()
    };
    let res1 = list_snippets(Some(filter1), state.clone()).await.unwrap();
    assert_eq!(res1.items.len(), 50);
    assert_eq!(res1.total_count, 120);
    assert!(res1.has_more);

    // 2. Second page (limit 50, offset 50)
    let filter2 = SnippetFilterDto {
        limit: Some(50),
        offset: Some(50),
        ..Default::default()
    };
    let res2 = list_snippets(Some(filter2), state.clone()).await.unwrap();
    assert_eq!(res2.items.len(), 50);
    assert_eq!(res2.total_count, 120);
    assert!(res2.has_more);

    // 3. Third page (limit 50, offset 100) -> 20 remaining
    let filter3 = SnippetFilterDto {
        limit: Some(50),
        offset: Some(100),
        ..Default::default()
    };
    let res3 = list_snippets(Some(filter3), state.clone()).await.unwrap();
    assert_eq!(res3.items.len(), 20);
    assert_eq!(res3.total_count, 120);
    assert!(!res3.has_more);

    // 4. Default limit (None -> 50)
    let filter_def = SnippetFilterDto {
        limit: None,
        offset: None,
        ..Default::default()
    };
    let res_def = list_snippets(Some(filter_def), state.clone()).await.unwrap();
    assert_eq!(res_def.items.len(), 50);
    assert_eq!(res_def.total_count, 120);

    // 5. Capped limit (500 -> capped at 200 max)
    let filter_cap = SnippetFilterDto {
        limit: Some(500),
        offset: Some(0),
        ..Default::default()
    };
    let res_cap = list_snippets(Some(filter_cap), state.clone()).await.unwrap();
    assert_eq!(res_cap.items.len(), 120);
    assert_eq!(res_cap.total_count, 120);
    assert!(!res_cap.has_more);

    // 6. Test content preview truncation (snippet 0 had 300 chars)
    let item0 = res_cap.items.iter().find(|it| it.title == "Snippet Title 000").unwrap();
    assert_eq!(item0.preview.len(), 200);
}
