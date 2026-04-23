use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::terminal_session::{CreateTerminalSession, TerminalSession};

pub fn insert_terminal_session(
    conn: &Connection,
    session: &CreateTerminalSession,
) -> AppResult<String> {
    let id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO terminal_sessions (id, note_id, persona_skill, mode, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'running', datetime('now'))",
        params![id, session.note_id, session.persona_skill, session.mode],
    )?;

    Ok(id)
}

pub fn update_session_status(conn: &Connection, session_id: &str, status: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE terminal_sessions SET status = ?1 WHERE id = ?2",
        params![status, session_id],
    )?;
    Ok(())
}

pub fn get_session_by_id(
    conn: &Connection,
    session_id: &str,
) -> AppResult<Option<TerminalSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, note_id, persona_skill, mode, status, created_at
         FROM terminal_sessions
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![session_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(TerminalSession {
            id: row.get(0)?,
            note_id: row.get(1)?,
            persona_skill: row.get(2)?,
            mode: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}
