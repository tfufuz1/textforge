use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRuleDto {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: String, // JSON: AutomationTrigger
    pub condition: Option<String>,
    pub script_id: String,
    pub sort_order: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAutomationRuleDto {
    pub name: String,
    pub trigger: String,
    pub condition: Option<String>,
    pub script_id: String,
}

#[tauri::command]
pub async fn list_automation_rules(
    state: State<'_, AppState>,
) -> Result<Vec<AutomationRuleDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        enabled: i64,
        trigger: String,
        condition: Option<String>,
        script_id: String,
        sort_order: i64,
        created_at: i64,
        updated_at: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT id, name, enabled, trigger, condition, script_id, sort_order, created_at, updated_at
         FROM automation_rules
         ORDER BY sort_order ASC, created_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| AutomationRuleDto {
        id: r.id,
        name: r.name,
        enabled: r.enabled != 0,
        trigger: r.trigger,
        condition: r.condition,
        script_id: r.script_id,
        sort_order: r.sort_order as u32,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect())
}

#[tauri::command]
pub async fn create_automation_rule(
    draft: CreateAutomationRuleDto,
    state: State<'_, AppState>,
) -> Result<AutomationRuleDto, String> {
    // INVARIANT-AR3: Maximal 50 aktive Regeln
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM automation_rules WHERE enabled = 1")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    if count.0 >= 50 {
        return Err("Maximal 50 aktive Automatisierungsregeln erlaubt (INVARIANT-AR3)".to_string());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO automation_rules (id, name, enabled, trigger, condition, script_id, sort_order, created_at, updated_at)
         VALUES (?, ?, 1, ?, ?, ?, 0, ?, ?)"
    )
    .bind(&id).bind(&draft.name).bind(&draft.trigger).bind(&draft.condition)
    .bind(&draft.script_id).bind(now).bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(AutomationRuleDto {
        id,
        name: draft.name,
        enabled: true,
        trigger: draft.trigger,
        condition: draft.condition,
        script_id: draft.script_id,
        sort_order: 0,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn delete_automation_rule(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM automation_rules WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
