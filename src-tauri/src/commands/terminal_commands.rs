use std::sync::Mutex;

use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::terminal_session::{CreateTerminalSession, TerminalSession};
use crate::repositories::terminal_session_repo;
use crate::terminal::PtyManager;

pub struct PtyManagerState(pub Mutex<PtyManager>);

#[tauri::command]
pub async fn start_persona_rewrite(
    note_id: i64,
    note_path: String,
    persona_skill: String,
    mode: String,
    db: State<'_, DbState>,
    pty_manager: State<'_, PtyManagerState>,
) -> AppResult<String> {
    let conn = db.0.lock().unwrap();

    let create_session = CreateTerminalSession {
        note_id,
        persona_skill: persona_skill.clone(),
        mode: mode.clone(),
    };

    let session_id = terminal_session_repo::insert_terminal_session(&conn, &create_session)?;

    // Release DB lock before acquiring PTY lock to avoid potential deadlocks
    drop(conn);

    let manager = pty_manager.0.lock().unwrap();
    manager.create_session(
        session_id.clone(),
        note_id,
        &note_path,
        &persona_skill,
    )?;

    Ok(session_id)
}

#[tauri::command]
pub async fn update_session_status(
    session_id: String,
    status: String,
    db: State<'_, DbState>,
) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    terminal_session_repo::update_session_status(&conn, &session_id, &status)
}

#[tauri::command]
pub async fn close_terminal_session(
    session_id: String,
    pty_manager: State<'_, PtyManagerState>,
) -> AppResult<()> {
    let manager = pty_manager.0.lock().unwrap();
    manager.close_session(&session_id)
}

#[tauri::command]
pub async fn resize_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    pty_manager: State<'_, PtyManagerState>,
) -> AppResult<()> {
    let manager = pty_manager.0.lock().unwrap();
    manager.resize_session(&session_id, rows, cols)
}

#[tauri::command]
pub async fn get_session_info(
    session_id: String,
    db: State<'_, DbState>,
) -> AppResult<Option<TerminalSession>> {
    let conn = db.0.lock().unwrap();
    terminal_session_repo::get_session_by_id(&conn, &session_id)
}
