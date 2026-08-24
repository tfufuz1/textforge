use textforge::db::init_db;
use tempfile::NamedTempFile;
use textforge::AppState;
use textforge::commands::collections::{
    list_collection_tabs, create_collection_tab, CreateCollectionTabDto,
};
use textforge::commands::automation::{
    list_automation_rules, create_automation_rule, CreateAutomationRuleDto,
};
use textforge::commands::tags::{
    suggest_tags, rename_tag, set_tag_color,
};
use std::sync::Mutex;
use tauri::Manager;

#[tokio::test]
async fn test_v3_tables_exist_and_commands() {
    let tmp = NamedTempFile::new().unwrap();
    let db = init_db(tmp.path()).await.expect("Failed to init db");

    let tables_to_check = vec![
        "collection_tabs",
        "collection_tab_members",
        "automation_rules",
        "script_tags",
        "pipeline_tags",
        "tag_colors",
    ];

    for table in &tables_to_check {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?"
        )
        .bind(table)
        .fetch_one(&db)
        .await
        .unwrap_or_else(|e| panic!("Error checking table {}: {}", table, e));

        assert_eq!(count, 1, "Table '{}' does not exist in database after init_db()", table);
    }

    // Verify folders updated_at column exists
    let pragma_cols: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(folders)")
            .fetch_all(&db)
            .await
            .expect("Failed to get table_info for folders");

    let has_updated_at = pragma_cols.iter().any(|c| c.1 == "updated_at");
    assert!(has_updated_at, "Column 'updated_at' missing from 'folders' table");

    // Set up Tauri Mock State to test commands
    let app = tauri::test::mock_app();
    app.manage(AppState {
        db: db.clone(),
        undo_stack: Mutex::new(textforge::commands::undo::UndoStack::new()),
        regex_cache: Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
    });
    let state = app.state::<AppState>();

    // 1. Test collection_tabs commands
    let tabs_before = list_collection_tabs(state.clone())
        .await
        .expect("list_collection_tabs failed");
    let initial_len = tabs_before.len();

    let created_tab = create_collection_tab(
        CreateCollectionTabDto {
            name: "Test Tab".to_string(),
            icon: Some("star".to_string()),
            color: Some("#ff0000".to_string()),
            kind: Some("manual".to_string()),
            kind_config: None,
        },
        state.clone(),
    )
    .await
    .expect("create_collection_tab failed");

    assert_eq!(created_tab.name, "Test Tab");

    let tabs_after = list_collection_tabs(state.clone())
        .await
        .expect("list_collection_tabs after create failed");
    assert_eq!(tabs_after.len(), initial_len + 1);

    // 2. Test automation_rules commands
    // First insert a dummy script to satisfy foreign key constraint on automation_rules
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO scripts (id, name, description, script_type, category, js_code, created_at, updated_at)
         VALUES ('script-test-1', 'Test Script', 'Desc', 'js', 'custom', 'return input;', ?, ?)"
    )
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .expect("Inserting dummy script failed");

    let rules_before = list_automation_rules(state.clone())
        .await
        .expect("list_automation_rules failed");

    let created_rule = create_automation_rule(
        CreateAutomationRuleDto {
            name: "Auto Rule 1".to_string(),
            trigger: r#"{"type":"on_copy"}"#.to_string(),
            condition: None,
            script_id: "script-test-1".to_string(),
        },
        state.clone(),
    )
    .await
    .expect("create_automation_rule failed");

    assert_eq!(created_rule.name, "Auto Rule 1");

    let rules_after = list_automation_rules(state.clone())
        .await
        .expect("list_automation_rules after create failed");
    assert_eq!(rules_after.len(), rules_before.len() + 1);

    // 3. Test tags commands
    // Insert dummy script_tag and pipeline_tag
    sqlx::query("INSERT INTO pipelines (id, name, description, created_at, updated_at) VALUES ('pipe-test-1', 'Test Pipe', '', ?, ?)")
        .bind(now).bind(now)
        .execute(&db).await.expect("Inserting dummy pipeline failed");

    sqlx::query("INSERT INTO script_tags (script_id, tag) VALUES ('script-test-1', 'rust')")
        .execute(&db).await.expect("Inserting script_tag failed");

    sqlx::query("INSERT INTO pipeline_tags (pipeline_id, tag) VALUES ('pipe-test-1', 'rust')")
        .execute(&db).await.expect("Inserting pipeline_tag failed");

    let suggestions = suggest_tags("ru".to_string(), 10, state.clone())
        .await
        .expect("suggest_tags failed");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].name, "rust");

    // set_tag_color
    set_tag_color("rust".to_string(), Some("#00ff00".to_string()), state.clone())
        .await
        .expect("set_tag_color failed");

    let suggestions_colored = suggest_tags("ru".to_string(), 10, state.clone())
        .await
        .expect("suggest_tags after color failed");
    assert_eq!(suggestions_colored[0].color, Some("#00ff00".to_string()));

    // rename_tag
    let rename_res = rename_tag("rust".to_string(), "ferris".to_string(), state.clone())
        .await
        .expect("rename_tag failed");
    assert_eq!(rename_res.affected_items, 2);

    let old_suggestions = suggest_tags("rust".to_string(), 10, state.clone())
        .await
        .expect("suggest_tags old failed");
    assert!(old_suggestions.is_empty());

    let new_suggestions = suggest_tags("ferris".to_string(), 10, state.clone())
        .await
        .expect("suggest_tags new failed");
    assert_eq!(new_suggestions.len(), 1);
    assert_eq!(new_suggestions[0].name, "ferris");
}
