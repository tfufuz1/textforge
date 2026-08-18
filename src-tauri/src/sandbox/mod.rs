use rquickjs::{Context, Function, Runtime};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Maximum execution time for user scripts (§ 22 SETTINGS_SCHEMA default).
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(3);

/// Maximum output size (512 KB as per spec).
const MAX_OUTPUT_BYTES: usize = 512 * 1024;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScriptExecutionResultDto {
    pub output: String,
    pub execution_time_ms: u32,
    pub console_logs: Vec<String>,
    pub error: Option<String>,
}

/// Public async entry point: runs the script with a timeout guard.
///
/// If the script exceeds `SCRIPT_TIMEOUT`, the blocking thread is abandoned
/// and a timeout error is returned. The QuickJS runtime will be dropped
/// when the abandoned thread eventually finishes or is cleaned up.
pub async fn run_script_in_sandbox(
    js_code: String,
    input_text: String,
    params_json: Option<String>,
) -> ScriptExecutionResultDto {
    let start = std::time::Instant::now();
    let input_text_fallback = input_text.clone();

    let result = tokio::time::timeout(
        SCRIPT_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            run_script_sync(&js_code, &input_text, params_json.as_deref())
        }),
    )
    .await;

    match result {
        // Completed within timeout
        Ok(Ok(dto)) => dto,
        // spawn_blocking panicked
        Ok(Err(join_err)) => ScriptExecutionResultDto {
            output: input_text_fallback,
            execution_time_ms: start.elapsed().as_millis() as u32,
            console_logs: vec![],
            error: Some(format!("Script thread panicked: {}", join_err)),
        },
        // Timeout elapsed
        Err(_) => ScriptExecutionResultDto {
            output: String::new(),
            execution_time_ms: start.elapsed().as_millis() as u32,
            console_logs: vec![],
            error: Some(format!(
                "Script execution timed out ({}s limit)",
                SCRIPT_TIMEOUT.as_secs()
            )),
        },
    }
}

/// Synchronous script execution inside QuickJS with console.log capture.
fn run_script_sync(
    js_code: &str,
    input_text: &str,
    params_json: Option<&str>,
) -> ScriptExecutionResultDto {
    let start = std::time::Instant::now();

    let runtime = match Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            return ScriptExecutionResultDto {
                output: input_text.to_string(),
                execution_time_ms: start.elapsed().as_millis() as u32,
                console_logs: vec![],
                error: Some(format!("Failed to create JS runtime: {}", e)),
            }
        }
    };

    let context = match Context::full(&runtime) {
        Ok(c) => c,
        Err(e) => {
            return ScriptExecutionResultDto {
                output: input_text.to_string(),
                execution_time_ms: start.elapsed().as_millis() as u32,
                console_logs: vec![],
                error: Some(format!("Failed to create JS context: {}", e)),
            }
        }
    };

    let result = context.with(|ctx| -> (Result<String, String>, Vec<String>) {
        let globals = ctx.globals();

        // --- Set up console.log/warn/error capture ---
        let console_logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

        let console_obj = match rquickjs::Object::new(ctx.clone()) {
            Ok(obj) => obj,
            Err(e) => return (Err(format!("Failed to create console object: {}", e)), vec![]),
        };

        // Helper macro to avoid repetition for log/warn/error
        macro_rules! register_console_fn {
            ($name:expr, $logs:expr) => {{
                let logs_ref = $logs.clone();
                let func = Function::new(ctx.clone(), move |args: rquickjs::function::Rest<rquickjs::Value>| {
                    let parts: Vec<String> = args.0.iter().map(|v| {
                        if let Some(s) = v.as_string() {
                            s.to_string().unwrap_or_default()
                        } else {
                            format!("{:?}", v)
                        }
                    }).collect();
                    logs_ref.borrow_mut().push(parts.join(" "));
                });
                match func {
                    Ok(f) => { let _ = console_obj.set($name, f); },
                    Err(e) => return (Err(format!("Failed to create console.{}: {}", $name, e)), vec![]),
                }
            }};
        }

        register_console_fn!("log", console_logs);
        register_console_fn!("warn", console_logs);
        register_console_fn!("error", console_logs);
        register_console_fn!("info", console_logs);

        if let Err(e) = globals.set("console", console_obj) {
            return (Err(format!("Failed to set console global: {}", e)), vec![]);
        }

        // --- Set input variables ---
        if let Err(e) = globals.set("inputText", input_text) {
            return (Err(e.to_string()), vec![]);
        }
        if let Err(e) = globals.set("sandboxParamsJson", params_json.unwrap_or("{}")) {
            return (Err(e.to_string()), vec![]);
        }

        let prelude = r#"
            const utils = {
                lines: (t) => String(t || '').split(/\r?\n/),
                unlines: (arr) => Array.isArray(arr) ? arr.join('\n') : String(arr),
                words: (t) => String(t || '').trim().split(/\s+/).filter(Boolean),
                sortLines: (t) => utils.lines(t).sort().join('\n'),
                uniqueLines: (t) => Array.from(new Set(utils.lines(t))).join('\n'),
                reverseLines: (t) => utils.lines(t).reverse().join('\n'),
                trim: (t) => String(t || '').trim(),
                uppercase: (t) => String(t || '').toUpperCase(),
                lowercase: (t) => String(t || '').toLowerCase(),
                prettyJSON: (t) => JSON.stringify(JSON.parse(t), null, 2),
                minifyJSON: (t) => JSON.stringify(JSON.parse(t)),
                base64Encode: (t) => btoa(t),
                base64Decode: (t) => atob(t),
                redact: (t, mask = '***') => String(t || '').replace(/[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g, mask)
            };
        "#;

        let wrapped_code = format!(
            r#"
            (function() {{
                {prelude}
                const input = inputText;
                let params = {{}};
                try {{
                    params = JSON.parse(sandboxParamsJson);
                }} catch(e) {{}}
                let transform = function(text, params) {{
                    {js_code}
                }};
                return transform(input, params);
            }})()
            "#
        );

        let eval_res: Result<String, _> = ctx.eval(wrapped_code);
        let logs = console_logs.borrow().clone();
        match eval_res {
            Ok(mut res) => {
                // Enforce output size limit
                if res.len() > MAX_OUTPUT_BYTES {
                    res.truncate(MAX_OUTPUT_BYTES);
                }
                (Ok(res), logs)
            }
            Err(e) => (Err(e.to_string()), logs),
        }
    });

    let elapsed = start.elapsed().as_millis() as u32;

    match result {
        (Ok(output), logs) => ScriptExecutionResultDto {
            output,
            execution_time_ms: elapsed,
            console_logs: logs,
            error: None,
        },
        (Err(err), logs) => ScriptExecutionResultDto {
            output: input_text.to_string(),
            execution_time_ms: elapsed,
            console_logs: logs,
            error: Some(err),
        },
    }
}
