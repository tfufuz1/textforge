use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedItemListItemDto {
    pub item_kind: String, // 'snippet' | 'clipboard' | 'script' | 'pipeline'
    pub id: String,
    pub title: String,
    pub preview: String,
    pub highlighted_preview: String,
    pub tags: Vec<String>,
    pub content_type: Option<String>,
    pub updated_at: i64,
    pub match_score: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedItemFilterDto {
    pub search_query: Option<String>,
    pub item_kinds: Vec<String>,
    pub tags: Vec<String>,
    pub tags_mode: Option<String>,
}

#[tauri::command]
pub async fn search_all_items(
    filter: UnifiedItemFilterDto,
    state: State<'_, AppState>,
) -> Result<Vec<UnifiedItemListItemDto>, String> {
    let query_str = filter.search_query.unwrap_or_default().trim().to_string();
    if query_str.is_empty() {
        return Ok(Vec::new());
    }

    let fts_query = crate::commands::clipboard::format_fts5_query(&query_str);

    let mut results = Vec::new();

    // 1. Search Snippets
    if filter.item_kinds.is_empty() || filter.item_kinds.contains(&"snippet".to_string()) {
        #[derive(sqlx::FromRow)]
        struct SnipRow {
            id: String,
            title: String,
            content: String,
            content_type: String,
            updated_at: i64,
        }

        let snips = sqlx::query_as::<_, SnipRow>(
            "SELECT s.id, s.title, s.content, s.content_type, s.updated_at
             FROM snippets s
             JOIN snippets_fts f ON s.rowid = f.rowid
             WHERE snippets_fts MATCH ?
             LIMIT 20"
        )
        .bind(&fts_query)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for s in snips {
            let highlighted = s.content.replace(&query_str, &format!("<mark>{}</mark>", query_str));
            results.push(UnifiedItemListItemDto {
                item_kind: "snippet".to_string(),
                id: s.id,
                title: s.title,
                preview: s.content.chars().take(150).collect(),
                highlighted_preview: highlighted.chars().take(200).collect(),
                tags: Vec::new(),
                content_type: Some(s.content_type),
                updated_at: s.updated_at,
                match_score: Some(1.0),
            });
        }
    }

    // 2. Search Clipboard History
    if filter.item_kinds.is_empty() || filter.item_kinds.contains(&"clipboard".to_string()) {
        #[derive(sqlx::FromRow)]
        struct ClipRow {
            id: String,
            content: String,
            content_type: String,
            updated_at: i64,
        }

        let clips = sqlx::query_as::<_, ClipRow>(
            "SELECT c.id, c.content, c.content_type, c.updated_at
             FROM clipboard_history c
             JOIN clipboard_fts f ON c.rowid = f.rowid
             WHERE clipboard_fts MATCH ?
             LIMIT 20"
        )
        .bind(&fts_query)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for c in clips {
            let highlighted = c.content.replace(&query_str, &format!("<mark>{}</mark>", query_str));
            results.push(UnifiedItemListItemDto {
                item_kind: "clipboard".to_string(),
                id: c.id,
                title: c.content.chars().take(30).collect(),
                preview: c.content.chars().take(150).collect(),
                highlighted_preview: highlighted.chars().take(200).collect(),
                tags: Vec::new(),
                content_type: Some(c.content_type),
                updated_at: c.updated_at,
                match_score: Some(0.9),
            });
        }
    }

    Ok(results)
}
