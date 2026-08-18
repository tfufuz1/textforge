use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;
use crate::AppState;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "_type", rename_all = "snake_case")]
pub enum BulkOperationDto {
    BulkTransform {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        #[serde(rename = "pipelineId")]
        pipeline_id: String,
        #[serde(rename = "saveResults")]
        save_results: bool,
    },
    BulkTag {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        #[serde(rename = "addTags")]
        add_tags: Vec<String>,
        #[serde(rename = "removeTags")]
        remove_tags: Vec<String>,
    },
    BulkMove {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        #[serde(rename = "targetLocation")]
        target_location: serde_json::Value,
    },
    BulkDelete {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        permanent: bool,
    },
    BulkExport {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        format: String,
        #[serde(rename = "outputPath")]
        output_path: String,
    },
    BulkPin {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        pinned: bool,
    },
    BulkFavorite {
        #[serde(rename = "snippetIds")]
        snippet_ids: Vec<String>,
        favorite: bool,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkOperationFailedDto {
    pub id: String,
    pub error: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkOperationPreviewDto {
    pub id: String,
    pub preview: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkOperationResultDto {
    pub operation: BulkOperationDto,
    pub succeeded: Vec<String>,
    pub failed: Vec<BulkOperationFailedDto>,
    pub total_count: u32,
    pub duration_ms: u32,
    pub previews: Option<Vec<BulkOperationPreviewDto>>,
}

#[tauri::command]
pub async fn execute_bulk_operation(
    operation: BulkOperationDto,
    state: State<'_, AppState>,
) -> Result<BulkOperationResultDto, String> {
    let start = Instant::now();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    let previews = None;

    match &operation {
        BulkOperationDto::BulkDelete { snippet_ids, permanent } => {
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            for id in snippet_ids {
                let res = if *permanent {
                    sqlx::query("DELETE FROM snippets WHERE id = ?").bind(id).execute(&mut *tx).await
                } else {
                    let now = chrono::Utc::now().timestamp_millis();
                    sqlx::query("UPDATE snippets SET location_type = 'trash', deleted_at = ?, updated_at = ? WHERE id = ?")
                        .bind(now).bind(now).bind(id).execute(&mut *tx).await
                };

                match res {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkPin { snippet_ids, pinned } => {
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp_millis();
            for id in snippet_ids {
                match sqlx::query("UPDATE snippets SET is_pinned = ?, updated_at = ? WHERE id = ?")
                    .bind(if *pinned { 1 } else { 0 }).bind(now).bind(id).execute(&mut *tx).await {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkFavorite { snippet_ids, favorite } => {
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp_millis();
            for id in snippet_ids {
                match sqlx::query("UPDATE snippets SET is_favorite = ?, updated_at = ? WHERE id = ?")
                    .bind(if *favorite { 1 } else { 0 }).bind(now).bind(id).execute(&mut *tx).await {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkMove { snippet_ids, target_location } => {
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp_millis();
            let loc_type = target_location.get("_type").and_then(|v| v.as_str()).unwrap_or("inbox");
            let folder_id = target_location.get("folderId").and_then(|v| v.as_str());

            for id in snippet_ids {
                match sqlx::query("UPDATE snippets SET location_type = ?, location_folder_id = ?, deleted_at = NULL, updated_at = ? WHERE id = ?")
                    .bind(loc_type).bind(folder_id).bind(now).bind(id).execute(&mut *tx).await {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkTag { snippet_ids, add_tags, remove_tags } => {
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            
            for id in snippet_ids {
                let mut has_error = false;
                for tag in remove_tags {
                    if let Err(e) = sqlx::query("DELETE FROM snippet_tags WHERE snippet_id = ? AND tag = ?")
                        .bind(id).bind(tag).execute(&mut *tx).await {
                        has_error = true;
                        failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) });
                        break;
                    }
                }
                
                if !has_error {
                    for tag in add_tags {
                        if let Err(e) = sqlx::query("INSERT OR IGNORE INTO snippet_tags (snippet_id, tag) VALUES (?, ?)")
                            .bind(id).bind(tag).execute(&mut *tx).await {
                            has_error = true;
                            failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) });
                            break;
                        }
                    }
                }
                
                if !has_error {
                    let now = chrono::Utc::now().timestamp_millis();
                    if let Err(e) = sqlx::query("UPDATE snippets SET updated_at = ? WHERE id = ?").bind(now).bind(id).execute(&mut *tx).await {
                        failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) });
                    } else {
                        succeeded.push(id.clone());
                    }
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkTransform { snippet_ids, .. } => {
            for id in snippet_ids {
                failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "NOT_IMPLEMENTED" }) });
            }
        }
        BulkOperationDto::BulkExport { snippet_ids, .. } => {
             for id in snippet_ids {
                failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "NOT_IMPLEMENTED" }) });
            }
        }
    }

    let elapsed = start.elapsed().as_millis() as u32;
    let total_count = succeeded.len() as u32 + failed.len() as u32;

    Ok(BulkOperationResultDto {
        operation,
        succeeded,
        failed,
        total_count,
        duration_ms: elapsed,
        previews,
    })
}
