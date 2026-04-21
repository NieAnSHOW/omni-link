use tauri::State;

use crate::ai::ai_service;
use crate::config::ConfigState;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::{AiResultParsed, ContentParsed, LinkDetail, UpdateContentInput};
use crate::repositories::{ai_result_repo, content_repo, link_repo, tag_repo};

#[tauri::command]
pub async fn get_link_detail(state: State<'_, DbState>, id: i64) -> AppResult<LinkDetail> {
    let conn = state.0.lock().unwrap();

    let link = link_repo::get_link_by_id(&conn, id)?
        .ok_or_else(|| AppError::NotFound("Link not found".into()))?;

    let content = content_repo::get_content_by_link_id(&conn, id)?.map(|c| {
        let images: Vec<String> = serde_json::from_str(&c.images).unwrap_or_default();
        let metadata: serde_json::Value = serde_json::from_str(&c.metadata).unwrap_or(serde_json::json!({}));
        ContentParsed {
            id: c.id,
            link_id: c.link_id,
            title: c.title,
            body_html: c.body_html,
            body_text: c.body_text,
            images,
            metadata,
            content_status: c.content_status,
            created_at: c.created_at,
        }
    });

    let ai = content.as_ref().and_then(|c| {
        ai_result_repo::get_ai_result_by_content_id(&conn, c.id)
            .ok()
            .flatten()
            .map(|r| {
                let tags: Vec<String> = serde_json::from_str(&r.tags).unwrap_or_default();
                AiResultParsed {
                    summary: r.summary.unwrap_or_default(),
                    tags,
                    provider: r.provider,
                }
            })
    });

    Ok(LinkDetail { link, content, ai, tags: None })
}

#[tauri::command]
pub async fn analyze_content_cmd(
    state: State<'_, DbState>,
    config_state: State<'_, ConfigState>,
    content_id: i64,
) -> AppResult<serde_json::Value> {
    let (body_text, title) = {
        let conn = state.0.lock().unwrap();
        let content = content_repo::get_content_by_id(&conn, content_id)?
            .ok_or_else(|| AppError::NotFound("Content not found".into()))?;
        (content.body_text.unwrap_or_default(), content.title)
    };

    let config = config_state.0.lock().unwrap().clone();
    let result = ai_service::analyze_content(&config.ai, &body_text, title.as_deref()).await?;

    let summary = result["summary"].as_str().unwrap_or("").to_string();
    let tags: Vec<String> = result["tags"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let conn = state.0.lock().unwrap();

    // AI 结果入库
    ai_result_repo::create_ai_result(
        &conn,
        content_id,
        Some(&summary),
        &tags,
        Some(&config.ai.provider),
    )?;

    // 标签自动入库 + 关联
    let tag_colors = [
        "#50fa7b", "#8be9fd", "#ff79c6", "#f1fa8c",
        "#ffb86c", "#bd93f9", "#ff5555", "#6272a4",
    ];
    let mut tag_ids = Vec::new();
    for (i, tag_name) in tags.iter().enumerate() {
        let color = tag_colors[i % tag_colors.len()];
        let tag_id = tag_repo::ensure_tag(&conn, tag_name, color)?;
        tag_ids.push(tag_id);
    }
    tag_repo::set_content_tags(&conn, content_id, &tag_ids)?;

    Ok(result)
}

#[tauri::command]
pub async fn update_content_cmd(state: State<'_, DbState>, input: UpdateContentInput) -> AppResult<bool> {
    let conn = state.0.lock().unwrap();
    content_repo::update_content(
        &conn,
        input.id,
        input.title.as_deref(),
        input.body_text.as_deref(),
        input.body_html.as_deref(),
    )?;
    Ok(true)
}

#[tauri::command]
pub async fn ai_process_content_cmd(
    state: State<'_, DbState>,
    config_state: State<'_, ConfigState>,
    content_id: i64,
    mode: String,
) -> AppResult<bool> {
    let (body_text, title) = {
        let conn = state.0.lock().unwrap();
        let content = content_repo::get_content_by_id(&conn, content_id)?
            .ok_or_else(|| AppError::NotFound("Content not found".into()))?;
        (content.body_text.unwrap_or_default(), content.title)
    };

    let config = config_state.0.lock().unwrap().clone();

    let result = match mode.as_str() {
        "organize" => ai_service::organize_content(&config.ai, &body_text).await?,
        "expand" => ai_service::expand_content(&config.ai, &body_text, title.as_deref()).await?,
        "both" => {
            let organized = ai_service::organize_content(&config.ai, &body_text).await?;
            let organized_text = organized["body_text"].as_str().unwrap_or(&body_text).to_string();
            ai_service::expand_content(&config.ai, &organized_text, title.as_deref()).await?
        }
        _ => return Err(AppError::Ai(format!("Unknown mode: {}", mode))),
    };

    let new_body_text = result["body_text"].as_str().unwrap_or("").to_string();
    if new_body_text.is_empty() {
        return Ok(false);
    }

    let conn = state.0.lock().unwrap();
    content_repo::update_content(
        &conn,
        content_id,
        None,
        Some(&new_body_text),
        None,
    )?;

    Ok(true)
}
