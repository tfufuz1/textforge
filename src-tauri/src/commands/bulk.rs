use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};
use crate::AppState;
use crate::commands::undo::{UndoActionDto, UndoEntryDto};

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

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BulkProgressPayload {
    pub completed: usize,
    pub total: usize,
    pub current_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
pub async fn execute_bulk_operation<R: tauri::Runtime>(
    app: AppHandle<R>,
    operation: BulkOperationDto,
    state: State<'_, AppState>,
) -> Result<BulkOperationResultDto, String> {
    let start = Instant::now();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    let mut previews = None;

    match &operation {
        BulkOperationDto::BulkDelete { snippet_ids, permanent } => {
            let total = snippet_ids.len();
            let mut undo_actions = Vec::new();
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

            for (idx, id) in snippet_ids.iter().enumerate() {
                app.emit("bulk:progress", BulkProgressPayload {
                    completed: idx + 1,
                    total,
                    current_id: id.clone(),
                }).ok();

                #[derive(sqlx::FromRow)]
                struct SnipRow { id: String, title: String, content: String, content_type: String }
                let fetched = sqlx::query_as::<_, SnipRow>("SELECT id, title, content, content_type FROM snippets WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&mut *tx)
                    .await
                    .ok()
                    .flatten();

                let res = if *permanent {
                    sqlx::query("DELETE FROM snippets WHERE id = ?").bind(id).execute(&mut *tx).await
                } else {
                    let now = chrono::Utc::now().timestamp_millis();
                    sqlx::query("UPDATE snippets SET location_type = 'trash', deleted_at = ?, updated_at = ? WHERE id = ?")
                        .bind(now).bind(now).bind(id).execute(&mut *tx).await
                };

                match res {
                    Ok(_) => {
                        succeeded.push(id.clone());
                        if let Some(snip) = fetched {
                            undo_actions.push(UndoActionDto::SnippetDelete {
                                deleted: serde_json::json!({
                                    "id": snip.id,
                                    "title": snip.title,
                                    "content": snip.content,
                                    "contentType": snip.content_type
                                })
                            });
                        }
                    }
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;

            if !undo_actions.is_empty() {
                let now = chrono::Utc::now().timestamp_millis();
                let undo_entry = UndoEntryDto {
                    id: uuid::Uuid::new_v4().to_string(),
                    performed_at: now,
                    description: format!("Bulk Delete auf {} Snippets angewendet", undo_actions.len()),
                    action: UndoActionDto::BulkOperation { operations: undo_actions },
                };
                if let Ok(mut stack) = state.undo_stack.lock() {
                    stack.push(undo_entry);
                }
            }
        }
        BulkOperationDto::BulkPin { snippet_ids, pinned } => {
            let total = snippet_ids.len();
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp_millis();
            for (idx, id) in snippet_ids.iter().enumerate() {
                app.emit("bulk:progress", BulkProgressPayload {
                    completed: idx + 1,
                    total,
                    current_id: id.clone(),
                }).ok();
                match sqlx::query("UPDATE snippets SET is_pinned = ?, updated_at = ? WHERE id = ?")
                    .bind(if *pinned { 1 } else { 0 }).bind(now).bind(id).execute(&mut *tx).await {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkFavorite { snippet_ids, favorite } => {
            let total = snippet_ids.len();
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp_millis();
            let fav_val = if *favorite { 1 } else { 0 };
            for (idx, id) in snippet_ids.iter().enumerate() {
                app.emit("bulk:progress", BulkProgressPayload {
                    completed: idx + 1,
                    total,
                    current_id: id.clone(),
                }).ok();
                match sqlx::query("UPDATE snippets SET is_favorite = ?, updated_at = ? WHERE id = ?")
                    .bind(fav_val).bind(now).bind(id).execute(&mut *tx).await {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkMove { snippet_ids, target_location } => {
            let total = snippet_ids.len();
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp_millis();
            let loc_type = target_location.get("_type").and_then(|v| v.as_str()).unwrap_or("inbox");
            let folder_id = target_location.get("folderId").and_then(|v| v.as_str());

            for (idx, id) in snippet_ids.iter().enumerate() {
                app.emit("bulk:progress", BulkProgressPayload {
                    completed: idx + 1,
                    total,
                    current_id: id.clone(),
                }).ok();
                match sqlx::query("UPDATE snippets SET location_type = ?, location_folder_id = ?, deleted_at = NULL, updated_at = ? WHERE id = ?")
                    .bind(loc_type).bind(folder_id).bind(now).bind(id).execute(&mut *tx).await {
                    Ok(_) => succeeded.push(id.clone()),
                    Err(e) => failed.push(BulkOperationFailedDto { id: id.clone(), error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }) }),
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;
        }
        BulkOperationDto::BulkTag { snippet_ids, add_tags, remove_tags } => {
            let total = snippet_ids.len();
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
            
            for (idx, id) in snippet_ids.iter().enumerate() {
                app.emit("bulk:progress", BulkProgressPayload {
                    completed: idx + 1,
                    total,
                    current_id: id.clone(),
                }).ok();
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
        BulkOperationDto::BulkTransform { snippet_ids, pipeline_id, save_results } => {
            let total = snippet_ids.len();
            let mut preview_list = Vec::new();
            let mut undo_actions = Vec::new();

            // Load pipeline & steps ONCE
            let pipeline_exists = sqlx::query("SELECT 1 FROM pipelines WHERE id = ?")
                .bind(pipeline_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| e.to_string())?;

            if pipeline_exists.is_none() {
                return Err(format!("Pipeline not found: {}", pipeline_id));
            }

            let steps = sqlx::query_as::<_, crate::commands::transform::PipelineStepRow>(
                "SELECT id, label, script_id, enabled, failure_policy, condition_json FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC"
            )
            .bind(pipeline_id)
            .fetch_all(&state.db)
            .await
            .map_err(|e| e.to_string())?;

            if *save_results {
                let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
                let mut storage_error = None;

                for (idx, id) in snippet_ids.iter().enumerate() {
                    app.emit("bulk:progress", BulkProgressPayload {
                        completed: idx + 1,
                        total,
                        current_id: id.clone(),
                    }).ok();

                    #[derive(sqlx::FromRow)]
                    struct SnipRow { content: String }

                    let snip = match sqlx::query_as::<_, SnipRow>("SELECT content FROM snippets WHERE id = ?")
                        .bind(id)
                        .fetch_optional(&mut *tx)
                        .await
                    {
                        Ok(Some(s)) => s,
                        Ok(None) => {
                            failed.push(BulkOperationFailedDto {
                                id: id.clone(),
                                error: serde_json::json!({ "code": "SNIPPET_NOT_FOUND" }),
                            });
                            continue;
                        }
                        Err(e) => {
                            storage_error = Some(e.to_string());
                            failed.push(BulkOperationFailedDto {
                                id: id.clone(),
                                error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }),
                            });
                            break;
                        }
                    };

                    match crate::commands::transform::execute_pipeline_with_steps(
                        &steps,
                        snip.content.clone(),
                        state,
                    )
                    .await
                    {
                        Ok(res) => {
                            if !res.is_success {
                                let err_msg = res.step_results
                                    .iter()
                                    .find_map(|s| s.error.clone())
                                    .unwrap_or_else(|| "Pipeline execution failed".to_string());
                                failed.push(BulkOperationFailedDto {
                                    id: id.clone(),
                                    error: serde_json::json!({ "code": "PIPELINE_ERROR", "details": err_msg }),
                                });
                                continue;
                            }

                            let now = chrono::Utc::now().timestamp_millis();
                            let update_res = sqlx::query("UPDATE snippets SET content = ?, updated_at = ? WHERE id = ?")
                                .bind(&res.final_output)
                                .bind(now)
                                .bind(id)
                                .execute(&mut *tx)
                                .await;

                            match update_res {
                                Ok(_) => {
                                    succeeded.push(id.clone());
                                    undo_actions.push(crate::commands::undo::UndoActionDto::TransformApply {
                                        snippet_id: id.clone(),
                                        original_content: snip.content,
                                        transformed_content: res.final_output,
                                        pipeline_id: Some(pipeline_id.clone()),
                                        script_id: None,
                                    });
                                }
                                Err(e) => {
                                    storage_error = Some(e.to_string());
                                    failed.push(BulkOperationFailedDto {
                                        id: id.clone(),
                                        error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }),
                                    });
                                    break;
                                }
                            }
                        }
                        Err(err_msg) => {
                            failed.push(BulkOperationFailedDto {
                                id: id.clone(),
                                error: serde_json::json!({ "code": "PIPELINE_ERROR", "details": err_msg }),
                            });
                        }
                    }
                }

                if storage_error.is_some() {
                    tx.rollback().await.map_err(|e| e.to_string())?;
                } else {
                    tx.commit().await.map_err(|e| e.to_string())?;

                    if !undo_actions.is_empty() {
                        let now = chrono::Utc::now().timestamp_millis();
                        let undo_entry = crate::commands::undo::UndoEntryDto {
                            id: uuid::Uuid::new_v4().to_string(),
                            performed_at: now,
                            description: format!("Bulk Transformation auf {} Snippets angewendet", undo_actions.len()),
                            action: crate::commands::undo::UndoActionDto::BulkOperation {
                                operations: undo_actions,
                            },
                        };
                        if let Ok(mut stack) = state.undo_stack.lock() {
                            stack.push(undo_entry);
                        }
                    }
                }
            } else {
                for (idx, id) in snippet_ids.iter().enumerate() {
                    app.emit("bulk:progress", BulkProgressPayload {
                        completed: idx + 1,
                        total,
                        current_id: id.clone(),
                    }).ok();

                    #[derive(sqlx::FromRow)]
                    struct SnipRow { content: String }

                    let snip = match sqlx::query_as::<_, SnipRow>("SELECT content FROM snippets WHERE id = ?")
                        .bind(id)
                        .fetch_optional(&state.db)
                        .await
                    {
                        Ok(Some(s)) => s,
                        Ok(None) => {
                            failed.push(BulkOperationFailedDto {
                                id: id.clone(),
                                error: serde_json::json!({ "code": "SNIPPET_NOT_FOUND" }),
                            });
                            continue;
                        }
                        Err(e) => {
                            failed.push(BulkOperationFailedDto {
                                id: id.clone(),
                                error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }),
                            });
                            continue;
                        }
                    };

                    match crate::commands::transform::execute_pipeline_with_steps(
                        &steps,
                        snip.content.clone(),
                        state,
                    )
                    .await
                    {
                        Ok(res) => {
                            if !res.is_success {
                                let err_msg = res.step_results
                                    .iter()
                                    .find_map(|s| s.error.clone())
                                    .unwrap_or_else(|| "Pipeline execution failed".to_string());
                                failed.push(BulkOperationFailedDto {
                                    id: id.clone(),
                                    error: serde_json::json!({ "code": "PIPELINE_ERROR", "details": err_msg }),
                                });
                                continue;
                            }

                            succeeded.push(id.clone());
                            preview_list.push(BulkOperationPreviewDto {
                                id: id.clone(),
                                preview: res.final_output,
                            });
                        }
                        Err(err_msg) => {
                            failed.push(BulkOperationFailedDto {
                                id: id.clone(),
                                error: serde_json::json!({ "code": "PIPELINE_ERROR", "details": err_msg }),
                            });
                        }
                    }
                }

                if !preview_list.is_empty() {
                    previews = Some(preview_list);
                }
            }
        }
        BulkOperationDto::BulkExport { snippet_ids, format, output_path } => {
            let total = snippet_ids.len();
            #[derive(Serialize, sqlx::FromRow)]
            struct SnipRow { id: String, title: String, content: String, content_type: String, is_pinned: i64, is_favorite: i64 }

            let mut fetched_snippets = Vec::new();
            let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

            for (idx, id) in snippet_ids.iter().enumerate() {
                app.emit("bulk:progress", BulkProgressPayload {
                    completed: idx + 1,
                    total,
                    current_id: id.clone(),
                }).ok();

                match sqlx::query_as::<_, SnipRow>("SELECT id, title, content, content_type, is_pinned, is_favorite FROM snippets WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&mut *tx)
                    .await
                {
                    Ok(Some(s)) => {
                        succeeded.push(id.clone());
                        fetched_snippets.push(s);
                    }
                    Ok(None) => {
                        failed.push(BulkOperationFailedDto {
                            id: id.clone(),
                            error: serde_json::json!({ "code": "SNIPPET_NOT_FOUND" }),
                        });
                    }
                    Err(e) => {
                        failed.push(BulkOperationFailedDto {
                            id: id.clone(),
                            error: serde_json::json!({ "code": "STORAGE_ERROR", "details": e.to_string() }),
                        });
                    }
                }
            }
            tx.commit().await.map_err(|e| e.to_string())?;

            let export_res = match format.to_lowercase().as_str() {
                "json" | "json_array" => {
                    let json_data = serde_json::to_string_pretty(&fetched_snippets).unwrap_or_default();
                    std::fs::write(output_path, json_data)
                }
                "text" | "markdown" => {
                    let combined = fetched_snippets
                        .iter()
                        .map(|s| format!("# {}\n\n{}", s.title, s.content))
                        .collect::<Vec<_>>()
                        .join("\n\n---\n\n");
                    std::fs::write(output_path, combined)
                }
                "csv" => {
                    let mut csv = String::from("id,title,content_type,content\n");
                    for s in &fetched_snippets {
                        let safe_title = s.title.replace('"', "\"\"");
                        let safe_content = s.content.replace('"', "\"\"");
                        csv.push_str(&format!(
                            "\"{}\",\"{}\",\"{}\",\"{}\"\n",
                            s.id, safe_title, s.content_type, safe_content
                        ));
                    }
                    std::fs::write(output_path, csv)
                }
                "bundle" | "tfbundle" => {
                    use std::fs::File;
                    use std::io::Write;
                    use zip::write::FileOptions;
                    use sha2::{Sha256, Digest};

                    let write_bundle = || -> Result<(), String> {
                        let file = File::create(output_path).map_err(|e| e.to_string())?;
                        let mut zip = zip::ZipWriter::new(file);
                        let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);
                        let mut checksums = std::collections::HashMap::new();

                        zip.add_directory("snippets/", options).map_err(|e| e.to_string())?;
                        for snip in &fetched_snippets {
                            let json = serde_json::to_string_pretty(snip).map_err(|e| e.to_string())?;
                            let bytes = json.as_bytes();
                            let path = format!("snippets/{}.json", snip.id);
                            let mut hasher = Sha256::new();
                            hasher.update(bytes);
                            let hash = format!("{:x}", hasher.finalize());
                            checksums.insert(path.clone(), hash);

                            zip.start_file(path, options).map_err(|e| e.to_string())?;
                            zip.write_all(bytes).map_err(|e| e.to_string())?;
                        }

                        let now = chrono::Utc::now().timestamp_millis();
                        let manifest = serde_json::json!({
                            "bundleVersion": "1.0",
                            "appVersion": "2.1.0",
                            "bundleId": uuid::Uuid::new_v4().to_string(),
                            "createdAt": now,
                            "platform": "TextForge Bulk Export",
                            "counts": {
                                "snippets": fetched_snippets.len(),
                                "scripts": 0,
                                "pipelines": 0,
                                "folders": 0
                            },
                            "checksums": checksums
                        });

                        zip.start_file("manifest.json", options).map_err(|e| e.to_string())?;
                        zip.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes()).map_err(|e| e.to_string())?;
                        zip.finish().map_err(|e| e.to_string())?;
                        Ok(())
                    };
                    write_bundle().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                }
                _ => {
                    let json_data = serde_json::to_string_pretty(&fetched_snippets).unwrap_or_default();
                    std::fs::write(output_path, json_data)
                }
            };

            if let Err(e) = export_res {
                for id in succeeded.clone() {
                    failed.push(BulkOperationFailedDto {
                        id,
                        error: serde_json::json!({ "code": "EXPORT_WRITE_ERROR", "details": e.to_string() }),
                    });
                }
                succeeded.clear();
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
