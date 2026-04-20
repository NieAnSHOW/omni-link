use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Listener, Manager, WebviewUrl};
use tauri::webview::{PageLoadEvent, WebviewWindowBuilder};

use crate::error::{AppError, AppResult};

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
    let (tx, rx) = mpsc::channel::<WebviewExtractResult>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let label = format!("extractor-{}", std::process::id());

    let parsed_url: url::Url = url
        .parse()
        .map_err(|e: url::ParseError| AppError::Parse(e.to_string()))?;

    let tx_clone = Arc::clone(&tx);
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(parsed_url))
        .title("Content Extractor")
        .visible(false)
        .inner_size(1280.0, 800.0)
        .on_page_load(move |webview_window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let resources_dir = webview_window.app_handle().path().resource_dir();
                let js_dir = match resources_dir {
                    Ok(dir) => dir.join("js"),
                    Err(_) => return,
                };

                let readability_js =
                    std::fs::read_to_string(js_dir.join("readability.js")).unwrap_or_default();
                let turndown_js =
                    std::fs::read_to_string(js_dir.join("turndown.js")).unwrap_or_default();
                let extract_js =
                    std::fs::read_to_string(js_dir.join("extract.js")).unwrap_or_default();

                let combined = format!(
                    "(()=>{{{};{};{};}})();",
                    readability_js, turndown_js, extract_js
                );

                let _ = webview_window.eval(&combined);
            }
        })
        .build()
        .map_err(|e| AppError::Parse(format!("Failed to create webview: {}", e)))?;

    // Listen for extraction result
    let app_clone = app.clone();
    let label_clone = label.clone();
    app.listen("extract-result", move |event| {
        if let Ok(result) = serde_json::from_str::<WebviewExtractResult>(event.payload()) {
            if let Ok(mut guard) = tx_clone.lock() {
                if let Some(tx) = guard.take() {
                    let _ = tx.send(result);
                }
            }
            // Destroy the webview window
            if let Some(w) = app_clone.get_webview_window(&label_clone) {
                let _ = w.destroy();
            }
        }
    });

    // Wait for result with timeout
    let result = rx
        .recv_timeout(Duration::from_secs(20))
        .map_err(|_| {
            let _ = window.destroy();
            AppError::Parse("Webview extraction timed out".into())
        })?;

    if !result.success {
        return Err(AppError::Parse(
            result
                .error
                .unwrap_or_else(|| "Unknown extraction error".into()),
        ));
    }

    Ok(result)
}
