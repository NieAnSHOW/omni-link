use rusqlite::{params, Connection};

use crate::error::AppResult;

pub struct ContentRow {
    pub id: i64,
    pub link_id: i64,
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub images: String,
    pub metadata: String,
    pub content_status: Option<String>,
    pub created_at: String,
}

pub fn create_content(
    conn: &Connection,
    link_id: i64,
    title: Option<&str>,
    body_html: Option<&str>,
    body_text: Option<&str>,
    images: &[String],
    metadata: &serde_json::Value,
    content_status: Option<&str>,
) -> AppResult<ContentRow> {
    let images_json = serde_json::to_string(images)?;
    let metadata_json = serde_json::to_string(metadata)?;

    conn.execute("DELETE FROM contents WHERE link_id = ?", params![link_id])?;
    conn.execute(
        "INSERT INTO contents (link_id, title, body_html, body_text, images, metadata, content_status) VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![link_id, title, body_html, body_text, images_json, metadata_json, content_status],
    )?;

    let id = conn.last_insert_rowid();
    get_content_by_id(conn, id)?.ok_or_else(|| crate::error::AppError::Db(rusqlite::Error::QueryReturnedNoRows))
}

pub fn get_content_by_link_id(conn: &Connection, link_id: i64) -> AppResult<Option<ContentRow>> {
    let mut stmt = conn.prepare("SELECT * FROM contents WHERE link_id = ?")?;
    let content = stmt.query_row(params![link_id], |row| row_to_content(row)).ok();
    Ok(content)
}

pub fn get_content_by_id(conn: &Connection, id: i64) -> AppResult<Option<ContentRow>> {
    let mut stmt = conn.prepare("SELECT * FROM contents WHERE id = ?")?;
    let content = stmt.query_row(params![id], |row| row_to_content(row)).ok();
    Ok(content)
}

pub fn update_content(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    body_text: Option<&str>,
    body_html: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE contents SET title = ?, body_text = ?, body_html = ?, updated_at = datetime('now') WHERE id = ?",
        params![title, body_text, body_html, id],
    )?;
    Ok(())
}

fn row_to_content(row: &rusqlite::Row) -> rusqlite::Result<ContentRow> {
    Ok(ContentRow {
        id: row.get("id")?,
        link_id: row.get("link_id")?,
        title: row.get("title")?,
        body_html: row.get("body_html")?,
        body_text: row.get("body_text")?,
        images: row.get("images")?,
        metadata: row.get("metadata")?,
        content_status: row.get("content_status")?,
        created_at: row.get("created_at")?,
    })
}
