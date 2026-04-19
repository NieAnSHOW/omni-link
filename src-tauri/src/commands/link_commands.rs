use tauri::State;

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
        let mut links = Vec::new();
        for u in url_list {
            let link = link_repo::create_link(&conn, &u, None, src)?;
            links.push(link);
        }
        Ok(links)
    } else if let Some(u) = url {
        let link = link_repo::create_link(&conn, &u, None, src)?;
        Ok(vec![link])
    } else {
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
    let conn = state.0.lock().unwrap();
    let deleted = link_repo::delete_link(&conn, id)?;
    if !deleted {
        return Err(AppError::NotFound("Link not found".into()));
    }
    Ok(true)
}

#[tauri::command]
pub async fn parse_link_cmd(state: State<'_, DbState>, id: i64) -> AppResult<serde_json::Value> {
    let result = pipeline::parse_link(&state, id).await?;
    Ok(serde_json::to_value(result)?)
}
