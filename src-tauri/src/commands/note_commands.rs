use tauri::State;

use crate::config::ConfigState;
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::{Note, NoteDetail};
use crate::parser::pipeline;
use crate::repositories::note_repo;

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
    state: State<'_, DbState>,
    config_state: State<'_, ConfigState>,
    url: String,
) -> AppResult<NoteDetail> {
    let parse_output = pipeline::parse_url_to_markdown(&app, &config_state, &url).await?;

    let conn = state.0.lock().unwrap();
    let note = note_repo::create_note_with_content(
        &conn,
        &parse_output.title,
        &parse_output.markdown,
        &parse_output.url,
    )?;

    let content = note_repo::read_note_content(&conn, note.id)?;
    Ok(NoteDetail { note, content })
}
