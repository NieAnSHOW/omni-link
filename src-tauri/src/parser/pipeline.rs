use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::repositories::{content_repo, link_repo};

use super::{extractor, fetcher, identifier};

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

pub async fn parse_link(state: &State<'_, DbState>, link_id: i64) -> AppResult<ParseResult> {
    let (url, existing_title) = {
        let conn = state.0.lock().unwrap();
        let link = link_repo::get_link_by_id(&conn, link_id)?
            .ok_or_else(|| AppError::NotFound(format!("Link {} not found", link_id)))?;
        link_repo::update_link_status(&conn, link_id, "parsing")?;
        (link.url, link.title)
    };

    let platform = identifier::identify_platform(&url).to_string();

    let result = match fetcher::fetch_page(&url).await {
        Ok(fetch_result) => {
            let content = extractor::extract_content(&fetch_result.html, &fetch_result.url)?;

            let title_to_use = if content.title.is_empty() {
                existing_title.as_deref()
            } else {
                Some(content.title.as_str())
            };

            let meta_value = serde_json::to_value(&content.metadata)?;
            let images_json = serde_json::to_string(&content.images)?;
            let meta_json = serde_json::to_string(&meta_value)?;

            let conn = state.0.lock().unwrap();
            conn.execute(
                "INSERT INTO contents (link_id, title, body_html, body_text, images, metadata) VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![link_id, title_to_use, content.body_html, content.body_text, images_json, meta_json],
            )?;

            if let Some(ref title) = title_to_use {
                if !title.is_empty() {
                    link_repo::update_link_title(&conn, link_id, title)?;
                }
            }
            link_repo::update_link_status(&conn, link_id, "parsed")?;

            ParseResult {
                link_id,
                platform,
                content: Some(ContentSummary {
                    title: content.title.clone(),
                    images_count: content.images.len(),
                }),
                error: None,
            }
        }
        Err(e) => {
            let conn = state.0.lock().unwrap();
            link_repo::update_link_status(&conn, link_id, "failed")?;
            ParseResult {
                link_id,
                platform,
                content: None,
                error: Some(e.to_string()),
            }
        }
    };

    Ok(result)
}
