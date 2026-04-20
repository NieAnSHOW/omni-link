use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine;
use serde::Deserialize;
use tauri::{AppHandle, Listener, Manager, WebviewUrl};
use tauri::webview::{PageLoadEvent, WebviewWindowBuilder};

use crate::error::{AppError, AppResult};

static EXTRACTOR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize, Clone)]
pub struct WebviewExtractResult {
    pub success: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub markdown: String,
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub raw_html: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub images: Vec<String>,
}

pub async fn extract_via_webview(app: &AppHandle, url: &str) -> AppResult<WebviewExtractResult> {
    let (tx, mut rx) = tokio::sync::oneshot::channel::<WebviewExtractResult>();

    let unique_id = EXTRACTOR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let label = format!("extractor-{}", unique_id);
    let event_name = format!("extract-result-{}", unique_id);

    eprintln!("[WV-{}] === Starting extraction ===", unique_id);
    eprintln!("[WV-{}] URL: {}", unique_id, url);

    let parsed_url: url::Url = url
        .parse()
        .map_err(|e: url::ParseError| AppError::Parse(e.to_string()))?;

    let resources_dir = app
        .path()
        .resource_dir()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    let js_dir = resources_dir.join("resources").join("js");
    eprintln!("[WV-{}] JS dir: {}", unique_id, js_dir.display());

    let readability_js = std::fs::read_to_string(js_dir.join("readability.js"))
        .map_err(|e| AppError::Parse(format!("Failed to load readability.js: {}", e)))?;
    let turndown_js = std::fs::read_to_string(js_dir.join("turndown.js"))
        .map_err(|e| AppError::Parse(format!("Failed to load turndown.js: {}", e)))?;
    let extract_js = std::fs::read_to_string(js_dir.join("extract.js"))
        .map_err(|e| AppError::Parse(format!("Failed to load extract.js: {}", e)))?;

    eprintln!("[WV-{}] JS loaded: readability={}B, turndown={}B, extract={}B",
        unique_id, readability_js.len(), turndown_js.len(), extract_js.len());

    // Register listener BEFORE building window to avoid race condition
    let app_clone = app.clone();
    let label_clone = label.clone();
    app.once(&event_name, move |event| {
        eprintln!("[WV-{}] Event received! payload len={}", unique_id, event.payload().len());
        if let Ok(result) = serde_json::from_str::<WebviewExtractResult>(event.payload()) {
            let _ = tx.send(result);
            if let Some(w) = app_clone.get_webview_window(&label_clone) {
                let _ = w.destroy();
            }
        } else {
            eprintln!("[WV-{}] Failed to deserialize event payload", unique_id);
        }
    });
    eprintln!("[WV-{}] Listener registered for event '{}'", unique_id, event_name);

    let event_name_clone = event_name.clone();
    let nav_result: Arc<Mutex<Option<WebviewExtractResult>>> = Arc::new(Mutex::new(None));
    let nav_result_clone = nav_result.clone();
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(parsed_url))
        .title("Content Extractor")
        .visible(false)
        .inner_size(1280.0, 800.0)
        .on_navigation(move |url| {
            if url.scheme() == "omnilink-extract" {
                eprintln!("[WV-{}] Navigation fallback triggered: {}", unique_id, url);
                if let Some(data_b64) = url.query_pairs().find(|(k, _)| k == "data").map(|(_, v)| v.to_string()) {
                    match base64::engine::general_purpose::STANDARD.decode(&data_b64) {
                        Ok(bytes) => match String::from_utf8(bytes) {
                            Ok(json_str) => match serde_json::from_str::<WebviewExtractResult>(&json_str) {
                                Ok(result) => {
                                    eprintln!("[WV-{}] Navigation fallback parsed OK, success={}", unique_id, result.success);
                                    *nav_result_clone.lock().unwrap() = Some(result);
                                }
                                Err(e) => eprintln!("[WV-{}] Navigation fallback JSON parse error: {}", unique_id, e),
                            },
                            Err(e) => eprintln!("[WV-{}] Navigation fallback UTF-8 error: {}", unique_id, e),
                        },
                        Err(e) => eprintln!("[WV-{}] Navigation fallback base64 decode error: {}", unique_id, e),
                    }
                }
                false
            } else {
                true
            }
        })
        .on_page_load(move |webview_window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                eprintln!("[WV-{}] Page loaded, injecting JS...", unique_id);
                let r1 = webview_window.eval(&readability_js);
                eprintln!("[WV-{}] eval(readability) => {:?}", unique_id, r1);
                let r2 = webview_window.eval(&turndown_js);
                eprintln!("[WV-{}] eval(turndown) => {:?}", unique_id, r2);
                let r3 = webview_window.eval(&format!(
                    "window.__EXTRACT_EVENT_NAME__ = '{}';",
                    event_name_clone
                ));
                eprintln!("[WV-{}] eval(event_name) => {:?}", unique_id, r3);
                let r4 = webview_window.eval(&extract_js);
                eprintln!("[WV-{}] eval(extract) => {:?}", unique_id, r4);
            }
        })
        .build()
        .map_err(|e| AppError::Parse(format!("Failed to create webview: {}", e)))?;

    eprintln!("[WV-{}] Window built, waiting for result (20s)...", unique_id);

    // Wait for either IPC event or navigation fallback
    let result = tokio::time::timeout(std::time::Duration::from_secs(20), async {
        // Poll for either IPC event (rx) or navigation fallback (nav_result)
        loop {
            // Check navigation fallback first
            if let Some(result) = nav_result.lock().unwrap().take() {
                return Ok(result);
            }
            // Check IPC event (non-blocking)
            match rx.try_recv() {
                Ok(result) => return Ok(result),
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                Err(_) => return Err("IPC channel closed".to_string()),
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| {
        eprintln!("[WV-{}] *** TIMEOUT *** destroying window", unique_id);
        let _ = window.destroy();
        AppError::Parse("Webview extraction timed out".into())
    })?
    .map_err(|e: String| AppError::Parse(e))?;

    eprintln!("[WV-{}] Done, success={}", unique_id, result.success);

    if !result.success {
        return Err(AppError::Parse(
            result
                .error
                .unwrap_or_else(|| "Unknown extraction error".into()),
        ));
    }

    Ok(result)
}
