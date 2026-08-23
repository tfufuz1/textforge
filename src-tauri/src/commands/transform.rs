use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::sandbox::{run_script_in_sandbox, ScriptExecutionResultDto};
use crate::commands::builtins;

/// Timeout for regex execution (2000 ms limit).
const REGEX_TIMEOUT: Duration = Duration::from_millis(2000);

/// Maximum input size for regex execution (2 MB as per spec).
const MAX_REGEX_INPUT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteScriptDto {
    pub script_id: Option<String>,
    pub js_code: Option<String>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,
    pub regex_flags: Option<String>,
    pub input: String,
    pub params_json: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStepResultDto {
    pub step_id: String,
    pub step_label: String,
    pub output: String,
    pub execution_time_ms: u32,
    pub error: Option<String>,
    pub was_skipped: bool,
    pub condition_result: Option<bool>,
    pub failure_policy: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineExecutionResultDto {
    pub final_output: String,
    pub step_results: Vec<PipelineStepResultDto>,
    pub total_time_ms: u32,
    pub is_success: bool,
    pub skipped_steps: Vec<String>,
}

#[tauri::command]
pub async fn execute_script(
    req: ExecuteScriptDto,
    state: State<'_, AppState>,
) -> Result<ScriptExecutionResultDto, String> {
    let start = std::time::Instant::now();

    if let Some(id) = req.script_id {
        #[derive(sqlx::FromRow)]
        struct ScriptRow {
            script_type: String,
            js_code: Option<String>,
            regex_pattern: Option<String>,
            regex_replacement: Option<String>,
            regex_flags: String,
            parameters_json: String,
        }

        let row = sqlx::query_as::<_, ScriptRow>(
            "SELECT script_type, js_code, regex_pattern, regex_replacement, regex_flags, parameters_json FROM scripts WHERE id = ?"
        )
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Script not found".to_string())?;

        if row.script_type == "regex" {
            let pattern = row.regex_pattern.unwrap_or_default();
            let replacement = row.regex_replacement.unwrap_or_default();
            let flags = row.regex_flags;
            let params_str = req.params_json.unwrap_or(row.parameters_json);
            return Ok(run_regex_transformation(&req.input, &pattern, &replacement, &flags, Some(&params_str), Some(&state.regex_cache), start).await);
        } else {
            let js_code = row.js_code.ok_or_else(|| "No JS code found".to_string())?;
            let params = req.params_json.unwrap_or(row.parameters_json);
            return Ok(run_script_in_sandbox(js_code, req.input, Some(params)).await);
        }
    }

    if let Some(pattern) = req.regex_pattern {
        let replacement = req.regex_replacement.unwrap_or_default();
        let flags = req.regex_flags.unwrap_or_else(|| "g".to_string());
        return Ok(run_regex_transformation(&req.input, &pattern, &replacement, &flags, req.params_json.as_deref(), Some(&state.regex_cache), start).await);
    }

    let js_code = req.js_code.ok_or_else(|| "Neither scriptId, regexPattern nor jsCode provided".to_string())?;
    Ok(run_script_in_sandbox(js_code, req.input, req.params_json).await)
}

fn substitute_params(text: &str, params_json: Option<&str>) -> String {
    let mut result = text.to_string();
    if let Some(pj) = params_json {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(pj) {
            if let Some(obj) = val.as_object() {
                for (k, v) in obj {
                    let placeholder = format!("{{{{{}}}}}", k);
                    let val_str = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    result = result.replace(&placeholder, &val_str);
                }
            }
        }
    }
    result
}

pub async fn run_regex_transformation(
    input: &str,
    pattern: &str,
    replacement: &str,
    flags: &str,
    params_json: Option<&str>,
    regex_cache: Option<&Mutex<LruCache<(String, String), regex::Regex>>>,
    start: std::time::Instant,
) -> ScriptExecutionResultDto {
    // Input size check before compiling or running regex
    if input.len() > MAX_REGEX_INPUT_BYTES {
        let elapsed = start.elapsed().as_millis() as u32;
        return ScriptExecutionResultDto {
            output: input.to_string(),
            execution_time_ms: elapsed,
            console_logs: vec![],
            error: Some(format!(
                "Input size ({} bytes) exceeds maximum limit ({} bytes)",
                input.len(),
                MAX_REGEX_INPUT_BYTES
            )),
        };
    }

    let sub_pattern = substitute_params(pattern, params_json);
    let sub_replacement = substitute_params(replacement, params_json);

    // Form cache key AFTER parameter substitution
    let cache_key = (sub_pattern.clone(), flags.to_string());

    let cached_re = if let Some(cache_mutex) = regex_cache {
        let mut cache = cache_mutex.lock().unwrap();
        cache.get(&cache_key).cloned()
    } else {
        None
    };

    let re = match cached_re {
        Some(re) => re,
        None => {
            let mut builder = regex::RegexBuilder::new(&sub_pattern);
            if flags.contains('i') {
                builder.case_insensitive(true);
            }
            if flags.contains('m') {
                builder.multi_line(true);
            }
            if flags.contains('s') {
                builder.dot_matches_new_line(true);
            }

            match builder.build() {
                Ok(compiled) => {
                    if let Some(cache_mutex) = regex_cache {
                        let mut cache = cache_mutex.lock().unwrap();
                        cache.put(cache_key, compiled.clone());
                    }
                    compiled
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u32;
                    return ScriptExecutionResultDto {
                        output: input.to_string(),
                        execution_time_ms: elapsed,
                        console_logs: vec![],
                        error: Some(format!("Invalid RegEx pattern: {}", e)),
                    };
                }
            }
        }
    };

    let input_owned = input.to_string();
    let flags_owned = flags.to_string();

    let exec_result = tokio::time::timeout(
        REGEX_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            // Flag 'g' handling:
            // If `flags` contains 'g', `replace_all` is used to replace all occurrences.
            // If 'g' is absent (e.g. empty flags "" or non-global flags like "i"), `replacen(..., 1, ...)`
            // is used to replace only the first occurrence.
            // Default behavior in Spec § 3.4 and DB schema is 'g' (global replacement).
            if flags_owned.contains('g') {
                re.replace_all(&input_owned, sub_replacement.as_str()).to_string()
            } else {
                re.replacen(&input_owned, 1, sub_replacement.as_str()).to_string()
            }
        }),
    )
    .await;

    let elapsed = start.elapsed().as_millis() as u32;

    match exec_result {
        Ok(Ok(output)) => ScriptExecutionResultDto {
            output,
            execution_time_ms: elapsed,
            console_logs: vec![],
            error: None,
        },
        Ok(Err(join_err)) => ScriptExecutionResultDto {
            output: input.to_string(),
            execution_time_ms: elapsed,
            console_logs: vec![],
            error: Some(format!("RegEx thread panicked: {}", join_err)),
        },
        Err(_) => ScriptExecutionResultDto {
            output: input.to_string(),
            execution_time_ms: elapsed,
            console_logs: vec![],
            error: Some("RegEx-Ausführung hat das Zeitlimit überschritten — Pattern könnte pathologisches Backtracking verursachen".to_string()),
        },
    }
}

