pub mod source_app;

use tauri::{AppHandle, Manager, Emitter};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::AppState;


pub struct ClipboardMonitorConfig {
    pub min_content_length: usize,
    pub dedup_window_ms: u64,
    pub max_entries: u32,
}

impl Default for ClipboardMonitorConfig {
    fn default() -> Self {
        Self {
            min_content_length: 3,
            dedup_window_ms: 500,
            max_entries: 500,
        }
    }
}

#[derive(Debug)]
pub enum MonitorError {
    WlPasteNotFound,
    WaylandSubprocessFailed(String),
    FallbackPollingError(String),
}

/// Try to start `wl-paste --watch` first; if that fails, fall back to arboard polling.
pub async fn start_monitor(
    app_handle: AppHandle,
) -> Result<(), MonitorError> {
    match try_wl_paste_monitor(app_handle.clone()).await {
        Ok(()) => {
            eprintln!("Clipboard monitor: using wl-paste --watch");
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "wl-paste monitor failed ({:?}), falling back to arboard polling (500ms)",
                e
            );
            start_arboard_polling(app_handle).await
        }
    }
}

/// Primary monitor: uses `wl-paste --watch` for event-driven clipboard monitoring.
async fn try_wl_paste_monitor(app_handle: AppHandle) -> Result<(), MonitorError> {
    let mut child = tokio::process::Command::new("wl-paste")
        .arg("--watch")
        .arg("--no-newline")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| MonitorError::WaylandSubprocessFailed(e.to_string()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or(MonitorError::WaylandSubprocessFailed("No stdout".into()))?;

    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let content = line;
            if content.len() < 3 {
                continue;
            }
            insert_clipboard_entry(&app_handle, &content).await;
        }
    });

    Ok(())
}

/// Fallback monitor: polls `arboard::Clipboard` every 500ms.
/// Used when wl-paste is not available (e.g. X11, missing wl-clipboard package).
async fn start_arboard_polling(app_handle: AppHandle) -> Result<(), MonitorError> {
    // Verify that arboard can open the clipboard at all before spawning the loop
    let _test_clipboard = arboard::Clipboard::new()
        .map_err(|e| MonitorError::FallbackPollingError(format!("Cannot open clipboard: {}", e)))?;

    tokio::task::spawn_blocking(move || {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                eprintln!("arboard polling: failed to open clipboard: {}", e);
                return;
            }
        };

        let mut last_text = String::new();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let current = match clipboard.get_text() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if current == last_text || current.len() < 3 {
                continue;
            }

            last_text = current.clone();

            // Use a handle to the tokio runtime to run the async insertion
            let handle = app_handle.clone();
            let text = current;
            tauri::async_runtime::spawn(async move {
                insert_clipboard_entry(&handle, &text).await;
            });
        }
    });

    Ok(())
}

/// Shared insertion logic used by both wl-paste and arboard monitors.
async fn insert_clipboard_entry(app_handle: &AppHandle, content: &str) {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let state = app_handle.state::<AppState>();
    let pool = &state.db;

    // Check duplicate
    let existing = sqlx::query("SELECT id FROM clipboard_history WHERE content_hash = ?")
        .bind(&hash)
        .fetch_optional(pool)
        .await;

    match existing {
        Ok(None) => {
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp_millis();

            let content_type = if content.starts_with("http://") || content.starts_with("https://")
            {
                "url"
            } else if content.starts_with('{') || content.starts_with('[') {
                "json"
            } else {
                "plain_text"
            };

            let source_app = source_app::detect_source_app().await;

            let res = sqlx::query(
                "INSERT INTO clipboard_history (id, content, content_hash, content_type, source_app, captured_at) VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind(&content)
            .bind(&hash)
            .bind(&content_type)
            .bind(&source_app)
            .bind(now)
            .execute(pool)
            .await;

            if res.is_ok() {
                app_handle.emit("clipboard:new_entry", &id).ok();

                // Delete unpinned items if history > 500
                let count_res: Result<(i64,), _> =
                    sqlx::query_as("SELECT COUNT(*) FROM clipboard_history")
                        .fetch_one(pool)
                        .await;
                if let Ok(count) = count_res {
                    if count.0 > 500 {
                        sqlx::query("DELETE FROM clipboard_history WHERE is_pinned = 0 ORDER BY captured_at ASC LIMIT ?")
                            .bind(count.0 - 500)
                            .execute(pool)
                            .await
                            .ok();
                    }
                }
            }
        }
        _ => {} // Duplicate or error, ignore
    }
}
