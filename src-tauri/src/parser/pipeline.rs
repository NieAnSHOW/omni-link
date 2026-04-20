use serde::Serialize;
use tauri::{AppHandle, State};

use crate::config::ConfigState;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::repositories::{content_repo, link_repo};

use super::{content_judge, identifier, llm_extractor, webview_extractor};

#[derive(Serialize)]
pub struct ParseResult {
    pub link_id: i64,
    pub platform: String,
    pub content: Option<ContentSummary>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ContentSummary {
    pub title: String,
    pub images_count: usize,
}

pub async fn parse_link(
    app: &AppHandle,
    state: &State<'_, DbState>,
    config_state: &State<'_, ConfigState>,
    link_id: i64,
) -> AppResult<ParseResult> {
    let (url, existing_title) = {
        let conn = state.0.lock().unwrap();
        let link = link_repo::get_link_by_id(&conn, link_id)?
            .ok_or_else(|| AppError::NotFound(format!("Link {} not found", link_id)))?;
        link_repo::update_link_status(&conn, link_id, "parsing")?;
        (link.url, link.title)
    };

    eprintln!("[Pipeline] === link_id={}, url={} ===", link_id, url);

    let platform = identifier::identify_platform(&url).to_string();
    eprintln!("[Pipeline] link_id={}, platform={}", link_id, platform);

    // Step 1: WebView extraction
    eprintln!("[Pipeline] link_id={}, Step 1: WebView extraction starting...", link_id);
    let extract_result = match webview_extractor::extract_via_webview(app, &url).await {
        Ok(result) => {
            eprintln!("[Pipeline] link_id={}, Step 1 OK: title='{}', md={}B, html={}B, images={}",
                link_id, result.title, result.markdown.len(), result.body_html.len(), result.images.len());
            result
        }
        Err(e) => {
            eprintln!("[Pipeline] link_id={}, Step 1 FAILED: {}", link_id, e);
            let conn = state.0.lock().unwrap();
            link_repo::update_link_status(&conn, link_id, "failed")?;
            return Ok(ParseResult {
                link_id,
                platform,
                content: None,
                error: Some(format!("WebView extraction failed: {}", e)),
            });
        }
    };

    let title = if extract_result.title.is_empty() {
        existing_title.as_deref()
    } else {
        Some(extract_result.title.as_str())
    };

    // Step 2: Judge content sufficiency
    let judgement = content_judge::judge_content(
        title.unwrap_or(""),
        &extract_result.markdown,
    );
    eprintln!("[Pipeline] link_id={}, Step 2: sufficient={}, reason='{}'",
        link_id, judgement.is_sufficient, judgement.reason);

    let (final_markdown, final_status) = if judgement.is_sufficient {
        (extract_result.markdown.clone(), "success")
    } else {
        // Step 3: LLM fallback
        eprintln!("[Pipeline] link_id={}, Step 3: LLM fallback starting...", link_id);
        let ai_config = config_state.0.lock().unwrap().ai.clone();
        match llm_extractor::extract_via_llm(&ai_config, &extract_result.raw_html, &url).await {
            Ok(llm_result) => {
                eprintln!("[Pipeline] link_id={}, Step 3 OK: md={}B", link_id, llm_result.markdown.len());
                (llm_result.markdown, "llm_fallback")
            }
            Err(e) => {
                eprintln!("[Pipeline] link_id={}, Step 3 FAILED: {}", link_id, e);
                (extract_result.markdown.clone(), "failed")
            }
        }
    };

    // Step 4: Store results
    eprintln!("[Pipeline] link_id={}, Step 4: storing, status={}", link_id, final_status);
    let meta_value = serde_json::to_value(&extract_result.metadata)?;

    {
        let conn = state.0.lock().unwrap();
        content_repo::create_content(
            &conn,
            link_id,
            title,
            Some(&extract_result.body_html),
            Some(&final_markdown),
            &extract_result.images,
            &meta_value,
            Some(final_status),
        )?;

        if let Some(ref t) = title {
            if !t.is_empty() {
                link_repo::update_link_title(&conn, link_id, t)?;
            }
        }
        link_repo::update_link_status(&conn, link_id, "parsed")?;
    }

    Ok(ParseResult {
        link_id,
        platform,
        content: Some(ContentSummary {
            title: title.unwrap_or("").to_string(),
            images_count: extract_result.images.len(),
        }),
        error: None,
    })
}
