use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

pub fn format_fts5_query(search: &str) -> String {
    let trimmed = search.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let escaped = trimmed.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardFilterDto {
    pub search_query: Option<String>,
    pub content_types: Vec<String>,
    pub source_apps: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResultDto<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardEntryListItemDto {
    pub id: String,
    pub preview: String,
    pub content_type: String,
    pub source_app: Option<String>,
    pub captured_at: i64,
    pub size_bytes: u32,
    pub is_pinned: bool,
    pub match_score: Option<f32>,
    pub promoted_to_snippet_id: Option<String>,
}

#[tauri::command]
pub async fn list_clipboard_history(
    filter: Option<ClipboardFilterDto>,
    page: Option<u32>,
    page_size: Option<u32>,
    state: State<'_, AppState>,
) -> Result<PagedResultDto<ClipboardEntryListItemDto>, String> {
    let filter = filter.unwrap_or_default();
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(50);

    let offset = page * page_size;

    // Use sqlx QueryBuilder for dynamic query
    let mut query = sqlx::QueryBuilder::new("SELECT id, content, content_type, source_app, captured_at, size_bytes, is_pinned, promoted_to_snippet_id FROM clipboard_history WHERE 1=1");

    if let Some(search) = &filter.search_query {
        let formatted = format_fts5_query(search);
        if !formatted.is_empty() {
            query.push(" AND rowid IN (SELECT rowid FROM clipboard_fts WHERE content MATCH ");
            query.push_bind(formatted);
            query.push(")");
        }
    }

    if !filter.content_types.is_empty() {
        query.push(" AND content_type IN (");
        let mut separated = query.separated(", ");
        for ct in &filter.content_types {
            separated.push_bind(ct.clone());
        }
        separated.push_unseparated(")");
    }

    if !filter.source_apps.is_empty() {
        query.push(" AND source_app IN (");
        let mut separated = query.separated(", ");
        for sa in &filter.source_apps {
            separated.push_bind(sa.clone());
        }
        separated.push_unseparated(")");
    }

    query.push(" ORDER BY captured_at DESC LIMIT ");
    query.push_bind(page_size);
    query.push(" OFFSET ");
    query.push_bind(offset);

    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        content: String,
        content_type: String,
        source_app: Option<String>,
        captured_at: i64,
        size_bytes: i64,
        is_pinned: i64,
        promoted_to_snippet_id: Option<String>,
    }

    let entries: Vec<Row> = query
        .build_query_as()
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let mut count_query = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM clipboard_history WHERE 1=1");
    if let Some(search) = &filter.search_query {
        let formatted = format_fts5_query(search);
        if !formatted.is_empty() {
            count_query.push(" AND rowid IN (SELECT rowid FROM clipboard_fts WHERE content MATCH ");
            count_query.push_bind(formatted);
            count_query.push(")");
        }
    }
    if !filter.content_types.is_empty() {
        count_query.push(" AND content_type IN (");
        let mut separated = count_query.separated(", ");
        for ct in &filter.content_types {
            separated.push_bind(ct.clone());
        }
        separated.push_unseparated(")");
    }
    if !filter.source_apps.is_empty() {
        count_query.push(" AND source_app IN (");
        let mut separated = count_query.separated(", ");
        for sa in &filter.source_apps {
            separated.push_bind(sa.clone());
        }
        separated.push_unseparated(")");
    }

    let total: i64 = count_query
        .build_query_scalar()
        .fetch_one(&state.db)
        .await
        .unwrap_or(entries.len() as i64);

    let items: Vec<ClipboardEntryListItemDto> = entries.into_iter().map(|r| {
        ClipboardEntryListItemDto {
            id: r.id,
            preview: r.content.chars().take(200).collect(),
            content_type: r.content_type,
            source_app: r.source_app,
            captured_at: r.captured_at,
            size_bytes: r.size_bytes as u32,
            is_pinned: r.is_pinned != 0,
            match_score: None,
            promoted_to_snippet_id: r.promoted_to_snippet_id,
        }
    }).collect();

    let total_u32 = total as u32;
    let has_next = (offset + items.len() as u32) < total_u32;

    Ok(PagedResultDto {
        total: total_u32,
        has_next,
        has_prev: page > 0,
        items,
        page,
        page_size,
    })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetLocationDto {
    pub _type: String,
    pub folder_id: Option<String>,
}

#[tauri::command]
pub async fn promote_clipboard_to_snippet(
    entry_id: String,
    title: Option<String>,
    location: SnippetLocationDto,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    #[derive(sqlx::FromRow)]
    struct ClipRow {
        content: String,
        content_type: String,
        source_app: Option<String>,
    }

    let clip = sqlx::query_as::<_, ClipRow>("SELECT content, content_type, source_app FROM clipboard_history WHERE id = ?")
        .bind(&entry_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Clipboard entry not found".to_string())?;

    let snippet_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let final_title = title.unwrap_or_else(|| {
        let trimmed = clip.content.trim();
        if trimmed.is_empty() {
            "Clipboard-Import".to_string()
        } else {
            let limit = trimmed.chars().take(60).collect::<String>();
            limit
        }
    });

    let is_template = if clip.content.contains("{{") && clip.content.contains("}}") { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, source_app, location_type, location_folder_id, created_at, updated_at, is_template)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&snippet_id)
    .bind(&final_title)
    .bind(&clip.content)
    .bind(&clip.content_type)
    .bind(&clip.source_app)
    .bind(&location._type)
    .bind(&location.folder_id)
    .bind(now)
    .bind(now)
    .bind(is_template)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE clipboard_history SET promoted_to_snippet_id = ? WHERE id = ?")
        .bind(&snippet_id)
        .bind(&entry_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(snippet_id)
}
#[tauri::command]
pub async fn pin_clipboard_entry(
    id: String,
    pinned: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pinned_val = if pinned { 1 } else { 0 };
    sqlx::query("UPDATE clipboard_history SET is_pinned = ? WHERE id = ?")
        .bind(pinned_val)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_clipboard_entry(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM clipboard_history WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard_history(
    keep_pinned: bool,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let query = if keep_pinned {
        "DELETE FROM clipboard_history WHERE is_pinned = 0"
    } else {
        "DELETE FROM clipboard_history"
    };

    let result = sqlx::query(query)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() as u32)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardEntryDto {
    pub id: String,
    pub content: String,
    pub content_hash: String,
    pub content_type: String,
    pub source_app: Option<String>,
    pub captured_at: i64,
    pub size_bytes: u32,
    pub line_count: u32,
    pub word_count: u32,
    pub is_pinned: bool,
    pub tags: Vec<String>,
    pub promoted_to_snippet_id: Option<String>,
}

#[tauri::command]
pub async fn get_clipboard_entry(
    id: String,
    state: State<'_, AppState>,
) -> Result<ClipboardEntryDto, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        content: String,
        content_hash: String,
        content_type: String,
        source_app: Option<String>,
        captured_at: i64,
        size_bytes: i64,
        is_pinned: i64,
        promoted_to_snippet_id: Option<String>,
    }

    let row = sqlx::query_as::<_, Row>(
        "SELECT id, content, content_hash, content_type, source_app, captured_at, size_bytes, is_pinned, promoted_to_snippet_id FROM clipboard_history WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Clipboard entry not found".to_string())?;

    let line_count = row.content.lines().count() as u32;
    let word_count = row.content.split_whitespace().count() as u32;

    Ok(ClipboardEntryDto {
        id: row.id,
        content: row.content,
        content_hash: row.content_hash,
        content_type: row.content_type,
        source_app: row.source_app,
        captured_at: row.captured_at,
        size_bytes: row.size_bytes as u32,
        line_count,
        word_count,
        is_pinned: row.is_pinned != 0,
        tags: vec![],
        promoted_to_snippet_id: row.promoted_to_snippet_id,
    })
}

#[tauri::command]
pub async fn read_clipboard_now(
    _state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let output = tokio::process::Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .await;

    if let Ok(out) = output {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            return Ok(if text.is_empty() { None } else { Some(text) });
        }
    }

    let mut board = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    match board.get_text() {
        Ok(text) => Ok(if text.is_empty() { None } else { Some(text) }),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub async fn write_to_clipboard(
    content: String,
    snippet_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let copy_res = tokio::process::Command::new("wl-copy")
        .arg(&content)
        .status()
        .await;

    if copy_res.is_err() || !copy_res.as_ref().map(|s| s.success()).unwrap_or(false) {
        let mut board = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        board.set_text(content.clone()).map_err(|e| e.to_string())?;
    }

    if let Some(s_id) = snippet_id {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query("UPDATE snippets SET usage_count = usage_count + 1, last_used_at = ? WHERE id = ?")
            .bind(now)
            .bind(s_id)
            .execute(&state.db)
            .await
            .ok();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_fts5_query() {
        assert_eq!(format_fts5_query("hello world"), "\"hello world\"");
        assert_eq!(format_fts5_query("  foo \"bar\"  "), "\"foo \"\"bar\"\"\"");
        assert_eq!(format_fts5_query("   "), "");
    }
}

