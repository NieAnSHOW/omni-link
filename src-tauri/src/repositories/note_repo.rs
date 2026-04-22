use rusqlite::{params, Connection, Row};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::models::Note;

fn row_to_note(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        file_name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        file_size: row.get(5)?,
        word_count: row.get(6)?,
    })
}

fn get_notes_dir(conn: &Connection) -> AppResult<PathBuf> {
    let mut stmt = conn.prepare("SELECT value FROM user_settings WHERE key = 'notes_storage_path'")?;
    let path_str: String = stmt.query_row([], |row| row.get(0))?;
    let expanded = shellexpand::tilde(&path_str).to_string();
    Ok(PathBuf::from(expanded))
}

fn validate_filename(filename: &str) -> AppResult<()> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::Parse(format!("Invalid filename: {}", filename)));
    }
    Ok(())
}

fn safe_join(base: &Path, filename: &str) -> AppResult<PathBuf> {
    validate_filename(filename)?;
    Ok(base.join(filename))
}

pub fn create_note(conn: &Connection, title: &str) -> AppResult<Note> {
    let timestamp = chrono::Utc::now().timestamp();
    let temp_file_name = format!("note_{}_temp.md", timestamp);

    conn.execute(
        "INSERT INTO notes (title, file_name) VALUES (?, ?)",
        params![title, temp_file_name],
    )?;

    let id = conn.last_insert_rowid();
    let file_name = format!("note_{}_{}.md", timestamp, id);

    conn.execute(
        "UPDATE notes SET file_name = ? WHERE id = ?",
        params![file_name, id],
    )?;

    let notes_dir = get_notes_dir(conn)?;
    fs::create_dir_all(&notes_dir)?;
    let file_path = safe_join(&notes_dir, &file_name)?;
    fs::write(&file_path, "")?;

    get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))
}

pub fn get_note_by_id(conn: &Connection, id: i64) -> AppResult<Option<Note>> {
    let mut stmt = conn.prepare("SELECT * FROM notes WHERE id = ?")?;
    let note = stmt.query_row(params![id], |row| row_to_note(row)).ok();
    Ok(note)
}

pub fn list_notes(conn: &Connection, limit: i64, offset: i64) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM notes ORDER BY updated_at DESC LIMIT ? OFFSET ?",
    )?;
    let rows = stmt.query_map(params![limit, offset], |row| row_to_note(row))?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row?);
    }
    Ok(notes)
}

pub fn read_note_content(conn: &Connection, id: i64) -> AppResult<String> {
    let note = get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))?;
    let notes_dir = get_notes_dir(conn)?;
    let file_path = safe_join(&notes_dir, &note.file_name)?;

    if !file_path.exists() {
        return Err(AppError::NotFound(format!("Note file not found: {}", note.file_name)));
    }

    let content = fs::read_to_string(&file_path)?;
    Ok(content)
}

pub fn update_note(conn: &Connection, id: i64, title: &str, content: &str) -> AppResult<()> {
    let note = get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))?;
    let notes_dir = get_notes_dir(conn)?;
    let file_path = safe_join(&notes_dir, &note.file_name)?;

    fs::write(&file_path, content)?;

    let file_size = content.len() as i64;
    let word_count = content.split_whitespace().count() as i64;

    conn.execute(
        "UPDATE notes SET title = ?, updated_at = datetime('now'), file_size = ?, word_count = ? WHERE id = ?",
        params![title, file_size, word_count, id],
    )?;

    Ok(())
}

pub fn delete_note(conn: &Connection, id: i64) -> AppResult<()> {
    let note = get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))?;
    let notes_dir = get_notes_dir(conn)?;
    let file_path = safe_join(&notes_dir, &note.file_name)?;

    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }

    let images_base = notes_dir.join("images");
    let images_dir = safe_join(&images_base, &id.to_string())?;
    if images_dir.exists() {
        fs::remove_dir_all(&images_dir)?;
    }

    conn.execute("DELETE FROM notes WHERE id = ?", params![id])?;

    Ok(())
}
