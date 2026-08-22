use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionTabDto {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: u32,
    pub kind: String, // 'manual' | 'smart' | 'clipboard_capture'
    pub kind_config: Option<String>,
    pub is_pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub item_count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionTabDto {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub kind: Option<String>,
    pub kind_config: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRefDto {
    pub item_kind: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn list_collection_tabs(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionTabDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        icon: Option<String>,
        color: Option<String>,
        sort_order: i64,
        kind: String,
        kind_config: Option<String>,
        is_pinned: i64,
        created_at: i64,
        updated_at: i64,
        item_count: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT t.id, t.name, t.icon, t.color, t.sort_order, t.kind, t.kind_config, t.is_pinned, t.created_at, t.updated_at,
                COUNT(m.item_id) as item_count
         FROM collection_tabs t
         LEFT JOIN collection_tab_members m ON t.id = m.tab_id
         GROUP BY t.id
         ORDER BY t.sort_order ASC, t.created_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| CollectionTabDto {
        id: r.id,
        name: r.name,
        icon: r.icon,
        color: r.color,
        sort_order: r.sort_order as u32,
        kind: r.kind,
        kind_config: r.kind_config,
        is_pinned: r.is_pinned != 0,
        created_at: r.created_at,
        updated_at: r.updated_at,
        item_count: r.item_count as u32,
    }).collect())
}

#[tauri::command]
pub async fn create_collection_tab(
    draft: CreateCollectionTabDto,
    state: State<'_, AppState>,
) -> Result<CollectionTabDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let kind = draft.kind.unwrap_or_else(|| "manual".to_string());

    sqlx::query(
        "INSERT INTO collection_tabs (id, name, icon, color, kind, kind_config, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&draft.name).bind(&draft.icon).bind(&draft.color)
    .bind(&kind).bind(&draft.kind_config).bind(now).bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CollectionTabDto {
        id,
        name: draft.name,
        icon: draft.icon,
        color: draft.color,
        sort_order: 0,
        kind,
        kind_config: draft.kind_config,
        is_pinned: false,
        created_at: now,
        updated_at: now,
        item_count: 0,
    })
}

#[tauri::command]
pub async fn delete_collection_tab(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if id == "default" {
        return Err("Default tab cannot be deleted".to_string());
    }
    sqlx::query("DELETE FROM collection_tabs WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_item_to_tab(
    tab_id: String,
    item_ref: ItemRefDto,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT OR IGNORE INTO collection_tab_members (tab_id, item_kind, item_id, added_at)
         VALUES (?, ?, ?, ?)"
    )
    .bind(&tab_id).bind(&item_ref.item_kind).bind(&item_ref.item_id).bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn remove_item_from_tab(
    tab_id: String,
    item_ref: ItemRefDto,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM collection_tab_members WHERE tab_id = ? AND item_kind = ? AND item_id = ?"
    )
    .bind(&tab_id).bind(&item_ref.item_kind).bind(&item_ref.item_id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
