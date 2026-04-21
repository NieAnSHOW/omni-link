use rusqlite::params;
use tauri::{Emitter, State};

use crate::ai::ai_service;
use crate::config::ConfigState;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::{Link, LinksResponse};
use crate::parser::pipeline;
use crate::repositories::link_repo;

#[tauri::command]
pub async fn get_links(
    state: State<'_, DbState>,
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<String>,
) -> AppResult<LinksResponse> {
    tracing::debug!("get_links called: limit={:?}, offset={:?}, status={:?}", limit, offset, status);
    let conn = state.0.lock().unwrap();
    let lim = limit.unwrap_or(20);
    let off = offset.unwrap_or(0);
    let links = link_repo::get_links(&conn, lim, off, status.as_deref())?;
    let total = link_repo::get_links_count(&conn, status.as_deref())?;
    Ok(LinksResponse { links, total })
}

#[tauri::command]
pub async fn create_link(
    state: State<'_, DbState>,
    url: Option<String>,
    urls: Option<Vec<String>>,
    source: Option<String>,
) -> AppResult<Vec<Link>> {
    let conn = state.0.lock().unwrap();
    let src = source.as_deref();

    if let Some(url_list) = urls {
        tracing::info!("Creating {} links from batch, source={:?}", url_list.len(), src);
        let mut links = Vec::new();
        for u in url_list {
            match link_repo::create_link(&conn, &u, None, src) {
                Ok(link) => {
                    tracing::info!("Link created: id={}, url={}", link.id, link.url);
                    links.push(link);
                }
                Err(e) => {
                    tracing::error!("Failed to create link url={}: {}", u, e);
                    return Err(e);
                }
            }
        }
        Ok(links)
    } else if let Some(u) = url {
        tracing::info!("Creating single link: url={}, source={:?}", u, src);
        match link_repo::create_link(&conn, &u, None, src) {
            Ok(link) => {
                tracing::info!("Link created: id={}, url={}", link.id, link.url);
                Ok(vec![link])
            }
            Err(e) => {
                tracing::error!("Failed to create link url={}: {}", u, e);
                Err(e)
            }
        }
    } else {
        tracing::error!("create_link called without url or urls parameter");
        Err(AppError::Parse("url or urls is required".into()))
    }
}

#[tauri::command]
pub async fn get_link(state: State<'_, DbState>, id: i64) -> AppResult<Link> {
    let conn = state.0.lock().unwrap();
    link_repo::get_link_by_id(&conn, id)?
        .ok_or_else(|| AppError::NotFound("Link not found".into()))
}

#[tauri::command]
pub async fn delete_link(state: State<'_, DbState>, id: i64) -> AppResult<bool> {
    tracing::info!("Deleting link: id={}", id);
    let conn = state.0.lock().unwrap();
    match link_repo::delete_link(&conn, id) {
        Ok(deleted) => {
            if !deleted {
                tracing::error!("Link not found for deletion: id={}", id);
                return Err(AppError::NotFound("Link not found".into()));
            }
            tracing::info!("Link deleted successfully: id={}", id);
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to delete link id={}: {}", id, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn parse_link_cmd(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    config: State<'_, ConfigState>,
    id: i64,
) -> AppResult<serde_json::Value> {
    eprintln!("[CMD] parse_link_cmd invoked, id={}", id);
    let result = pipeline::parse_link(&app, &state, &config, id).await?;
    eprintln!("[CMD] parse_link_cmd done, id={}, error={:?}", id, result.error);
    Ok(serde_json::to_value(result)?)
}

#[tauri::command]
pub async fn start_ai_process(
    link_id: i64,
    mode: String,
    state: State<'_, DbState>,
    config: State<'_, ConfigState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use crate::repositories::content_repo;

    // 1. 更新 ai_processing_status
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        link_repo::update_ai_processing_status(&conn, link_id, &mode)
            .map_err(|e| e.to_string())?;
    }

    // 2. 异步执行 AI 处理
    let db_mutex = state.0.clone();
    let config_clone = config.0.lock().map_err(|e| e.to_string())?.clone();
    tauri::async_runtime::spawn(async move {
        // 获取 content_id
        let content_id_result = {
            let conn = db_mutex.lock().unwrap();
            conn.query_row(
                "SELECT id FROM contents WHERE link_id = ?1",
                params![link_id],
                |row| row.get::<_, i64>(0),
            )
        };

        if let Ok(content_id) = content_id_result {
            // 获取正文内容
            let (body_text, title) = {
                let conn = db_mutex.lock().unwrap();
                match content_repo::get_content_by_id(&conn, content_id) {
                    Ok(Some(c)) => (c.body_text.unwrap_or_default(), c.title),
                    _ => {
                        {
                            let conn = db_mutex.lock().unwrap();
                            let _ = link_repo::update_ai_processing_status(&conn, link_id, "idle");
                        }
                        let _ = app_handle.emit("ai-process-failed", link_id);
                        return;
                    }
                }
            };

            // 执行 AI 处理
            let result: Result<serde_json::Value, _> = match mode.as_str() {
                "analyze" => ai_service::analyze_content(&config_clone.ai, &body_text, title.as_deref()).await,
                "organize" => ai_service::organize_content(&config_clone.ai, &body_text).await,
                "expand" => ai_service::expand_content(&config_clone.ai, &body_text, title.as_deref()).await,
                _ => {
                    Err(crate::error::AppError::Ai(format!("Unknown mode: {}", mode)).into())
                }
            };

            // 3. 处理完成，重置状态
            {
                let conn = db_mutex.lock().unwrap();
                let _ = link_repo::update_ai_processing_status(&conn, link_id, "idle");
            }

            // 4. 发送前端事件
            if result.is_ok() {
                let _ = app_handle.emit("ai-process-complete", link_id);
            } else {
                let _ = app_handle.emit("ai-process-failed", link_id);
            }
        } else {
            // 未找到 content，重置状态
            {
                let conn = db_mutex.lock().unwrap();
                let _ = link_repo::update_ai_processing_status(&conn, link_id, "idle");
            }
            let _ = app_handle.emit("ai-process-failed", link_id);
        }
    });

    Ok(())
}