#[tauri::command]
pub async fn execute_builtin(
    builtin_id: String,
    input: String,
    params: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let p = params.unwrap_or_default();
    builtins::execute_builtin(&builtin_id, &input, &p)
}

/// Evaluiert eine PipelineCondition gegen den aktuellen Input-Text.
/// Gibt Ok(true) wenn Schritt ausgeführt werden soll, Ok(false) wenn übersprungen.
fn evaluate_condition(condition_json: &str, input: &str) -> Result<bool, String> {
    let val: serde_json::Value = serde_json::from_str(condition_json)
        .map_err(|e| format!("Ungültige Condition JSON: {}", e))?;

    let ctype = val.get("_type").and_then(|v| v.as_str()).unwrap_or("");

    match ctype {
        "content_type_is" => {
            // Prüft ob der Input einem bestimmten Content-Type entspricht (heuristic detection)
            let types = val.get("types").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            let detected = detect_content_type_heuristic(input);
            Ok(types.iter().any(|t| *t == detected))
        }
        "size_gt" => {
            let bytes = val.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
            Ok(input.len() as u64 > bytes)
        }
        "size_lt" => {
            let bytes = val.get("bytes").and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
            Ok((input.len() as u64) < bytes)
        }
        "contains_regex" => {
            let pattern = val.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
            Ok(re.is_match(input))
        }
        "line_count_gt" => {
            let n = val.get("n").and_then(|v| v.as_u64()).unwrap_or(0);
            Ok(input.lines().count() as u64 > n)
        }
        _ => {
            eprintln!("Unbekannter PipelineCondition-Typ: {}", ctype);
            Ok(true) // Unbekannte Conditions: Schritt ausführen
        }
    }
}

