use serde::Serialize;
use tauri::{AppHandle, State};

use crate::config::ConfigState;
use crate::error::{AppError, AppResult};

use super::{content_judge, identifier, llm_extractor, webview_extractor};

#[derive(Serialize)]
pub struct ParseOutput {
    pub url: String,
    pub title: String,
    pub markdown: String,
    pub platform: String,
}

pub async fn parse_url_to_markdown(
    app: &AppHandle,
    config_state: &State<'_, ConfigState>,
    url: &str,
) -> AppResult<ParseOutput> {
    tracing::info!("Starting parse for url={}", url);

    let platform = identifier::identify_platform(url).to_string();

    // Step 1: WebView extraction
    let extract_result = webview_extractor::extract_via_webview(app, url).await
        .map_err(|e| {
            tracing::error!("Content extraction failed: url={}, error={}", url, e);
            AppError::Parse(format!("内容提取失败: {}", e))
        })?;

    let title = if extract_result.title.is_empty() {
        url.to_string()
    } else {
        extract_result.title.clone()
    };

    // Step 2: Judge content sufficiency
    let judgement = content_judge::judge_content(&title, &extract_result.markdown);

    let final_markdown = if judgement.is_sufficient {
        extract_result.markdown.clone()
    } else {
        // Step 3: LLM fallback
        tracing::warn!("Content insufficient, falling back to LLM: reason='{}'", judgement.reason);
        let ai_config = config_state.0.lock().unwrap().ai.clone();
        match llm_extractor::extract_via_llm(&ai_config, &extract_result.raw_html, url).await {
            Ok(llm_result) => llm_result.markdown,
            Err(e) => {
                tracing::error!("LLM extraction failed: error={}", e);
                extract_result.markdown.clone()
            }
        }
    };

    Ok(ParseOutput {
        url: url.to_string(),
        title,
        markdown: final_markdown,
        platform,
    })
}
