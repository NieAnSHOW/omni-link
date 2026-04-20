use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

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
    let (tx, rx) = tokio::sync::oneshot::channel::<WebviewExtractResult>();

    let unique_id = EXTRACTOR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let label = format!("extractor-{}", unique_id);
    let event_name = format!("extract-result-{}", unique_id);

    let parsed_url: url::Url = url
        .parse()
        .map_err(|e: url::ParseError| AppError::Parse(e.to_string()))?;

    // Validate JS resource files before creating the WebView
    let resources_dir = app
        .path()
        .resource_dir()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    let js_dir = resources_dir.join("js");

    let readability_js = std::fs::read_to_string(js_dir.join("readability.js"))
        .map_err(|e| AppError::Parse(format!("Failed to load readability.js: {}", e)))?;
    if readability_js.trim().is_empty() {
        return Err(AppError::Parse("readability.js is empty".into()));
    }

    let turndown_js = std::fs::read_to_string(js_dir.join("turndown.js"))
        .map_err(|e| AppError::Parse(format!("Failed to load turndown.js: {}", e)))?;
    if turndown_js.trim().is_empty() {
        return Err(AppError::Parse("turndown.js is empty".into()));
    }

    let extract_js = std::fs::read_to_string(js_dir.join("extract.js"))
        .map_err(|e| AppError::Parse(format!("Failed to load extract.js: {}", e)))?;
    if extract_js.trim().is_empty() {
        return Err(AppError::Parse("extract.js is empty".into()));
    }

    let event_name_clone = event_name.clone();
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(parsed_url))
        .title("Content Extractor")
        .visible(false)
        .inner_size(1280.0, 800.0)
        .on_page_load(move |webview_window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                // Inject the scoped event name for the JS extractor
                let inject_event_name = format!(
                    "window.__EXTRACT_EVENT_NAME__ = '{}';",
                    event_name_clone
                );

                let combined = format!(
                    "(()=>{{{inject_event_name};{};{};{};}})();",
                    readability_js, turndown_js, extract_js
                );

                let _ = webview_window.eval(&combined);
            }
        })
        .build()
        .map_err(|e| AppError::Parse(format!("Failed to create webview: {}", e)))?;

    // Listen for extraction result (use once() for auto-deregistration)
    let app_clone = app.clone();
    let label_clone = label.clone();
    app.once(&event_name, move |event| {
        if let Ok(result) = serde_json::from_str::<WebviewExtractResult>(event.payload()) {
            let _ = tx.send(result);
            // Destroy the webview window
            if let Some(w) = app_clone.get_webview_window(&label_clone) {
                let _ = w.destroy();
            }
        }
    });

    // Wait for result with timeout using tokio
    let result = tokio::time::timeout(std::time::Duration::from_secs(20), rx)
        .await
        .map_err(|_| {
            let _ = window.destroy();
            AppError::Parse("Webview extraction timed out".into())
        })?
        .map_err(|_| AppError::Parse("Webview extraction cancelled".into()))?;

    if !result.success {
        return Err(AppError::Parse(
            result
                .error
                .unwrap_or_else(|| "Unknown extraction error".into()),
        ));
    }

    Ok(result)
}