/// Einfache heuristische Content-Type-Erkennung für PipelineCondition
fn detect_content_type_heuristic(s: &str) -> &'static str {
    let t = s.trim();
    if t.starts_with('{') || t.starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(t).is_ok() { return "json"; }
    }
    if t.starts_with("---\n") || regex::Regex::new(r"#{1,6}\s").unwrap().is_match(t) { return "markdown"; }
    if t.starts_with('<') && t.contains("</") { return "html"; }
    if t.starts_with("<?xml") { return "xml"; }
    if regex::Regex::new(r"(?i)^(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER)\s").unwrap().is_match(t) { return "sql"; }
    if t.starts_with("#!/bin/") { return "bash"; }
    "plain_text"
}

#[tauri::command]
pub async fn run_pipeline(
    pipeline_id: String,
    input: String,
    state: State<'_, AppState>,
) -> Result<PipelineExecutionResultDto, String> {
    let start = std::time::Instant::now();

    let pipeline_exists = sqlx::query("SELECT 1 FROM pipelines WHERE id = ?")
        .bind(&pipeline_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if pipeline_exists.is_none() {
        return Err(format!("Pipeline not found: {}", pipeline_id));
    }

    #[derive(sqlx::FromRow)]
    struct StepRow {
        id: String,
        label: String,
        script_id: Option<String>,
        enabled: i64,
        failure_policy: String,
        condition_json: Option<String>,
    }

    let steps = sqlx::query_as::<_, StepRow>(
        "SELECT id, label, script_id, enabled, failure_policy, condition_json FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC"
    )
    .bind(&pipeline_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let mut current_text = input;
    let mut step_results = Vec::new();
    let mut overall_success = true;
    let mut skipped_steps: Vec<String> = Vec::new();

    for step in steps {
        if step.enabled == 0 {
            continue;
        }

        // Schritt 1: Condition prüfen
        let condition_result: Option<bool> = if let Some(cond_json) = &step.condition_json {
            match evaluate_condition(cond_json, &current_text) {
                Ok(result) => Some(result),
                Err(e) => {
                    eprintln!("Condition evaluation error in step {}: {}", step.id, e);
                    Some(true) // Bei Fehler: Schritt ausführen
                }
            }
        } else {
            None // Kein Condition → immer ausführen
        };


        // Wenn Condition false: Schritt überspringen
        if condition_result == Some(false) {
            skipped_steps.push(step.id.clone());
            step_results.push(PipelineStepResultDto {
                step_id: step.id.clone(),
                step_label: step.label.clone(),
                output: current_text.clone(),
                execution_time_ms: 0,
                error: None,
                was_skipped: true,
                condition_result,
                failure_policy: step.failure_policy.clone(),
            });
            continue;
        }

        if let Some(script_id) = step.script_id {
            let step_start = std::time::Instant::now();
            let res = execute_script(
                ExecuteScriptDto {
                    script_id: Some(script_id),
                    js_code: None,
                    regex_pattern: None,
                    regex_replacement: None,
                    regex_flags: None,
                    input: current_text.clone(),
                    params_json: None,
                },
                state.clone(),
            )
            .await?;

            let error = res.error.clone();
            let step_elapsed = step_start.elapsed().as_millis() as u32;
            let new_output = res.output.clone();

            let has_error = error.is_some();

            step_results.push(PipelineStepResultDto {
                step_id: step.id.clone(),
                step_label: step.label.clone(),
                output: if has_error { current_text.clone() } else { new_output.clone() },
                execution_time_ms: step_elapsed,
                error: error.clone(),
                was_skipped: false,
                condition_result,
                failure_policy: step.failure_policy.clone(),
            });

            if has_error {
                overall_success = false;
                match step.failure_policy.as_str() {
                    "abort" => {
                        // Pipeline sofort abbrechen
                        break;
                    }
                    "warn" => {
                        // Warnung, aber Input unverändert weitergeben
                        // current_text bleibt
                    }
                    "passthrough" => {
                        // Input unverändert durch (current_text bleibt gleich)
                        // Fehler wird protokolliert, aber Pipeline läuft weiter
                    }
                    _ => {
                        // Unbekannte Policy → abort (safe default)
                        break;
                    }
                }
            } else {
                current_text = new_output;
            }
        }
    }

    let elapsed = start.elapsed().as_millis() as u32;

    Ok(PipelineExecutionResultDto {
        final_output: current_text,
        step_results,
        total_time_ms: elapsed,
        is_success: overall_success,
        skipped_steps,
    })
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptFullDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script_type: String,
    pub category: String,
    pub js_code: Option<String>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,
    pub regex_flags: String,
    pub is_favorite: bool,
    pub usage_count: u32,
    pub current_version: u32,
    pub color: String,
    pub parameters_json: String,
    pub tags_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScriptDto {
    pub name: String,
    pub description: Option<String>,
    pub script_type: Option<String>,
    pub category: Option<String>,
    pub js_code: Option<String>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,
    pub regex_flags: Option<String>,
    pub color: Option<String>,
    pub parameters_json: Option<String>,
    pub tags_json: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScriptDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub script_type: Option<String>,
    pub category: Option<String>,
    pub js_code: Option<String>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,
    pub regex_flags: Option<String>,
    pub is_favorite: Option<bool>,
    pub color: Option<String>,
    pub parameters_json: Option<String>,
    pub tags_json: Option<String>,
}

#[tauri::command]
pub async fn list_scripts(
    state: State<'_, AppState>,
) -> Result<Vec<ScriptFullDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        description: String,
        script_type: String,
        category: String,
        js_code: Option<String>,
        regex_pattern: Option<String>,
        regex_replacement: Option<String>,
        regex_flags: String,
        is_favorite: i64,
        usage_count: i64,
        current_version: i64,
        color: String,
        parameters_json: String,
        tags_json: String,
        created_at: i64,
        updated_at: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT id, name, description, script_type, category, js_code, regex_pattern, regex_replacement, regex_flags, is_favorite, usage_count, current_version, color, parameters_json, tags_json, created_at, updated_at FROM scripts ORDER BY is_favorite DESC, name ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| ScriptFullDto {
        id: r.id,
        name: r.name,
        description: r.description,
        script_type: r.script_type,
        category: r.category,
        js_code: r.js_code,
        regex_pattern: r.regex_pattern,
        regex_replacement: r.regex_replacement,
        regex_flags: r.regex_flags,
        is_favorite: r.is_favorite != 0,
        usage_count: r.usage_count as u32,
        current_version: r.current_version as u32,
        color: r.color,
        parameters_json: r.parameters_json,
        tags_json: r.tags_json,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect())
}

#[tauri::command]
pub async fn get_script(
    id: String,
    state: State<'_, AppState>,
) -> Result<ScriptFullDto, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        description: String,
        script_type: String,
        category: String,
        js_code: Option<String>,
        regex_pattern: Option<String>,
        regex_replacement: Option<String>,
        regex_flags: String,
        is_favorite: i64,
        usage_count: i64,
        current_version: i64,
        color: String,
        parameters_json: String,
        tags_json: String,
        created_at: i64,
        updated_at: i64,
    }

    let r = sqlx::query_as::<_, Row>(
        "SELECT id, name, description, script_type, category, js_code, regex_pattern, regex_replacement, regex_flags, is_favorite, usage_count, current_version, color, parameters_json, tags_json, created_at, updated_at FROM scripts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Script not found".to_string())?;

    Ok(ScriptFullDto {
        id: r.id,
        name: r.name,
        description: r.description,
        script_type: r.script_type,
        category: r.category,
        js_code: r.js_code,
        regex_pattern: r.regex_pattern,
        regex_replacement: r.regex_replacement,
        regex_flags: r.regex_flags,
        is_favorite: r.is_favorite != 0,
        usage_count: r.usage_count as u32,
        current_version: r.current_version as u32,
        color: r.color,
        parameters_json: r.parameters_json,
        tags_json: r.tags_json,
        created_at: r.created_at,
        updated_at: r.updated_at,
    })
}

