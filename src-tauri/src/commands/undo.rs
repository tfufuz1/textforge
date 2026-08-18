use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "_type", rename_all = "snake_case")]
pub enum UndoActionDto {
    SnippetUpdate { before: serde_json::Value, after: serde_json::Value },
    SnippetCreate { created: serde_json::Value },
    SnippetDelete { deleted: serde_json::Value },
    SnippetMove { id: String, from: serde_json::Value, to: serde_json::Value },
    
    ScriptUpdate { before: serde_json::Value, after: serde_json::Value },
    ScriptCreate { created: serde_json::Value },
    ScriptDelete { deleted: serde_json::Value },
    
    PipelineUpdate { before: serde_json::Value, after: serde_json::Value },
    
    TransformApply {
        #[serde(rename = "snippetId")]
        snippet_id: String,
        #[serde(rename = "originalContent")]
        original_content: String,
        #[serde(rename = "transformedContent")]
        transformed_content: String,
        #[serde(rename = "pipelineId")]
        pipeline_id: Option<String>,
        #[serde(rename = "scriptId")]
        script_id: Option<String>,
    },
    
    BulkOperation {
        operations: Vec<UndoActionDto>,
    },
    
    FolderCreate { created: serde_json::Value },
    FolderRename { id: String, from: String, to: String },
    FolderDelete {
        deleted: serde_json::Value,
        #[serde(rename = "movedSnippets")]
        moved_snippets: Vec<String>
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UndoEntryDto {
    pub id: String,
    pub performed_at: i64,
    pub description: String,
    pub action: UndoActionDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoStateDto {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_count: u32,
    pub redo_count: u32,
    pub top_undo_description: Option<String>,
    pub top_redo_description: Option<String>,
}

pub struct UndoStack {
    pub undo_history: Vec<UndoEntryDto>,
    pub redo_history: Vec<UndoEntryDto>,
    pub max_size: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            max_size: 50,
        }
    }

    pub fn push(&mut self, entry: UndoEntryDto) {
        self.undo_history.push(entry);
        if self.undo_history.len() > self.max_size {
            self.undo_history.remove(0);
        }
        self.redo_history.clear();
    }
}

pub type SharedUndoStack = Mutex<UndoStack>;

// Helper for applying actions to the DB
async fn apply_action(action: &UndoActionDto, is_undo: bool, db: &sqlx::SqlitePool) -> Result<(), String> {
    let mut tx = db.begin().await.map_err(|e| e.to_string())?;
    execute_action_recursive(action, is_undo, &mut tx).await?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn execute_action_recursive(action: &UndoActionDto, is_undo: bool, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<(), String> {
    match action {
        UndoActionDto::SnippetUpdate { before, after } => {
            let state = if is_undo { before } else { after };
            let id = state.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let content = state.get("content").and_then(|v| v.as_str()).unwrap_or_default();
            let title = state.get("title").and_then(|v| v.as_str()).unwrap_or_default();
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE snippets SET title = ?, content = ?, updated_at = ? WHERE id = ?")
                .bind(title).bind(content).bind(now).bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        UndoActionDto::SnippetCreate { created } => {
            let id = created.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if is_undo {
                sqlx::query("DELETE FROM snippets WHERE id = ?").bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            } else {
                let title = created.get("title").and_then(|v| v.as_str()).unwrap_or_default();
                let content = created.get("content").and_then(|v| v.as_str()).unwrap_or_default();
                let content_type = created.get("contentType").and_then(|v| v.as_str()).unwrap_or("text");
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("INSERT OR REPLACE INTO snippets (id, title, content, content_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(id).bind(title).bind(content).bind(content_type).bind(now).bind(now).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        UndoActionDto::SnippetDelete { deleted } => {
            let id = deleted.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if is_undo {
                let title = deleted.get("title").and_then(|v| v.as_str()).unwrap_or_default();
                let content = deleted.get("content").and_then(|v| v.as_str()).unwrap_or_default();
                let content_type = deleted.get("contentType").and_then(|v| v.as_str()).unwrap_or("text");
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("INSERT OR REPLACE INTO snippets (id, title, content, content_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(id).bind(title).bind(content).bind(content_type).bind(now).bind(now).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            } else {
                sqlx::query("DELETE FROM snippets WHERE id = ?").bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        UndoActionDto::SnippetMove { id, from, to } => {
            let loc = if is_undo { from } else { to };
            let loc_type = loc.get("_type").and_then(|v| v.as_str()).unwrap_or("inbox");
            let folder_id = loc.get("folderId").and_then(|v| v.as_str());
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE snippets SET location_type = ?, location_folder_id = ?, updated_at = ? WHERE id = ?")
                .bind(loc_type).bind(folder_id).bind(now).bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        UndoActionDto::ScriptUpdate { before, after } => {
            let state = if is_undo { before } else { after };
            let id = state.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let name = state.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let js_code = state.get("jsCode").and_then(|v| v.as_str()).unwrap_or_default();
            let description = state.get("description").and_then(|v| v.as_str()).unwrap_or_default();
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE scripts SET name = ?, js_code = ?, description = ?, updated_at = ? WHERE id = ?")
                .bind(name).bind(js_code).bind(description).bind(now).bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        UndoActionDto::ScriptCreate { created } => {
            let id = created.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if is_undo {
                sqlx::query("DELETE FROM scripts WHERE id = ?").bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            } else {
                let name = created.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let js_code = created.get("jsCode").and_then(|v| v.as_str()).unwrap_or_default();
                let description = created.get("description").and_then(|v| v.as_str()).unwrap_or_default();
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("INSERT OR REPLACE INTO scripts (id, name, js_code, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(id).bind(name).bind(js_code).bind(description).bind(now).bind(now).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        UndoActionDto::ScriptDelete { deleted } => {
            let id = deleted.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if is_undo {
                let name = deleted.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let js_code = deleted.get("jsCode").and_then(|v| v.as_str()).unwrap_or_default();
                let description = deleted.get("description").and_then(|v| v.as_str()).unwrap_or_default();
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("INSERT OR REPLACE INTO scripts (id, name, js_code, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(id).bind(name).bind(js_code).bind(description).bind(now).bind(now).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            } else {
                sqlx::query("DELETE FROM scripts WHERE id = ?").bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        UndoActionDto::PipelineUpdate { before, after } => {
            let state = if is_undo { before } else { after };
            let id = state.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let name = state.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let description = state.get("description").and_then(|v| v.as_str()).unwrap_or_default();
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE pipelines SET name = ?, description = ?, updated_at = ? WHERE id = ?")
                .bind(name).bind(description).bind(now).bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        UndoActionDto::TransformApply { snippet_id, original_content, transformed_content, .. } => {
            let content = if is_undo { original_content } else { transformed_content };
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE snippets SET content = ?, updated_at = ? WHERE id = ?")
                .bind(content).bind(now).bind(snippet_id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        UndoActionDto::BulkOperation { operations } => {
            if is_undo {
                for op in operations.iter().rev() {
                    Box::pin(execute_action_recursive(op, is_undo, tx)).await?;
                }
            } else {
                for op in operations.iter() {
                    Box::pin(execute_action_recursive(op, is_undo, tx)).await?;
                }
            }
        }
        UndoActionDto::FolderCreate { created } => {
            let id = created.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if is_undo {
                sqlx::query("DELETE FROM folders WHERE id = ?").bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            } else {
                let name = created.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let parent_id = created.get("parentId").and_then(|v| v.as_str());
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("INSERT OR REPLACE INTO folders (id, name, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
                    .bind(id).bind(name).bind(parent_id).bind(now).bind(now).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        UndoActionDto::FolderRename { id, from, to } => {
            let name = if is_undo { from } else { to };
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query("UPDATE folders SET name = ?, updated_at = ? WHERE id = ?")
                .bind(name).bind(now).bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        UndoActionDto::FolderDelete { deleted, moved_snippets } => {
            let id = deleted.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if is_undo {
                let name = deleted.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let parent_id = deleted.get("parentId").and_then(|v| v.as_str());
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("INSERT OR REPLACE INTO folders (id, name, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
                    .bind(id).bind(name).bind(parent_id).bind(now).bind(now).execute(&mut **tx).await.map_err(|e| e.to_string())?;
                for snip_id in moved_snippets {
                    sqlx::query("UPDATE snippets SET location_folder_id = ?, updated_at = ? WHERE id = ?")
                        .bind(id).bind(now).bind(snip_id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
                }
            } else {
                let now = chrono::Utc::now().timestamp_millis();
                sqlx::query("UPDATE snippets SET location_type = 'inbox', location_folder_id = NULL, updated_at = ? WHERE location_folder_id = ?")
                    .bind(now).bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
                sqlx::query("DELETE FROM folders WHERE id = ?").bind(id).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn undo(
    state: State<'_, AppState>,
) -> Result<UndoEntryDto, String> {
    let entry = {
        let mut stack = state.undo_stack.lock().map_err(|e| e.to_string())?;
        stack.undo_history.pop().ok_or_else(|| "Nothing to undo".to_string())?
    };

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
    execute_action_recursive(&entry.action, true, &mut tx).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    {
        let mut stack = state.undo_stack.lock().map_err(|e| e.to_string())?;
        stack.redo_history.push(entry.clone());
    }

    Ok(entry)
}

#[tauri::command]
pub async fn redo(
    state: State<'_, AppState>,
) -> Result<UndoEntryDto, String> {
    let entry = {
        let mut stack = state.undo_stack.lock().map_err(|e| e.to_string())?;
        stack.redo_history.pop().ok_or_else(|| "Nothing to redo".to_string())?
    };

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
    execute_action_recursive(&entry.action, false, &mut tx).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    {
        let mut stack = state.undo_stack.lock().map_err(|e| e.to_string())?;
        stack.undo_history.push(entry.clone());
    }

    Ok(entry)
}

#[tauri::command]
pub async fn get_undo_state(
    state: State<'_, AppState>,
) -> Result<UndoStateDto, String> {
    let stack = state.undo_stack.lock().map_err(|e| e.to_string())?;
    let can_undo = !stack.undo_history.is_empty();
    let can_redo = !stack.redo_history.is_empty();
    let top_undo_description = stack.undo_history.last().map(|e| e.description.clone());
    let top_redo_description = stack.redo_history.last().map(|e| e.description.clone());

    Ok(UndoStateDto {
        can_undo,
        can_redo,
        undo_count: stack.undo_history.len() as u32,
        redo_count: stack.redo_history.len() as u32,
        top_undo_description,
        top_redo_description,
    })
}

