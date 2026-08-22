use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SequenceItemDto {
    pub id: String,
    pub order_index: u32,
    pub ref_type: String, // 'snippet' | 'clipboard' | 'script_output' | 'literal'
    pub ref_id: Option<String>,
    pub literal_text: Option<String>,
    pub pipeline_id: Option<String>,
    pub prefix_override: Option<String>,
    pub suffix_override: Option<String>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SequenceDto {
    pub id: String,
    pub name: String,
    pub separator: String, // JSON
    pub favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub items: Vec<SequenceItemDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSequenceDto {
    pub name: String,
    pub separator: Option<String>,
    pub items: Vec<SequenceItemDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceRenderResultDto {
    pub final_output: String,
    pub item_count: u32,
}

#[tauri::command]
pub async fn list_sequences(
    state: State<'_, AppState>,
) -> Result<Vec<SequenceDto>, String> {
    #[derive(sqlx::FromRow)]
    struct SeqRow {
        id: String,
        name: String,
        separator: String,
        favorite: i64,
        created_at: i64,
        updated_at: i64,
    }

    let seqs = sqlx::query_as::<_, SeqRow>("SELECT id, name, separator, favorite, created_at, updated_at FROM sequences ORDER BY updated_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for s in seqs {
        #[derive(sqlx::FromRow)]
        struct ItemRow {
            id: String,
            order_index: i64,
            ref_type: String,
            ref_id: Option<String>,
            literal_text: Option<String>,
            pipeline_id: Option<String>,
            prefix_override: Option<String>,
            suffix_override: Option<String>,
            enabled: i64,
        }

        let items_rows = sqlx::query_as::<_, ItemRow>("SELECT id, order_index, ref_type, ref_id, literal_text, pipeline_id, prefix_override, suffix_override, enabled FROM sequence_items WHERE sequence_id = ? ORDER BY order_index ASC")
            .bind(&s.id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

        let items = items_rows.into_iter().map(|i| SequenceItemDto {
            id: i.id,
            order_index: i.order_index as u32,
            ref_type: i.ref_type,
            ref_id: i.ref_id,
            literal_text: i.literal_text,
            pipeline_id: i.pipeline_id,
            prefix_override: i.prefix_override,
            suffix_override: i.suffix_override,
            enabled: i.enabled != 0,
        }).collect();

        result.push(SequenceDto {
            id: s.id,
            name: s.name,
            separator: s.separator,
            favorite: s.favorite != 0,
            created_at: s.created_at,
            updated_at: s.updated_at,
            items,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn create_sequence(
    draft: CreateSequenceDto,
    state: State<'_, AppState>,
) -> Result<SequenceDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let sep = draft.separator.unwrap_or_else(|| "{\"_type\":\"newline\",\"count\":1}".to_string());

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO sequences (id, name, separator, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(&draft.name).bind(&sep).bind(now).bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (idx, item) in draft.items.iter().enumerate() {
        let item_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO sequence_items (id, sequence_id, order_index, ref_type, ref_id, literal_text, pipeline_id, prefix_override, suffix_override, enabled)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&item_id).bind(&id).bind(idx as i64).bind(&item.ref_type)
        .bind(&item.ref_id).bind(&item.literal_text).bind(&item.pipeline_id)
        .bind(&item.prefix_override).bind(&item.suffix_override)
        .bind(if item.enabled { 1 } else { 0 })
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(SequenceDto {
        id,
        name: draft.name,
        separator: sep,
        favorite: false,
        created_at: now,
        updated_at: now,
        items: draft.items,
    })
}

#[tauri::command]
pub async fn delete_sequence(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM sequences WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn quick_combine(
    texts: Vec<String>,
    separator: Option<String>,
) -> Result<String, String> {
    let sep = separator.unwrap_or_else(|| "\n\n".to_string());
    Ok(texts.join(&sep))
}