/// Validates regex pattern compilation before saving/updating a script.
/// Parameter placeholders `{{...}}` are substituted with dummy text during testing.
pub fn validate_regex_pattern(pattern: &str, flags: &str) -> Result<(), String> {
    let test_pattern = substitute_dummy_params(pattern);
    let mut builder = regex::RegexBuilder::new(&test_pattern);
    if flags.contains('i') {
        builder.case_insensitive(true);
    }
    if flags.contains('m') {
        builder.multi_line(true);
    }
    if flags.contains('s') {
        builder.dot_matches_new_line(true);
    }

    builder
        .build()
        .map_err(|e| format!("Ungültiges RegEx-Muster: {}", e))?;
    Ok(())
}

fn substitute_dummy_params(pattern: &str) -> String {
    if let Ok(re) = regex::Regex::new(r"\{\{.*?\}\}") {
        re.replace_all(pattern, "dummy").to_string()
    } else {
        pattern.to_string()
    }
}

#[tauri::command]
pub async fn create_script(
    draft: CreateScriptDto,
    state: State<'_, AppState>,
) -> Result<ScriptFullDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let desc = draft.description.unwrap_or_default();
    let stype = draft.script_type.unwrap_or_else(|| "js".to_string());
    let cat = draft.category.unwrap_or_else(|| "custom".to_string());
    let flags = draft.regex_flags.unwrap_or_else(|| "g".to_string());
    let color = draft.color.unwrap_or_else(|| "#6366f1".to_string());
    let parameters = draft.parameters_json.unwrap_or_else(|| "[]".to_string());
    let tags = draft.tags_json.unwrap_or_else(|| "[]".to_string());

    if stype == "regex" {
        if let Some(ref pattern) = draft.regex_pattern {
            validate_regex_pattern(pattern, &flags)?;
        }
    }

    sqlx::query(
        "INSERT INTO scripts (id, name, description, script_type, category, js_code, regex_pattern, regex_replacement, regex_flags, color, parameters_json, tags_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&draft.name).bind(&desc).bind(&stype).bind(&cat)
    .bind(&draft.js_code).bind(&draft.regex_pattern).bind(&draft.regex_replacement).bind(&flags)
    .bind(&color).bind(&parameters).bind(&tags)
    .bind(now).bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ScriptFullDto {
        id,
        name: draft.name,
        description: desc,
        script_type: stype,
        category: cat,
        js_code: draft.js_code,
        regex_pattern: draft.regex_pattern,
        regex_replacement: draft.regex_replacement,
        regex_flags: flags,
        is_favorite: false,
        usage_count: 0,
        current_version: 1,
        color,
        parameters_json: parameters,
        tags_json: tags,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_script(
    id: String,
    draft: UpdateScriptDto,
    state: State<'_, AppState>,
) -> Result<ScriptFullDto, String> {
    let existing = get_script(id.clone(), state.clone()).await?;
    let now = chrono::Utc::now().timestamp_millis();

    let name = draft.name.unwrap_or(existing.name);
    let description = draft.description.unwrap_or(existing.description);
    let script_type = draft.script_type.unwrap_or(existing.script_type);
    let category = draft.category.unwrap_or(existing.category);
    let js_code = draft.js_code.or(existing.js_code);
    let regex_pattern = draft.regex_pattern.or(existing.regex_pattern);
    let regex_replacement = draft.regex_replacement.or(existing.regex_replacement);
    let regex_flags = draft.regex_flags.unwrap_or(existing.regex_flags);
    let is_favorite = draft.is_favorite.unwrap_or(existing.is_favorite);
    let color = draft.color.unwrap_or(existing.color);
    let parameters_json = draft.parameters_json.unwrap_or(existing.parameters_json);
    let tags_json = draft.tags_json.unwrap_or(existing.tags_json);

    if script_type == "regex" {
        if let Some(ref pattern) = regex_pattern {
            validate_regex_pattern(pattern, &regex_flags)?;
        }
    }

    sqlx::query(
        "UPDATE scripts SET name = ?, description = ?, script_type = ?, category = ?, js_code = ?, regex_pattern = ?, regex_replacement = ?, regex_flags = ?, is_favorite = ?, color = ?, parameters_json = ?, tags_json = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&name).bind(&description).bind(&script_type).bind(&category)
    .bind(&js_code).bind(&regex_pattern).bind(&regex_replacement).bind(&regex_flags)
    .bind(if is_favorite { 1 } else { 0 }).bind(&color).bind(&parameters_json).bind(&tags_json).bind(now).bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    get_script(id, state).await
}

#[tauri::command]
pub async fn delete_script(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM scripts WHERE id = ?").bind(&id).execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineFullDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub steps: Vec<PipelineStepFullDto>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStepFullDto {
    pub id: String,
    pub pipeline_id: String,
    pub script_id: Option<String>,
    pub step_order: u32,
    pub label: String,
    pub enabled: bool,
    pub failure_policy: String,
    pub condition_json: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePipelineDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePipelineDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_favorite: Option<bool>,
}

#[tauri::command]
pub async fn list_pipelines(
    state: State<'_, AppState>,
) -> Result<Vec<PipelineFullDto>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        description: String,
        is_favorite: i64,
        created_at: i64,
        updated_at: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT id, name, description, is_favorite, created_at, updated_at FROM pipelines ORDER BY name ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for r in rows {
        #[derive(sqlx::FromRow)]
        struct StepRow {
            id: String,
            pipeline_id: String,
            script_id: Option<String>,
            step_order: i64,
            label: String,
            enabled: i64,
            failure_policy: String,
            condition_json: Option<String>,
        }

        let step_rows = sqlx::query_as::<_, StepRow>(
            "SELECT id, pipeline_id, script_id, step_order, label, enabled, failure_policy, condition_json FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC"
        )
        .bind(&r.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let steps = step_rows.into_iter().map(|s| PipelineStepFullDto {
            id: s.id,
            pipeline_id: s.pipeline_id,
            script_id: s.script_id,
            step_order: s.step_order as u32,
            label: s.label,
            enabled: s.enabled != 0,
            failure_policy: s.failure_policy,
            condition_json: s.condition_json,
        }).collect();

        result.push(PipelineFullDto {
            id: r.id,
            name: r.name,
            description: r.description,
            is_favorite: r.is_favorite != 0,
            created_at: r.created_at,
            updated_at: r.updated_at,
            steps,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_pipeline(
    id: String,
    state: State<'_, AppState>,
) -> Result<PipelineFullDto, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        name: String,
        description: String,
        is_favorite: i64,
        created_at: i64,
        updated_at: i64,
    }

    let r = sqlx::query_as::<_, Row>(
        "SELECT id, name, description, is_favorite, created_at, updated_at FROM pipelines WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Pipeline not found".to_string())?;

    #[derive(sqlx::FromRow)]
    struct StepRow {
        id: String,
        pipeline_id: String,
        script_id: Option<String>,
        step_order: i64,
        label: String,
        enabled: i64,
        failure_policy: String,
        condition_json: Option<String>,
    }

    let step_rows = sqlx::query_as::<_, StepRow>(
        "SELECT id, pipeline_id, script_id, step_order, label, enabled, failure_policy, condition_json FROM pipeline_steps WHERE pipeline_id = ? ORDER BY step_order ASC"
    )
    .bind(&r.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let steps = step_rows.into_iter().map(|s| PipelineStepFullDto {
        id: s.id,
        pipeline_id: s.pipeline_id,
        script_id: s.script_id,
        step_order: s.step_order as u32,
        label: s.label,
        enabled: s.enabled != 0,
        failure_policy: s.failure_policy,
        condition_json: s.condition_json,
    }).collect();

    Ok(PipelineFullDto {
        id: r.id,
        name: r.name,
        description: r.description,
        is_favorite: r.is_favorite != 0,
        created_at: r.created_at,
        updated_at: r.updated_at,
        steps,
    })
}

#[tauri::command]
pub async fn create_pipeline(
    draft: CreatePipelineDto,
    state: State<'_, AppState>,
) -> Result<PipelineFullDto, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let desc = draft.description.unwrap_or_default();

    sqlx::query("INSERT INTO pipelines (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(&draft.name).bind(&desc).bind(now).bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(PipelineFullDto {
        id,
        name: draft.name,
        description: desc,
        is_favorite: false,
        created_at: now,
        updated_at: now,
        steps: vec![],
    })
}

#[tauri::command]
pub async fn update_pipeline(
    id: String,
    draft: UpdatePipelineDto,
    state: State<'_, AppState>,
) -> Result<PipelineFullDto, String> {
    let existing = get_pipeline(id.clone(), state.clone()).await?;
    let now = chrono::Utc::now().timestamp_millis();

    let name = draft.name.unwrap_or(existing.name);
    let description = draft.description.unwrap_or(existing.description);
    let is_favorite = draft.is_favorite.unwrap_or(existing.is_favorite);

    sqlx::query("UPDATE pipelines SET name = ?, description = ?, is_favorite = ?, updated_at = ? WHERE id = ?")
        .bind(&name).bind(&description).bind(if is_favorite { 1 } else { 0 }).bind(now).bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    get_pipeline(id, state).await
}

#[tauri::command]
pub async fn delete_pipeline(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM pipelines WHERE id = ?").bind(&id).execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_pipeline_step(
    pipeline_id: String,
    script_id: Option<String>,
    label: String,
    order: Option<u32>,
    state: State<'_, AppState>,
) -> Result<PipelineStepFullDto, String> {
    let step_id = uuid::Uuid::new_v4().to_string();

    let step_order = match order {
        Some(o) => o,
        None => {
            #[derive(sqlx::FromRow)]
            struct MaxOrderRow { max_order: Option<i64> }
            let row = sqlx::query_as::<_, MaxOrderRow>(
                "SELECT MAX(step_order) as max_order FROM pipeline_steps WHERE pipeline_id = ?"
            )
            .bind(&pipeline_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;
            (row.max_order.unwrap_or(-1) + 1) as u32
        }
    };

    sqlx::query(
        "INSERT INTO pipeline_steps (id, pipeline_id, script_id, step_order, label, enabled, failure_policy) VALUES (?, ?, ?, ?, ?, 1, 'abort')"
    )
    .bind(&step_id).bind(&pipeline_id).bind(&script_id).bind(step_order as i64).bind(&label)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(PipelineStepFullDto {
        id: step_id,
        pipeline_id,
        script_id,
        step_order,
        label,
        enabled: true,
        failure_policy: "abort".to_string(),
        condition_json: None,
    })
}

#[tauri::command]
pub async fn remove_pipeline_step(
    step_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM pipeline_steps WHERE id = ?")
        .bind(&step_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformClipboardResultDto {
    pub original_content: String,
    pub transformed_content: String,
    pub execution_time_ms: u32,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn transform_clipboard_entry(
    entry_id: String,
    script_id: Option<String>,
    pipeline_id: Option<String>,
    params_json: Option<String>,
    state: State<'_, AppState>,
) -> Result<TransformClipboardResultDto, String> {
    let original_content: String = sqlx::query_scalar("SELECT content FROM clipboard_history WHERE id = ?")
        .bind(&entry_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Clipboard entry '{}' not found", entry_id))?;

    if let Some(pipe_id) = pipeline_id {
        let res = run_pipeline(pipe_id, original_content.clone(), state).await?;
        let err = if !res.is_success {
            let error_step = res.step_results.iter().find(|s| s.error.is_some());
            error_step.and_then(|s| s.error.clone()).or_else(|| Some("Pipeline execution failed".to_string()))
        } else {
            None
        };
        Ok(TransformClipboardResultDto {
            original_content,
            transformed_content: res.final_output,
            execution_time_ms: res.total_time_ms,
            error: err,
        })
    } else if let Some(s_id) = script_id {
        let res = execute_script(
            ExecuteScriptDto {
                script_id: Some(s_id),
                js_code: None,
                regex_pattern: None,
                regex_replacement: None,
                regex_flags: None,
                input: original_content.clone(),
                params_json,
            },
            state,
        )
        .await?;
        Ok(TransformClipboardResultDto {
            original_content,
            transformed_content: res.output,
            execution_time_ms: res.execution_time_ms,
            error: res.error,
        })
    } else {
        Err("Neither script_id nor pipeline_id provided".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;
    use std::sync::Mutex;
    use lru::LruCache;

    #[tokio::test]
    async fn test_regex_g_flag_global_vs_first_only() {
        let start = std::time::Instant::now();
        let input = "foo bar foo baz foo";

        // With 'g' flag -> replace all
        let res_g = run_regex_transformation(input, "foo", "QUX", "g", None, None, start).await;
        assert_eq!(res_g.output, "QUX bar QUX baz QUX");
        assert!(res_g.error.is_none());

        // Without 'g' flag (e.g. empty or "i") -> replace first occurrence only
        let res_nog = run_regex_transformation(input, "foo", "QUX", "", None, None, start).await;
        assert_eq!(res_nog.output, "QUX bar foo baz foo");
        assert!(res_nog.error.is_none());

        let res_i = run_regex_transformation(input, "FOO", "QUX", "i", None, None, start).await;
        assert_eq!(res_i.output, "QUX bar foo baz foo");
        assert!(res_i.error.is_none());
    }

    #[tokio::test]
    async fn test_regex_input_size_limit() {
        let start = std::time::Instant::now();
        let large_input = "a".repeat(MAX_REGEX_INPUT_BYTES + 1);

        let res = run_regex_transformation(&large_input, "a", "b", "g", None, None, start).await;
        assert!(res.error.is_some());
        let err_msg = res.error.unwrap();
        assert!(err_msg.contains("exceeds maximum limit"));
    }

    #[tokio::test]
    async fn test_regex_redos_timeout() {
        let start = std::time::Instant::now();
        // Heavy replacement workload on a large input (1.9 MB)
        let input = "a".repeat(1_900_000);
        let pattern = "a?";
        let replacement = "abcdefghijklmnopqrstuvwxyz1234567890";

        let res = run_regex_transformation(&input, pattern, replacement, "g", None, None, start).await;
        assert!(res.error.is_some());
        let err_msg = res.error.unwrap();
        assert!(err_msg.contains("Zeitlimit überschritten"));
    }

    #[tokio::test]
    async fn test_regex_caching() {
        let start = std::time::Instant::now();
        let cache = Mutex::new(LruCache::new(NonZeroUsize::new(10).unwrap()));

        let pattern = "foo";
        let input = "foo bar";

        // First call -> Cache Miss & Compiles
        let res1 = run_regex_transformation(input, pattern, "baz", "g", None, Some(&cache), start).await;
        assert_eq!(res1.output, "baz bar");
        assert_eq!(cache.lock().unwrap().len(), 1);

        // Second call with same pattern & flags -> Cache Hit
        let res2 = run_regex_transformation(input, pattern, "qux", "g", None, Some(&cache), start).await;
        assert_eq!(res2.output, "qux bar");
        assert_eq!(cache.lock().unwrap().len(), 1);

        // Call with parameter substitution -> Cache key formed after substitution
        let param_pattern = "hello {{target}}";
        let params_str1 = r#"{"target": "world"}"#;
        let params_str2 = r#"{"target": "there"}"#;

        run_regex_transformation("hello world", param_pattern, "hi", "g", Some(params_str1), Some(&cache), start).await;
        assert_eq!(cache.lock().unwrap().len(), 2); // "hello world" cached

        run_regex_transformation("hello world", param_pattern, "hi", "g", Some(params_str1), Some(&cache), start).await;
        assert_eq!(cache.lock().unwrap().len(), 2); // Hit for "hello world"

        run_regex_transformation("hello there", param_pattern, "hi", "g", Some(params_str2), Some(&cache), start).await;
        assert_eq!(cache.lock().unwrap().len(), 3); // "hello there" cached
    }

    #[test]
    fn test_validate_regex_pattern() {
        // Valid patterns
        assert!(validate_regex_pattern(r"\d+", "g").is_ok());
        assert!(validate_regex_pattern("(?i)hello {{name}}", "g").is_ok());

        // Invalid pattern
        let err = validate_regex_pattern("[a-z", "g");
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Ungültiges RegEx-Muster"));
    }
}

#[tauri::command]
pub async fn reorder_pipeline_steps(
    pipeline_id: String,
    step_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    for (idx, step_id) in step_ids.into_iter().enumerate() {
        sqlx::query("UPDATE pipeline_steps SET step_order = ? WHERE id = ? AND pipeline_id = ?")
            .bind(idx as i64)
            .bind(&step_id)
            .bind(&pipeline_id)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_pipeline_step(
    step_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("UPDATE pipeline_steps SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(&step_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
