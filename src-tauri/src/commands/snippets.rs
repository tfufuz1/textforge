use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use tauri::State;
use crate::AppState;


#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DateRangeFilterDto {
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub preset: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SizeRangeFilterDto {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SnippetFilterDto {
    pub search_query: Option<String>,
    pub content_types: Vec<String>,
    pub tags: Vec<String>,
    pub location_type: Option<String>,
    pub folder_id: Option<String>,
    pub is_trashed: Option<bool>,
    pub is_pinned: Option<bool>,
    pub is_favorite: Option<bool>,
    pub is_template: Option<bool>,
    pub tags_mode: Option<String>, // "all" | "any"
    pub date_field: Option<String>, // "createdAt" | "updatedAt"
    pub date_range: Option<DateRangeFilterDto>,
    pub size_range: Option<SizeRangeFilterDto>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetListItemDto {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub content_type: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_pinned: bool,
    pub is_favorite: bool,
    pub color: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetDto {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub source_app: Option<String>,
    pub location_type: String,
    pub folder_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub usage_count: u32,
    pub is_pinned: bool,
    pub is_template: bool,
    pub is_favorite: bool,
    pub color: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSnippetDto {
    pub title: String,
    pub content: String,
    pub content_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub folder_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnippetDto {
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_pinned: Option<bool>,
    pub is_favorite: Option<bool>,
    pub color: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordFreqDto {
    pub word: String,
    pub count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStatsDto {
    pub char_count: u32,
    pub char_no_space_count: u32,
    pub word_count: u32,
    pub line_count: u32,
    pub paragraph_count: u32,
    pub sentence_count: u32,
    pub estimated_tokens: u32,
    pub unique_word_count: u32,
    pub avg_word_length: f32,
    pub longest_word: String,
    pub most_frequent_words: Vec<WordFreqDto>,
    pub avg_sentence_length: f32,
    pub flesch_kincaid_grade: Option<f32>,
    pub avg_line_length: f32,
    pub longest_line_length: u32,
    pub empty_line_count: u32,
    pub reading_time_ms: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVariableDto {
    pub name: String,
    pub has_default: bool,
    pub default_val: Option<String>,
    pub filter: Option<String>,
    pub is_special: bool,
    pub is_required: bool,
    pub occurrences: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTemplateDto {
    pub variables: Vec<TemplateVariableDto>,
    pub required_vars: Vec<String>,
    pub optional_vars: Vec<String>,
    pub has_conditionals: bool,
    pub has_loops: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateRenderResultDto {
    pub output: String,
    pub resolved_variables: HashMap<String, String>,
    pub unresolved_vars: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineDto {
    pub kind: String, // 'equal' | 'insert' | 'delete'
    pub old_line_num: Option<u32>,
    pub new_line_num: Option<u32>,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResultDto {
    pub lines: Vec<DiffLineDto>,
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub unchanged: u32,
    pub similarity: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptVersionDto {
    pub id: String,
    pub script_id: String,
    pub js_code: String,
    pub change_note: Option<String>,
    pub created_at: i64,
}

#[tauri::command]
pub async fn list_snippets(
    filter: Option<SnippetFilterDto>,
    state: State<'_, AppState>,
) -> Result<Vec<SnippetListItemDto>, String> {
    let filter = filter.unwrap_or_default();

    let mut query = sqlx::QueryBuilder::new(
        "SELECT s.id, s.title, s.content, s.content_type, s.created_at, s.updated_at, s.is_pinned, s.favorite, s.color, GROUP_CONCAT(t.tag) as tags 
         FROM snippets s 
         LEFT JOIN snippet_tags t ON s.id = t.snippet_id 
         WHERE 1=1"
    );

    if let Some(search) = &filter.search_query {
        let formatted = crate::commands::clipboard::format_fts5_query(search);
        if !formatted.is_empty() {
            query.push(" AND s.rowid IN (SELECT rowid FROM snippets_fts WHERE snippets_fts MATCH ");
            query.push_bind(formatted);
            query.push(")");
        }
    }

    if !filter.content_types.is_empty() {
        query.push(" AND s.content_type IN (");
        let mut separated = query.separated(", ");
        for ct in &filter.content_types {
            separated.push_bind(ct.clone());
        }
        separated.push_unseparated(")");
    }

    if !filter.tags.is_empty() {
        let mode = filter.tags_mode.as_deref().unwrap_or("all");
        if mode == "any" {
            query.push(" AND EXISTS (SELECT 1 FROM snippet_tags st WHERE st.snippet_id = s.id AND st.tag IN (");
            let mut separated = query.separated(", ");
            for tag in &filter.tags {
                separated.push_bind(tag.clone());
            }
            separated.push_unseparated("))");
        } else {
            for tag in &filter.tags {
                query.push(" AND EXISTS (SELECT 1 FROM snippet_tags st WHERE st.snippet_id = s.id AND st.tag = ");
                query.push_bind(tag.clone());
                query.push(")");
            }
        }
    }

    if let Some(pinned) = filter.is_pinned {
        query.push(" AND s.is_pinned = ");
        query.push_bind(if pinned { 1 } else { 0 });
    }

    if let Some(favorite) = filter.is_favorite {
        query.push(" AND s.is_favorite = ");
        query.push_bind(if favorite { 1 } else { 0 });
    }

    if let Some(template) = filter.is_template {
        query.push(" AND s.is_template = ");
        query.push_bind(if template { 1 } else { 0 });
    }

    if let Some(true) = filter.is_trashed {
        query.push(" AND s.location_type = 'trash'");
    } else {
        query.push(" AND s.location_type != 'trash'");
        if let Some(loc_type) = &filter.location_type {
            if loc_type == "folder" {
                if let Some(fid) = &filter.folder_id {
                    query.push(" AND s.location_type = 'folder' AND s.location_folder_id = ");
                    query.push_bind(fid.clone());
                }
            } else if loc_type == "inbox" {
                query.push(" AND s.location_type = 'inbox'");
            } else if loc_type == "archive" {
                query.push(" AND s.location_type = 'archive'");
            }
        }
    }

    // Date Range
    if let Some(dr) = &filter.date_range {
        let field = match filter.date_field.as_deref().unwrap_or("updatedAt") {
            "createdAt" => "s.created_at",
            _ => "s.updated_at",
        };
        if let Some(from_ts) = dr.from {
            query.push(&format!(" AND {} >= ", field));
            query.push_bind(from_ts);
        }
        if let Some(to_ts) = dr.to {
            query.push(&format!(" AND {} <= ", field));
            query.push_bind(to_ts);
        }
    }

    // Size Range
    if let Some(sr) = &filter.size_range {
        if let Some(min_size) = sr.min {
            query.push(" AND LENGTH(s.content) >= ");
            query.push_bind(min_size);
        }
        if let Some(max_size) = sr.max {
            query.push(" AND LENGTH(s.content) <= ");
            query.push_bind(max_size);
        }
    }

    query.push(" GROUP BY s.id");

    // Sorting
    let sort_field = match filter.sort_by.as_deref().unwrap_or("updatedAt") {
        "title" => "s.title",
        "createdAt" => "s.created_at",
        "size" => "LENGTH(s.content)",
        "usageCount" => "s.usage_count",
        _ => "s.updated_at",
    };
    let sort_dir = match filter.sort_dir.as_deref().unwrap_or("desc") {
        "asc" => "ASC",
        _ => "DESC",
    };

    query.push(&format!(" ORDER BY s.is_pinned DESC, {} {}", sort_field, sort_dir));
    query.push(" LIMIT 100");

    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        title: String,
        content: String,
        content_type: String,
        created_at: i64,
        updated_at: i64,
        is_pinned: i64,
        favorite: i64,
        color: Option<String>,
        tags: Option<String>,
    }

    let entries: Vec<Row> = query
        .build_query_as()
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let items: Vec<SnippetListItemDto> = entries.into_iter().map(|r| {
        let tags = r.tags.map(|t| t.split(',').map(|s| s.to_string()).collect()).unwrap_or_default();
        SnippetListItemDto {
            id: r.id,
            title: r.title,
            preview: r.content.chars().take(200).collect(),
            content_type: r.content_type,
            created_at: r.created_at,
            updated_at: r.updated_at,
            is_pinned: r.is_pinned != 0,
            is_favorite: r.favorite != 0,
            color: r.color,
            tags,
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn get_snippet(
    id: String,
    state: State<'_, AppState>,
) -> Result<SnippetDto, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        title: String,
        content: String,
        content_type: String,
        source_app: Option<String>,
        location_type: String,
        location_folder_id: Option<String>,
        created_at: i64,
        updated_at: i64,
        last_used_at: Option<i64>,
        usage_count: i64,
        is_pinned: i64,
        is_template: i64,
        favorite: i64,
        color: Option<String>,
    }

    let row = sqlx::query_as::<_, Row>(
        "SELECT id, title, content, content_type, source_app, location_type, location_folder_id, created_at, updated_at, last_used_at, usage_count, is_pinned, is_template, favorite, color FROM snippets WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Snippet not found".to_string())?;

    let tags_rows: Vec<(String,)> = sqlx::query_as("SELECT tag FROM snippet_tags WHERE snippet_id = ?")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let tags = tags_rows.into_iter().map(|r| r.0).collect();

    Ok(SnippetDto {
        id: row.id,
        title: row.title,
        content: row.content,
        content_type: row.content_type,
        source_app: row.source_app,
        location_type: row.location_type,
        folder_id: row.location_folder_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_used_at: row.last_used_at,
        usage_count: row.usage_count as u32,
        is_pinned: row.is_pinned != 0,
        is_template: row.is_template != 0,
        is_favorite: row.favorite != 0,
        color: row.color,
        tags,
    })
}

#[tauri::command]
pub async fn create_snippet(
    draft: CreateSnippetDto,
    state: State<'_, AppState>,
) -> Result<SnippetDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let content_type = draft.content_type.unwrap_or_else(|| "plain_text".to_string());
    let is_template = if draft.content.contains("{{") && draft.content.contains("}}") { 1 } else { 0 };

    let location_type = if draft.folder_id.is_some() { "folder" } else { "root" };

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO snippets (id, title, content, content_type, location_type, location_folder_id, created_at, updated_at, is_template)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&draft.title)
    .bind(&draft.content)
    .bind(&content_type)
    .bind(&location_type)
    .bind(&draft.folder_id)
    .bind(now)
    .bind(now)
    .bind(is_template)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let tags = draft.tags.unwrap_or_default();
    for tag in &tags {
        sqlx::query("INSERT OR IGNORE INTO snippet_tags (snippet_id, tag) VALUES (?, ?)")
            .bind(&id)
            .bind(tag)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    let created_snippet = SnippetDto {
        id: id.clone(),
        title: draft.title.clone(),
        content: draft.content.clone(),
        content_type,
        source_app: None,
        location_type: location_type.to_string(),
        folder_id: draft.folder_id,
        created_at: now,
        updated_at: now,
        last_used_at: None,
        usage_count: 0,
        is_pinned: false,
        is_template: is_template != 0,
        is_favorite: false,
        color: None,
        tags,
    };

    let undo_entry = crate::commands::undo::UndoEntryDto {
        id: uuid::Uuid::new_v4().to_string(),
        performed_at: now,
        description: format!("Snippet '{}' erstellt", draft.title),
        action: crate::commands::undo::UndoActionDto::SnippetCreate {
            created: serde_json::to_value(&created_snippet).unwrap_or_default(),
        },
    };

    if let Ok(mut stack) = state.undo_stack.lock() {
        stack.push(undo_entry);
    }

    Ok(created_snippet)
}

#[tauri::command]
pub async fn update_snippet(
    id: String,
    draft: UpdateSnippetDto,
    state: State<'_, AppState>,
) -> Result<SnippetDto, String> {
    let before_snippet = get_snippet(id.clone(), state.clone()).await.ok();
    let now = chrono::Utc::now().timestamp_millis();
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    if let Some(title) = &draft.title {
        sqlx::query("UPDATE snippets SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title).bind(now).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    if let Some(content) = &draft.content {
        let is_template = if content.contains("{{") && content.contains("}}") { 1 } else { 0 };
        sqlx::query("UPDATE snippets SET content = ?, is_template = ?, updated_at = ? WHERE id = ?")
            .bind(content).bind(is_template).bind(now).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    if let Some(ct) = &draft.content_type {
        sqlx::query("UPDATE snippets SET content_type = ?, updated_at = ? WHERE id = ?")
            .bind(ct).bind(now).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    if let Some(pinned) = draft.is_pinned {
        let p_val = if pinned { 1 } else { 0 };
        sqlx::query("UPDATE snippets SET is_pinned = ?, updated_at = ? WHERE id = ?")
            .bind(p_val).bind(now).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    if let Some(favorite) = draft.is_favorite {
        let f_val = if favorite { 1 } else { 0 };
        sqlx::query("UPDATE snippets SET favorite = ?, updated_at = ? WHERE id = ?")
            .bind(f_val).bind(now).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    if let Some(color) = &draft.color {
        sqlx::query("UPDATE snippets SET color = ?, updated_at = ? WHERE id = ?")
            .bind(color).bind(now).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    if let Some(tags) = &draft.tags {
        sqlx::query("DELETE FROM snippet_tags WHERE snippet_id = ?").bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        for tag in tags {
            sqlx::query("INSERT INTO snippet_tags (snippet_id, tag) VALUES (?, ?)").bind(&id).bind(tag).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    let updated = get_snippet(id, state.clone()).await?;

    if let Some(before) = before_snippet {
        let undo_entry = crate::commands::undo::UndoEntryDto {
            id: uuid::Uuid::new_v4().to_string(),
            performed_at: now,
            description: format!("Snippet '{}' aktualisiert", updated.title),
            action: crate::commands::undo::UndoActionDto::SnippetUpdate {
                before: serde_json::to_value(&before).unwrap_or_default(),
                after: serde_json::to_value(&updated).unwrap_or_default(),
            },
        };

        if let Ok(mut stack) = state.undo_stack.lock() {
            stack.push(undo_entry);
        }
    }

    Ok(updated)
}

#[tauri::command]
pub async fn duplicate_snippet(
    id: String,
    state: State<'_, AppState>,
) -> Result<SnippetDto, String> {
    let original = get_snippet(id, state.clone()).await?;
    let new_title = format!("{} (Kopie)", original.title);
    create_snippet(
        CreateSnippetDto {
            title: new_title,
            content: original.content,
            content_type: Some(original.content_type),
            tags: Some(original.tags),
            folder_id: original.folder_id,
        },
        state,
    )
    .await
}

#[tauri::command]
pub async fn trash_snippet(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE snippets SET location_type = 'trash', updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn restore_snippet(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE snippets SET location_type = 'root', location_folder_id = NULL, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_snippet_permanently(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM snippets WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn empty_trash(
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let res = sqlx::query("DELETE FROM snippets WHERE location_type = 'trash'")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.rows_affected() as u32)
}

#[tauri::command]
pub async fn list_all_tags(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT tag FROM snippet_tags ORDER BY tag ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

#[tauri::command]
pub async fn list_folders(
    state: State<'_, AppState>,
) -> Result<Vec<FolderDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        parent_id: Option<String>,
        icon: Option<String>,
        created_at: i64,
    }

    let rows = sqlx::query_as::<_, Row>("SELECT id, name, parent_id, icon, created_at FROM folders ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| FolderDto {
        id: r.id,
        name: r.name,
        parent_id: r.parent_id,
        icon: r.icon,
        created_at: r.created_at,
    }).collect())
}

#[tauri::command]
pub async fn create_folder(
    name: String,
    parent_id: Option<String>,
    icon: Option<String>,
    state: State<'_, AppState>,
) -> Result<FolderDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("INSERT INTO folders (id, name, parent_id, icon, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(&name).bind(&parent_id).bind(&icon).bind(now)
        .execute(&state.db).await.map_err(|e| e.to_string())?;

    Ok(FolderDto { id, name, parent_id, icon, created_at: now })
}

#[tauri::command]
pub async fn rename_folder(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("UPDATE folders SET name = ? WHERE id = ?")
        .bind(&name).bind(&id)
        .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_folder(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM folders WHERE id = ?")
        .bind(&id)
        .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn compute_text_stats(
    content: String,
) -> Result<TextStatsDto, String> {
    let char_count = content.chars().count() as u32;
    let char_no_space_count = content.chars().filter(|c| !c.is_whitespace()).count() as u32;
    let words: Vec<&str> = content.split_whitespace().collect();
    let word_count = words.len() as u32;
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len().max(1) as u32;
    let empty_line_count = lines.iter().filter(|l| l.trim().is_empty()).count() as u32;
    let paragraph_count = content.split("\n\n").filter(|p| !p.trim().is_empty()).count().max(1) as u32;
    let sentence_count = content.split(&['.', '!', '?'][..]).filter(|s| !s.trim().is_empty()).count().max(1) as u32;
    let estimated_tokens = (char_count / 4).max(1);

    let mut freq_map: HashMap<String, u32> = HashMap::new();
    let mut longest_word = String::new();
    let mut total_word_len = 0usize;

    for w in &words {
        let clean = w.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<String>();
        if clean.is_empty() { continue; }
        if clean.len() > longest_word.len() {
            longest_word = clean.clone();
        }
        total_word_len += clean.len();
        *freq_map.entry(clean).or_insert(0) += 1;
    }

    let unique_word_count = freq_map.len() as u32;
    let avg_word_length = if word_count > 0 { total_word_len as f32 / word_count as f32 } else { 0.0 };

    let mut freq_vec: Vec<WordFreqDto> = freq_map.into_iter().map(|(w, c)| WordFreqDto { word: w, count: c }).collect();
    freq_vec.sort_by(|a, b| b.count.cmp(&a.count));
    freq_vec.truncate(10);

    let avg_sentence_length = if sentence_count > 0 { word_count as f32 / sentence_count as f32 } else { 0.0 };
    let longest_line_length = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u32;
    let total_line_len: usize = lines.iter().map(|l| l.chars().count()).sum();
    let avg_line_length = if line_count > 0 { total_line_len as f32 / line_count as f32 } else { 0.0 };
    let reading_time_ms = (word_count as f32 / 200.0 * 60.0 * 1000.0) as u32;

    let flesch_kincaid_grade = if word_count > 0 && sentence_count > 0 {
        let grade = 0.39 * (word_count as f32 / sentence_count as f32) + 11.8 * 1.5 - 15.59;
        Some(grade.max(0.0))
    } else {
        None
    };

    Ok(TextStatsDto {
        char_count,
        char_no_space_count,
        word_count,
        line_count,
        paragraph_count,
        sentence_count,
        estimated_tokens,
        unique_word_count,
        avg_word_length,
        longest_word,
        most_frequent_words: freq_vec,
        avg_sentence_length,
        flesch_kincaid_grade,
        avg_line_length,
        longest_line_length,
        empty_line_count,
        reading_time_ms,
    })
}

#[tauri::command]
pub async fn parse_template(
    content: String,
) -> Result<ParsedTemplateDto, String> {
    // Syntax: {{name}}, {{name:Default}}, {{name|filter}}, {{name:Default|filter}}, {{name|f1|f2}} (chained)
    // Ignoriert {{#if ...}}, {{/if}}, {{#each ...}}, {{/each}}, {{#else}} Block-Tags
    let re = regex::Regex::new(
        r"\{\{\s*([a-zA-Z0-9_\-]+)(?::([^|{}]*))?((?:\|[a-zA-Z0-9_:]+)*)\s*\}\}"
    ).map_err(|e| e.to_string())?;

    // Use an ordered map to preserve first-seen order
    let mut var_map: IndexMap<String, (u32, Option<String>, Option<String>)>
        = IndexMap::new();


    for cap in re.captures_iter(&content) {
        let var_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        // Skip block tags: #if, /if, #else, #each, /each, @index, @first, @last, this
        if var_name.starts_with('#') || var_name.starts_with('/')
            || var_name.starts_with('@') || var_name == "this"
            || var_name == "else" {
            continue;
        }

        let default_val = cap.get(2).map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        let filter = cap.get(3).map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());

        let entry = var_map.entry(var_name).or_insert((0, default_val.clone(), filter.clone()));
        entry.0 += 1;
        // Keep first default/filter seen
        if entry.1.is_none() { entry.1 = default_val; }
        if entry.2.is_none() { entry.2 = filter; }
    }

    let variables: Vec<TemplateVariableDto> = var_map.into_iter().map(|(name, (occurrences, default_val, filter))| {
        let has_default = default_val.is_some();
        let is_required = !has_default && !name.starts_with('_');
        TemplateVariableDto {
            name: name.clone(),
            has_default,
            default_val,
            filter,
            is_special: name.starts_with('_'),
            is_required,
            occurrences,
        }
    }).collect();

    let required_vars: Vec<String> = variables.iter()
        .filter(|v| v.is_required)
        .map(|v| v.name.clone())
        .collect();
    let optional_vars: Vec<String> = variables.iter()
        .filter(|v| !v.is_required)
        .map(|v| v.name.clone())
        .collect();

    // Fix: korrekte Erkennung der Syntax {{#if ...}} und {{#each ...}}
    let has_conditionals = content.contains("{{#if ") || content.contains("{{#unless ");
    let has_loops = content.contains("{{#each ");

    Ok(ParsedTemplateDto {
        variables,
        required_vars,
        optional_vars,
        has_conditionals,
        has_loops,
    })
}

/// Injiziert Spezial-Variablen in den Template-Context (gemäß § 6.1)
fn inject_special_vars(context: &mut HashMap<String, String>) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let timestamp_ms = now.as_millis();

    // Einfaches Datum/Zeit ohne externe Crate
    let secs = now.as_secs();
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    // Datum berechnen (Gregorianisch)
    let (year, month, day) = epoch_days_to_date(days_since_epoch);
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
    let time_str = format!("{:02}:{:02}:{:02}", hour, minute, second);
    let datetime_str = format!("{} {}", date_str, time_str);

    context.entry("_date".to_string()).or_insert(date_str);
    context.entry("_time".to_string()).or_insert(time_str);
    context.entry("_datetime".to_string()).or_insert(datetime_str);
    context.entry("_timestamp".to_string()).or_insert(timestamp_ms.to_string());
    context.entry("_uuid".to_string()).or_insert_with(|| uuid::Uuid::new_v4().to_string());

    // Clipboard-Inhalt ermitteln falls nicht bereits gesetzt
    if !context.contains_key("_clipboard") {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(text) = cb.get_text() {
                context.insert("_clipboard".to_string(), text);
            }
        }
    }

    // Textstatistiken berechnen für _input oder _clipboard
    let ref_text = context.get("_input").or_else(|| context.get("_clipboard")).cloned().unwrap_or_default();
    let char_count = ref_text.chars().count();
    let word_count = if ref_text.trim().is_empty() { 0 } else { ref_text.split_whitespace().count() };
    let line_count = if ref_text.is_empty() { 0 } else { ref_text.lines().count() };

    context.entry("_charcount".to_string()).or_insert(char_count.to_string());
    context.entry("_wordcount".to_string()).or_insert(word_count.to_string());
    context.entry("_linecount".to_string()).or_insert(line_count.to_string());
}

/// Einfache Gregorianische Datumsberechnung aus Unix-Epoch-Tagen
fn epoch_days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithmus: https://www.researchgate.net/publication/316558298
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Verarbeitet {{#each items}}...{{/each}} Schleifen
fn process_loops(template: &str, context: &HashMap<String, String>) -> String {
    let each_re = regex::Regex::new(
        r"(?s)\{\{#each\s+([a-zA-Z0-9_\-]+)\}\}(.*?)\{\{/each\}\}"
    ).unwrap();

    each_re.replace_all(template, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        let raw_val = context.get(var_name).cloned().unwrap_or_default();
        let items: Vec<String> = if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&raw_val) {
            parsed
        } else if !raw_val.is_empty() {
            raw_val.lines().map(|l| l.to_string()).collect()
        } else {
            Vec::new()
        };

        if items.is_empty() {
            return String::new();
        }

        let total = items.len();
        let mut loop_output = String::new();

        for (idx, item) in items.iter().enumerate() {
            let is_first = idx == 0;
            let is_last = idx == total - 1;

            let mut iterated = body.to_string();
            iterated = iterated.replace("{{this}}", item);
            iterated = iterated.replace("{{@index}}", &idx.to_string());
            iterated = iterated.replace("{{@first}}", if is_first { "true" } else { "false" });
            iterated = iterated.replace("{{@last}}", if is_last { "true" } else { "false" });

            loop_output.push_str(&iterated);
        }

        loop_output
    }).to_string()
}

/// Verarbeitet {{#if var}}...{{#else}}...{{/if}} und {{#unless var}}...{{/unless}} Blöcke
fn process_conditionals(template: &str, context: &HashMap<String, String>) -> String {
    // Verarbeitet {{#if NAME}}...{{/if}} und {{#if NAME}}...{{#else}}...{{/if}}
    let if_re = regex::Regex::new(
        r"(?s)\{\{#if\s+(\w+)\}\}(.*?)(?:\{\{#else\}\}(.*?))?\{\{/if\}\}"
    ).unwrap();

    let unless_re = regex::Regex::new(
        r"(?s)\{\{#unless\s+(\w+)\}\}(.*?)\{\{/unless\}\}"
    ).unwrap();

    let result = if_re.replace_all(template, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let then_block = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let else_block = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        let is_truthy = context.get(var_name)
            .map(|v| !v.is_empty() && v != "false" && v != "0")
            .unwrap_or(false);

        if is_truthy { then_block.to_string() } else { else_block.to_string() }
    });

    let result = unless_re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let content = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        let is_truthy = context.get(var_name)
            .map(|v| !v.is_empty() && v != "false" && v != "0")
            .unwrap_or(false);

        if !is_truthy { content.to_string() } else { String::new() }
    });

    result.to_string()
}

#[tauri::command]
pub async fn render_template(
    content: String,
    context: HashMap<String, String>,
    strict: bool,
) -> Result<TemplateRenderResultDto, String> {
    // Full syntax: {{name}}, {{name:default}}, {{name|filter}}, {{name:default|filter}}, {{name|f1|f2}} (chained)
    let re = regex::Regex::new(
        r"\{\{\s*([a-zA-Z0-9_\-]+)(?::([^|{}]*))?((?:\|[a-zA-Z0-9_:]+)*)\s*\}\}"
    ).map_err(|e| e.to_string())?;

    // Schritt 1: Spezial-Variablen injizieren
    let mut full_context = context.clone();
    inject_special_vars(&mut full_context);

    // Schritt 2: {{#each}} Schleifen verarbeiten
    let looped_content = process_loops(&content, &full_context);

    // Schritt 3: {{#if}} / {{#unless}} Conditionals verarbeiten
    let processed_content = process_conditionals(&looped_content, &full_context);

    let mut output = processed_content.clone();
    let mut resolved_variables: HashMap<String, String> = HashMap::new();
    let mut unresolved_vars: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Collect all matches first to avoid borrow issues with replace
    let matches: Vec<(String, String, Option<String>, Option<String>)> = re
        .captures_iter(&processed_content)
        .filter_map(|cap| {
            let name = cap.get(1).unwrap().as_str().to_string();
            // Block-Tags überspringen
            if name.starts_with('#') || name.starts_with('/')
                || name.starts_with('@') || name == "this" || name == "else" {
                return None;
            }
            let full = cap.get(0).unwrap().as_str().to_string();
            let default_val = cap.get(2).map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty());
            let filter = cap.get(3).map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty());
            Some((full, name, default_val, filter))
        })
        .collect();

    for (full, var_name, default_val, filter) in matches {
        let raw_val = full_context.get(&var_name).cloned()
            .or(default_val.clone());

        if let Some(val) = raw_val {
            // Apply chained filters if present
            let filtered_val = apply_template_filters(&val, filter.as_deref());
            output = output.replace(&full, &filtered_val);
            resolved_variables.insert(var_name.clone(), filtered_val);
        } else {
            if !unresolved_vars.contains(&var_name) {
                unresolved_vars.push(var_name.clone());
            }
            if strict {
                // Im Strict-Modus: Warnung, Variable bleibt als {{var}} erhalten
                warnings.push(format!("Fehlende Variable: {{{{{}}}}} ", var_name));
                // Variable bleibt unverändert im Output (kein replace)
            }
            // Im Non-Strict-Modus: Variable bleibt als {{var}} erhalten (Spec § 6.3)
            // KEIN output.replace() → Variable bleibt sichtbar
        }
    }

    Ok(TemplateRenderResultDto {
        output,
        resolved_variables,
        unresolved_vars,
        warnings,
    })
}

/// Wendet eine Kette von Template-Filter-Operatoren auf einen Wert an.
/// Filter-Operatoren gemäß § 6.1:
/// upper, lower, title, trim, slug, snake, camel, pascal, json, base64, url,
/// truncate:N, lines, words, default:X, reverse, first, last, len
/// Unterstützt chained filters: "|upper|truncate:10" → ["upper", "truncate:10"]
fn apply_template_filters(value: &str, filter_chain: Option<&str>) -> String {
    match filter_chain {
        None | Some("") => value.to_string(),
        Some(chain) => {
            // Chain kommt als "|upper|truncate:10" oder "upper|truncate:10"
            let filters: Vec<&str> = chain.split('|')
                .map(|f| f.trim())
                .filter(|f| !f.is_empty())
                .collect();
            let mut result = value.to_string();
            for f in filters {
                result = apply_single_template_filter(&result, f);
            }
            result
        }
    }
}

/// Wendet einen einzelnen Template-Filter auf einen Wert an.
fn apply_single_template_filter(value: &str, filter: &str) -> String {
    if filter.starts_with("truncate:") {
        let n: usize = filter["truncate:".len()..].parse().unwrap_or(100);
        let chars: Vec<char> = value.chars().collect();
        if chars.len() > n {
            let truncated: String = chars[..n].iter().collect();
            return format!("{}…", truncated);
        }
        return value.to_string();
    }
    if filter.starts_with("default:") {
        let fallback = &filter["default:".len()..];
        return if value.is_empty() { fallback.to_string() } else { value.to_string() };
    }

    match filter {
        "upper" => value.to_uppercase(),
        "lower" => value.to_lowercase(),
        "title" => {
            let mut result = String::new();
            let mut cap_next = true;
            for c in value.chars() {
                if c.is_whitespace() {
                    cap_next = true;
                    result.push(c);
                } else if cap_next {
                    result.extend(c.to_uppercase());
                    cap_next = false;
                } else {
                    result.extend(c.to_lowercase());
                }
            }
            result
        }
        "trim" => value.trim().to_string(),
        "slug" => {
            let re1 = regex::Regex::new(r"[^a-zA-Z0-9\s-]").unwrap();
            let re2 = regex::Regex::new(r"[\s_]+").unwrap();
            let clean = re1.replace_all(value, "").to_lowercase();
            re2.replace_all(&clean, "-").trim_matches('-').to_string()
        }
        "snake" => {
            let re = regex::Regex::new(r"([a-z0-9])([A-Z])").unwrap();
            re.replace_all(value, "${1}_${2}").to_lowercase()
                .replace([' ', '-'], "_")
        }
        "camel" => {
            let mut result = String::new();
            let mut cap_next = false;
            for (i, c) in value.chars().enumerate() {
                if c == '_' || c == '-' || c == ' ' {
                    cap_next = true;
                } else if cap_next {
                    result.extend(c.to_uppercase());
                    cap_next = false;
                } else if i == 0 {
                    result.extend(c.to_lowercase());
                } else {
                    result.push(c);
                }
            }
            result
        }
        "pascal" => {
            let mut result = String::new();
            let mut cap_next = true;
            for c in value.chars() {
                if c == '_' || c == '-' || c == ' ' {
                    cap_next = true;
                } else if cap_next {
                    result.extend(c.to_uppercase());
                    cap_next = false;
                } else {
                    result.push(c);
                }
            }
            result
        }
        "json" => {
            serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value))
        }
        "base64" => {
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD.encode(value.as_bytes())
        }
        "url" => urlencoding::encode(value).to_string(),
        "reverse" => value.chars().rev().collect(),
        "first" => value.lines().next().unwrap_or("").to_string(),
        "last" => value.lines().last().unwrap_or("").to_string(),
        "lines" => value.lines().count().to_string(),
        "words" => value.split_whitespace().count().to_string(),
        "len" => value.chars().count().to_string(),
        f => {
            eprintln!("Unknown template filter: {}", f);
            value.to_string()
        }
    }
}



#[tauri::command]
pub async fn compute_diff(
    original: String,
    modified: String,
) -> Result<DiffResultDto, String> {
    let diff = similar::TextDiff::from_lines(&original, &modified);
    let mut lines = Vec::new();
    let mut added_lines = 0u32;
    let mut deleted_lines = 0u32;
    let mut unchanged = 0u32;

    let mut old_idx = 1u32;
    let mut new_idx = 1u32;

    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Equal => {
                unchanged += 1;
                lines.push(DiffLineDto {
                    kind: "equal".to_string(),
                    old_line_num: Some(old_idx),
                    new_line_num: Some(new_idx),
                    content: change.value().to_string(),
                });
                old_idx += 1;
                new_idx += 1;
            }
            similar::ChangeTag::Delete => {
                deleted_lines += 1;
                lines.push(DiffLineDto {
                    kind: "delete".to_string(),
                    old_line_num: Some(old_idx),
                    new_line_num: None,
                    content: change.value().to_string(),
                });
                old_idx += 1;
            }
            similar::ChangeTag::Insert => {
                added_lines += 1;
                lines.push(DiffLineDto {
                    kind: "insert".to_string(),
                    old_line_num: None,
                    new_line_num: Some(new_idx),
                    content: change.value().to_string(),
                });
                new_idx += 1;
            }
        }
    }

    let total = added_lines + deleted_lines + unchanged;
    let similarity = if total > 0 { unchanged as f32 / total as f32 } else { 1.0 };

    Ok(DiffResultDto {
        lines,
        added_lines,
        deleted_lines,
        unchanged,
        similarity,
    })
}

#[tauri::command]
pub async fn save_script_version(
    script_id: String,
    change_note: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let version_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    #[derive(sqlx::FromRow)]
    struct ScriptRow {
        js_code: String,
    }

    let script = sqlx::query_as::<_, ScriptRow>("SELECT js_code FROM scripts WHERE id = ?")
        .bind(&script_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Script not found".to_string())?;

    sqlx::query("INSERT INTO script_versions (id, script_id, js_code, change_note, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&version_id).bind(&script_id).bind(&script.js_code).bind(&change_note).bind(now)
        .execute(&state.db).await.map_err(|e| e.to_string())?;

    Ok(version_id)
}

#[tauri::command]
pub async fn list_script_versions(
    script_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ScriptVersionDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        script_id: String,
        js_code: String,
        change_note: Option<String>,
        created_at: i64,
    }

    let rows = sqlx::query_as::<_, Row>("SELECT id, script_id, js_code, change_note, created_at FROM script_versions WHERE script_id = ? ORDER BY created_at DESC")
        .bind(&script_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| ScriptVersionDto {
        id: r.id,
        script_id: r.script_id,
        js_code: r.js_code,
        change_note: r.change_note,
        created_at: r.created_at,
    }).collect())
}

#[tauri::command]
pub async fn restore_script_version(
    script_id: String,
    version_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    #[derive(sqlx::FromRow)]
    struct VerRow {
        js_code: String,
    }

    let ver = sqlx::query_as::<_, VerRow>("SELECT js_code FROM script_versions WHERE id = ? AND script_id = ?")
        .bind(&version_id).bind(&script_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Script version not found".to_string())?;

    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE scripts SET js_code = ?, updated_at = ? WHERE id = ?")
        .bind(&ver.js_code).bind(now).bind(&script_id)
        .execute(&state.db).await.map_err(|e| e.to_string())?;

    Ok(())
}