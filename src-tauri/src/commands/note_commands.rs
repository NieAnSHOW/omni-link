use std::fs;
use std::path::Path;

use tauri::State;

use crate::config::ConfigState;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::{Note, NoteDetail};
use crate::parser::pipeline;
use crate::repositories::note_repo;

/// Sanitize a string for use as a file name by replacing illegal characters with `_`.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if "/\\:*?\"<>|".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect()
}

#[tauri::command]
pub async fn create_note(state: State<'_, DbState>, title: String) -> AppResult<Note> {
    let conn = state.0.lock().unwrap();
    note_repo::create_note(&conn, &title)
}

#[tauri::command]
pub async fn get_note(state: State<'_, DbState>, id: i64) -> AppResult<NoteDetail> {
    let conn = state.0.lock().unwrap();
    let note = note_repo::get_note_by_id(&conn, id)?
        .ok_or_else(|| crate::error::AppError::NotFound("Note not found".into()))?;
    let content = note_repo::read_note_content(&conn, id)?;
    Ok(NoteDetail { note, content })
}

#[tauri::command]
pub async fn list_notes(
    state: State<'_, DbState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<Vec<Note>> {
    let conn = state.0.lock().unwrap();
    let lim = limit.unwrap_or(50);
    let off = offset.unwrap_or(0);
    note_repo::list_notes(&conn, lim, off)
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, DbState>,
    id: i64,
    title: String,
    content: String,
) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    note_repo::update_note(&conn, id, &title, &content)
}

#[tauri::command]
pub async fn delete_note(state: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    note_repo::delete_note(&conn, id)
}

#[tauri::command]
pub async fn create_note_from_link(
    app: tauri::AppHandle,
    config_state: State<'_, ConfigState>,
    url: String,
) -> AppResult<String> {
    let parse_output = pipeline::parse_url_to_markdown(&app, &config_state, &url).await?;

    let workspace_path = {
        let cfg = config_state.0.lock().unwrap();
        cfg.workspace_path
            .clone()
            .ok_or_else(|| AppError::Config("No workspace path configured".into()))?
    };

    let file_name = format!("{}.md", sanitize_filename(&parse_output.title));
    let file_path = Path::new(&workspace_path).join(&file_name);

    fs::write(&file_path, &parse_output.markdown)?;

    Ok(file_path.to_str().unwrap_or(&file_name).to_string())
}
