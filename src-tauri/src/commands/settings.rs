use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSessionDto {
    #[serde(default = "default_active_view")]
    pub active_view: String,
    pub selected_snippet_id: Option<String>,
    pub selected_script_id: Option<String>,
    pub selected_pipeline_id: Option<String>,
    pub selected_clipboard_id: Option<String>,
    pub search_query: Option<String>,
    #[serde(default = "default_true")]
    pub sidebar_open: bool,
    pub sidebar_width: Option<u32>,
    pub preview_mode: Option<String>,
    pub filter_state: Option<serde_json::Value>,
    pub saved_at: Option<i64>,
}

fn default_active_view() -> String {
    "clipboard".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatsDto {
    pub total_snippets: u32,
    pub total_clipboard_entries: u32,
    pub total_scripts: u32,
    pub total_pipelines: u32,
    pub db_size_bytes: u64,
}

#[tauri::command]
pub async fn get_all_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        key: String,
        value: String,
    }

    let rows = sqlx::query_as::<_, Row>("SELECT key, value FROM settings")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for r in rows {
        map.insert(r.key, r.value);
    }
    Ok(map)
}

#[tauri::command]
pub async fn set_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at")
        .bind(&key).bind(&value).bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(mut config) = state.clipboard_config.write() {
        match key.as_str() {
            "clipboard.max_entries" => {
                if let Ok(v) = value.parse::<u32>() { config.max_entries = v; }
            }
            "clipboard.min_length" => {
                if let Ok(v) = value.parse::<usize>() { config.min_content_length = v; }
            }
            "clipboard.dedup_window_ms" => {
                if let Ok(v) = value.parse::<u64>() { config.dedup_window_ms = v; }
            }
            _ => {}
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_workspace_session(
    state: State<'_, AppState>,
) -> Result<Option<WorkspaceSessionDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        value: String,
    }

    let row = sqlx::query_as::<_, Row>("SELECT value FROM settings WHERE key = 'session.data'")
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(r) = row {
        let session: WorkspaceSessionDto = serde_json::from_str(&r.value).map_err(|e| e.to_string())?;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn save_workspace_session(
    session: WorkspaceSessionDto,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let json_val = serde_json::to_string(&session).map_err(|e| e.to_string())?;
    set_setting("session.data".to_string(), json_val, state).await
}

#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<DatabaseStatsDto, String> {
    let (snippets_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM snippets")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let (clipboard_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_history")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let (scripts_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scripts")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let (pipelines_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pipelines")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let (page_count,): (i64,) = sqlx::query_as("PRAGMA page_count")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let (page_size,): (i64,) = sqlx::query_as("PRAGMA page_size")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let db_size_bytes = (page_count * page_size) as u64;

    Ok(DatabaseStatsDto {
        total_snippets: snippets_count as u32,
        total_clipboard_entries: clipboard_count as u32,
        total_scripts: scripts_count as u32,
        total_pipelines: pipelines_count as u32,
        db_size_bytes,
    })
}
