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

#[derive(Deserialize)]
#[serde(tag = "_type", rename_all = "snake_case")]
enum SeparatorConfig {
    None,
    Newline { count: Option<usize> },
    Custom { text: Option<String> },
    NumberedList,
    MarkdownSection,
}

#[tauri::command]
pub async fn render_sequence(
    sequence_id: String,
    state: State<'_, AppState>,
) -> Result<SequenceRenderResultDto, String> {
    #[derive(sqlx::FromRow)]
    struct SeqRow {
        separator: String,
    }

    let seq = sqlx::query_as::<_, SeqRow>("SELECT separator FROM sequences WHERE id = ?")
        .bind(&sequence_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Sequence not found".to_string())?;

    #[derive(sqlx::FromRow)]
    struct ItemRow {
        ref_type: String,
        ref_id: Option<String>,
        literal_text: Option<String>,
        pipeline_id: Option<String>,
        prefix_override: Option<String>,
        suffix_override: Option<String>,
        enabled: i64,
    }

    let item_rows = sqlx::query_as::<_, ItemRow>(
        "SELECT ref_type, ref_id, literal_text, pipeline_id, prefix_override, suffix_override, enabled FROM sequence_items WHERE sequence_id = ? ORDER BY order_index ASC"
    )
    .bind(&sequence_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let enabled_items: Vec<_> = item_rows.into_iter().filter(|i| i.enabled != 0).collect();
    let item_count = enabled_items.len() as u32;

    let sep_config: Option<SeparatorConfig> = serde_json::from_str(&seq.separator).ok();

    let mut parts = Vec::new();

    for (idx, item) in enabled_items.into_iter().enumerate() {
        let mut content = match item.ref_type.as_str() {
            "snippet" => {
                if let Some(ref ref_id) = item.ref_id {
                    #[derive(sqlx::FromRow)]
                    struct SnippetRow { content: String }
                    let res = sqlx::query_as::<_, SnippetRow>("SELECT content FROM snippets WHERE id = ?")
                        .bind(ref_id)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten();
                    match res {
                        Some(row) => row.content,
                        None => format!("[Fehlender Verweis: snippet {}]", ref_id),
                    }
                } else {
                    "[Fehlender Verweis: snippet]".to_string()
                }
            }
            "clipboard" => {
                if let Some(ref ref_id) = item.ref_id {
                    #[derive(sqlx::FromRow)]
                    struct ClipRow { content: String }
                    let res = sqlx::query_as::<_, ClipRow>("SELECT content FROM clipboard_history WHERE id = ?")
                        .bind(ref_id)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten();
                    match res {
                        Some(row) => row.content,
                        None => format!("[Fehlender Verweis: clipboard {}]", ref_id),
                    }
                } else {
                    "[Fehlender Verweis: clipboard]".to_string()
                }
            }
            "script_output" => {
                if let Some(ref script_id) = item.ref_id {
                    let initial_input = item.literal_text.clone().unwrap_or_default();
                    let exec_req = crate::commands::transform::ExecuteScriptDto {
                        script_id: Some(script_id.clone()),
                        js_code: None,
                        regex_pattern: None,
                        regex_replacement: None,
                        regex_flags: None,
                        input: initial_input,
                        params_json: None,
                    };
                    match crate::commands::transform::execute_script(exec_req, state.clone()).await {
                        Ok(script_res) => script_res.output,
                        Err(_) => format!("[Fehlender Verweis: script_output {}]", script_id),
                    }
                } else {
                    item.literal_text.unwrap_or_default()
                }
            }
            "literal" => item.literal_text.unwrap_or_default(),
            other => {
                let ref_id_str = item.ref_id.as_deref().unwrap_or("");
                format!("[Fehlender Verweis: {} {}]", other, ref_id_str).trim().to_string()
            }
        };

        if let Some(ref pipeline_id) = item.pipeline_id {
            if !pipeline_id.is_empty() {
                if let Ok(pipe_res) = crate::commands::transform::run_pipeline(
                    pipeline_id.clone(),
                    content.clone(),
                    state.clone(),
                )
                .await {
                    content = pipe_res.final_output;
                }
            }
        }

        let is_override_none = item.prefix_override.is_none();
        let mut prefix = item.prefix_override.unwrap_or_default();
        if prefix.is_empty() && is_override_none {
            if let Some(ref sep) = sep_config {
                match sep {
                    SeparatorConfig::NumberedList => {
                        prefix = format!("{}. ", idx + 1);
                    }
                    SeparatorConfig::MarkdownSection => {
                        prefix = format!("## Abschnitt {}\n\n", idx + 1);
                    }
                    _ => {}
                }
            }
        }

        let suffix = item.suffix_override.unwrap_or_default();
        parts.push(format!("{}{}{}", prefix, content, suffix));
    }

    let joiner = match sep_config {
        Some(SeparatorConfig::None) => "".to_string(),
        Some(SeparatorConfig::Newline { count }) => "\n".repeat(count.unwrap_or(1)),
        Some(SeparatorConfig::Custom { text }) => text.unwrap_or_default(),
        Some(SeparatorConfig::NumberedList) | Some(SeparatorConfig::MarkdownSection) => "\n\n".to_string(),
        None => "\n".to_string(),
    };

    let final_output = parts.join(&joiner);

    Ok(SequenceRenderResultDto {
        final_output,
        item_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_render_sequence_mixed_and_missing_refs() {
        let db = crate::db::init_db(std::path::Path::new(":memory:")).await.unwrap();

        use tauri::Manager;
        let app = tauri::test::mock_app();
        app.manage(AppState {
            db: db.clone(),
            undo_stack: std::sync::Mutex::new(crate::commands::undo::UndoStack::new()),
        });
        let state = app.state::<AppState>();

        // Insert a snippet
        let snippet_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO snippets (id, title, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&snippet_id)
        .bind("Snippet Title")
        .bind("First Snippet Content")
        .bind(now)
        .bind(now)
        .execute(&db)
        .await
        .unwrap();

        // Create sequence with 3 items:
        // 1. snippet (existing)
        // 2. snippet (missing - deleted)
        // 3. literal text
        let seq_id = uuid::Uuid::new_v4().to_string();
        let missing_snippet_id = uuid::Uuid::new_v4().to_string();
        let sep_json = serde_json::to_string(&serde_json::json!({
            "_type": "custom",
            "text": "\n---\n"
        }))
        .unwrap();

        sqlx::query(
            "INSERT INTO sequences (id, name, separator, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&seq_id)
        .bind("Test Sequence")
        .bind(&sep_json)
        .bind(now)
        .bind(now)
        .execute(&db)
        .await
        .unwrap();

        // Item 1: Snippet
        sqlx::query(
            "INSERT INTO sequence_items (id, sequence_id, order_index, ref_type, ref_id, enabled) VALUES (?, ?, ?, ?, ?, 1)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&seq_id)
        .bind(0)
        .bind("snippet")
        .bind(&snippet_id)
        .execute(&db)
        .await
        .unwrap();

        // Item 2: Missing Snippet
        sqlx::query(
            "INSERT INTO sequence_items (id, sequence_id, order_index, ref_type, ref_id, enabled) VALUES (?, ?, ?, ?, ?, 1)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&seq_id)
        .bind(1)
        .bind("snippet")
        .bind(&missing_snippet_id)
        .execute(&db)
        .await
        .unwrap();

        // Item 3: Literal text
        sqlx::query(
            "INSERT INTO sequence_items (id, sequence_id, order_index, ref_type, literal_text, enabled) VALUES (?, ?, ?, ?, ?, 1)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&seq_id)
        .bind(2)
        .bind("literal")
        .bind("Literal Baustein Output")
        .execute(&db)
        .await
        .unwrap();

        let render_res = render_sequence(seq_id, state)
            .await
            .expect("render_sequence failed");

        assert_eq!(render_res.item_count, 3);
        let expected_missing_placeholder = format!("[Fehlender Verweis: snippet {}]", missing_snippet_id);
        let expected_output = format!(
            "First Snippet Content\n---\n{}\n---\nLiteral Baustein Output",
            expected_missing_placeholder
        );
        assert_eq!(render_res.final_output, expected_output);
    }
}
