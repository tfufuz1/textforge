use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardIgnoreRuleDto {
    pub id: String,
    pub enabled: bool,
    pub match_type: String, // 'source_app' | 'content_regex' | 'content_type'
    pub pattern: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIgnoreRuleDto {
    pub match_type: String,
    pub pattern: String,
}

#[tauri::command]
pub async fn list_ignore_rules(
    state: State<'_, AppState>,
) -> Result<Vec<ClipboardIgnoreRuleDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        enabled: i64,
        match_type: String,
        pattern: String,
        created_at: i64,
    }

    let rows = sqlx::query_as::<_, Row>("SELECT id, enabled, match_type, pattern, created_at FROM clipboard_ignore_rules ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| ClipboardIgnoreRuleDto {
        id: r.id,
        enabled: r.enabled != 0,
        match_type: r.match_type,
        pattern: r.pattern,
        created_at: r.created_at,
    }).collect())
}

#[tauri::command]
pub async fn create_ignore_rule(
    draft: CreateIgnoreRuleDto,
    state: State<'_, AppState>,
) -> Result<ClipboardIgnoreRuleDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("INSERT INTO clipboard_ignore_rules (id, enabled, match_type, pattern, created_at) VALUES (?, 1, ?, ?, ?)")
        .bind(&id).bind(&draft.match_type).bind(&draft.pattern).bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ClipboardIgnoreRuleDto {
        id,
        enabled: true,
        match_type: draft.match_type,
        pattern: draft.pattern,
        created_at: now,
    })
}

#[tauri::command]
pub async fn delete_ignore_rule(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM clipboard_ignore_rules WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
