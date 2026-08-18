use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use std::fs::File;
use std::io::{Write, Read};
use zip::write::FileOptions;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequestDto {
    pub export_type: Option<String>, // "full" | "snippets" | "scripts" | "pipelines"
    pub format: String,              // "tfbundle" | "json"
    pub target_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResultDto {
    pub success: bool,
    pub exported_count: u32,
    pub file_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequestDto {
    pub source_path: String,
    pub conflict_policy: Option<String>, // "skip" | "overwrite" | "rename"
    pub overwrite: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResultDto {
    pub success: bool,
    pub snippets_imported: u32,
    pub scripts_imported: u32,
    pub pipelines_imported: u32,
    pub folders_imported: u32,
    pub skipped: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewDto {
    pub snippet_count: u32,
    pub script_count: u32,
    pub pipeline_count: u32,
    pub folder_count: u32,
    pub created_at: i64,
}

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
pub async fn export_data(
    request: ExportRequestDto,
    state: State<'_, AppState>,
) -> Result<ExportResultDto, String> {
    if request.format != "tfbundle" {
        return Err("Only tfbundle export format is supported".to_string());
    }

    let file = File::create(&request.target_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    let mut exported_count = 0;
    let mut checksums = HashMap::new();

    // 1. Export Snippets
    zip.add_directory("snippets/", options).map_err(|e| e.to_string())?;
    #[derive(Serialize, sqlx::FromRow)]
    struct SnippetExport { id: String, title: String, content: String, content_type: String, is_pinned: i64, is_favorite: i64 }
    let snippets = sqlx::query_as::<_, SnippetExport>("SELECT id, title, content, content_type, is_pinned, is_favorite FROM snippets")
        .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    for snip in &snippets {
        let json = serde_json::to_string_pretty(snip).map_err(|e| e.to_string())?;
        let bytes = json.as_bytes();
        let path = format!("snippets/{}.json", snip.id);
        checksums.insert(path.clone(), compute_sha256(bytes));
        zip.start_file(path, options).map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
        exported_count += 1;
    }

    // 2. Export Scripts
    zip.add_directory("scripts/", options).map_err(|e| e.to_string())?;
    #[derive(Serialize, sqlx::FromRow)]
    struct ScriptExport { id: String, name: String, description: Option<String>, js_code: String }
    let scripts = sqlx::query_as::<_, ScriptExport>("SELECT id, name, description, js_code FROM scripts")
        .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    for sc in &scripts {
        let json = serde_json::to_string_pretty(sc).map_err(|e| e.to_string())?;
        let bytes = json.as_bytes();
        let path = format!("scripts/{}.json", sc.id);
        checksums.insert(path.clone(), compute_sha256(bytes));
        zip.start_file(path, options).map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
        exported_count += 1;
    }

    // 3. Export Pipelines
    zip.add_directory("pipelines/", options).map_err(|e| e.to_string())?;
    #[derive(Serialize, sqlx::FromRow)]
    struct PipelineExport { id: String, name: String, description: Option<String> }
    let pipelines = sqlx::query_as::<_, PipelineExport>("SELECT id, name, description FROM pipelines")
        .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    for pipe in &pipelines {
        let json = serde_json::to_string_pretty(pipe).map_err(|e| e.to_string())?;
        let bytes = json.as_bytes();
        let path = format!("pipelines/{}.json", pipe.id);
        checksums.insert(path.clone(), compute_sha256(bytes));
        zip.start_file(path, options).map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
        exported_count += 1;
    }

    // 4. Export Folders
    zip.add_directory("folders/", options).map_err(|e| e.to_string())?;
    #[derive(Serialize, sqlx::FromRow)]
    struct FolderExport { id: String, name: String, parent_id: Option<String> }
    let folders = sqlx::query_as::<_, FolderExport>("SELECT id, name, parent_id FROM folders")
        .fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    let folders_json = serde_json::to_string_pretty(&folders).map_err(|e| e.to_string())?;
    let folders_bytes = folders_json.as_bytes();
    let folders_path = "folders/folders.json".to_string();
    checksums.insert(folders_path.clone(), compute_sha256(folders_bytes));
    zip.start_file(folders_path, options).map_err(|e| e.to_string())?;
    zip.write_all(folders_bytes).map_err(|e| e.to_string())?;

    // 5. Write manifest.json
    let now = chrono::Utc::now().timestamp_millis();
    let manifest = serde_json::json!({
        "bundleVersion": "1.0",
        "appVersion": "2.1.0",
        "bundleId": uuid::Uuid::new_v4().to_string(),
        "createdAt": now,
        "platform": "KDE Plasma 6 / Wayland",
        "counts": {
            "snippets": snippets.len(),
            "scripts": scripts.len(),
            "pipelines": pipelines.len(),
            "folders": folders.len()
        },
        "checksums": checksums
    });
    
    zip.start_file("manifest.json", options).map_err(|e| e.to_string())?;
    zip.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;

    Ok(ExportResultDto {
        success: true,
        exported_count,
        file_path: request.target_path,
    })
}

#[tauri::command]
pub async fn import_data(
    request: ImportRequestDto,
    state: State<'_, AppState>,
) -> Result<ImportResultDto, String> {
    let file = File::open(&request.source_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    // Parse manifest & checksums if present
    let mut checksum_map: HashMap<String, String> = HashMap::new();
    if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
        let mut manifest_contents = String::new();
        if manifest_file.read_to_string(&mut manifest_contents).is_ok() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&manifest_contents) {
                if let Some(c_obj) = val.get("checksums").and_then(|v| v.as_object()) {
                    for (k, v) in c_obj {
                        if let Some(chk) = v.as_str() {
                            checksum_map.insert(k.clone(), chk.to_string());
                        }
                    }
                }
            }
        }
    }

    let policy = request.conflict_policy.as_deref().unwrap_or(
        if request.overwrite.unwrap_or(false) { "overwrite" } else { "skip" }
    );

    let mut snippets_imported = 0;
    let mut scripts_imported = 0;
    let mut pipelines_imported = 0;
    let mut folders_imported = 0;
    let mut skipped = 0;

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = zip_file.name().to_string();
        
        if zip_file.is_dir() { continue; }

        let mut raw_bytes = Vec::new();
        zip_file.read_to_end(&mut raw_bytes).map_err(|e| e.to_string())?;

        // Checksum validation if present in manifest
        if let Some(expected_chk) = checksum_map.get(&name) {
            let actual_chk = compute_sha256(&raw_bytes);
            if actual_chk != *expected_chk {
                return Err(format!("Checksum verification failed for {}", name));
            }
        }

        let contents = match String::from_utf8(raw_bytes) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if name.starts_with("snippets/") && name.ends_with(".json") {
            let val: serde_json::Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
            if let (Some(id), Some(title), Some(content)) = (val.get("id"), val.get("title"), val.get("content")) {
                let mut id_str = id.as_str().unwrap_or_default().to_string();
                let mut title_str = title.as_str().unwrap_or_default().to_string();
                let content_str = content.as_str().unwrap_or_default();
                let content_type = val.get("contentType").and_then(|v| v.as_str()).unwrap_or("text");

                if policy == "rename" {
                    title_str = format!("{} (Import)", title_str);
                    id_str = uuid::Uuid::new_v4().to_string();
                }

                let q = match policy {
                    "overwrite" => "INSERT OR REPLACE INTO snippets (id, title, content, content_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                    _ => "INSERT OR IGNORE INTO snippets (id, title, content, content_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                };

                let res = sqlx::query(q)
                    .bind(&id_str).bind(&title_str).bind(content_str).bind(content_type).bind(now).bind(now)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

                if res.rows_affected() > 0 {
                    snippets_imported += 1;
                } else {
                    skipped += 1;
                }
            }
        } else if name.starts_with("scripts/") && name.ends_with(".json") {
            let val: serde_json::Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
            if let (Some(id), Some(name_val), Some(js_code)) = (val.get("id"), val.get("name"), val.get("jsCode")) {
                let id_str = id.as_str().unwrap_or_default();
                let name_str = name_val.as_str().unwrap_or_default();
                let code_str = js_code.as_str().unwrap_or_default();
                let desc_str = val.get("description").and_then(|v| v.as_str());

                let q = match policy {
                    "overwrite" => "INSERT OR REPLACE INTO scripts (id, name, js_code, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                    _ => "INSERT OR IGNORE INTO scripts (id, name, js_code, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                };

                let res = sqlx::query(q)
                    .bind(id_str).bind(name_str).bind(code_str).bind(desc_str).bind(now).bind(now)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

                if res.rows_affected() > 0 {
                    scripts_imported += 1;
                } else {
                    skipped += 1;
                }
            }
        } else if name.starts_with("pipelines/") && name.ends_with(".json") {
            let val: serde_json::Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
            if let (Some(id), Some(name_val)) = (val.get("id"), val.get("name")) {
                let id_str = id.as_str().unwrap_or_default();
                let name_str = name_val.as_str().unwrap_or_default();
                let desc_str = val.get("description").and_then(|v| v.as_str());

                let q = match policy {
                    "overwrite" => "INSERT OR REPLACE INTO pipelines (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                    _ => "INSERT OR IGNORE INTO pipelines (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                };

                let res = sqlx::query(q)
                    .bind(id_str).bind(name_str).bind(desc_str).bind(now).bind(now)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

                if res.rows_affected() > 0 {
                    pipelines_imported += 1;
                } else {
                    skipped += 1;
                }
            }
        } else if name == "folders/folders.json" {
            let folders_arr: Vec<serde_json::Value> = serde_json::from_str(&contents).unwrap_or_default();
            for f_val in folders_arr {
                if let (Some(id), Some(name_val)) = (f_val.get("id"), f_val.get("name")) {
                    let id_str = id.as_str().unwrap_or_default();
                    let name_str = name_val.as_str().unwrap_or_default();
                    let parent_id = f_val.get("parentId").and_then(|v| v.as_str());

                    let q = match policy {
                        "overwrite" => "INSERT OR REPLACE INTO folders (id, name, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                        _ => "INSERT OR IGNORE INTO folders (id, name, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                    };

                    if let Ok(res) = sqlx::query(q).bind(id_str).bind(name_str).bind(parent_id).bind(now).bind(now).execute(&mut *tx).await {
                        if res.rows_affected() > 0 {
                            folders_imported += 1;
                        }
                    }
                }
            }
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(ImportResultDto {
        success: true,
        snippets_imported,
        scripts_imported,
        pipelines_imported,
        folders_imported,
        skipped,
    })
}

#[tauri::command]
pub async fn preview_import(
    source_path: String,
    _state: State<'_, AppState>,
) -> Result<ImportPreviewDto, String> {
    let file = File::open(&source_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    let mut snippet_count = 0;
    let mut script_count = 0;
    let mut pipeline_count = 0;
    let mut folder_count = 0;
    let mut created_at = chrono::Utc::now().timestamp_millis();
    
    if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
        let mut manifest_contents = String::new();
        if manifest_file.read_to_string(&mut manifest_contents).is_ok() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&manifest_contents) {
                if let Some(ca) = val.get("createdAt").and_then(|v| v.as_i64()) {
                    created_at = ca;
                }
                if let Some(counts) = val.get("counts") {
                    snippet_count = counts.get("snippets").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    script_count = counts.get("scripts").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    pipeline_count = counts.get("pipelines").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    folder_count = counts.get("folders").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    return Ok(ImportPreviewDto {
                        snippet_count,
                        script_count,
                        pipeline_count,
                        folder_count,
                        created_at,
                    });
                }
            }
        }
    }

    // Fallback if manifest is missing or basic
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if name.starts_with("snippets/") && name.ends_with(".json") {
                snippet_count += 1;
            } else if name.starts_with("scripts/") && name.ends_with(".json") {
                script_count += 1;
            } else if name.starts_with("pipelines/") && name.ends_with(".json") {
                pipeline_count += 1;
            } else if name == "folders/folders.json" {
                folder_count += 1;
            }
        }
    }

    Ok(ImportPreviewDto {
        snippet_count,
        script_count,
        pipeline_count,
        folder_count,
        created_at,
    })
}
