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
) -> AppResult<ContentRow> {
    let images_json = serde_json::to_string(images)?;
    let metadata_json = serde_json::to_string(metadata)?;

    conn.execute(
        "INSERT INTO contents (link_id, title, body_html, body_text, images, metadata) VALUES (?, ?, ?, ?, ?, ?)",
        params![link_id, title, body_html, body_text, images_json, metadata_json],
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

fn row_to_content(row: &rusqlite::Row) -> rusqlite::Result<ContentRow> {
    Ok(ContentRow {
        id: row.get("id")?,
        link_id: row.get("link_id")?,
        title: row.get("title")?,
        body_html: row.get("body_html")?,
        body_text: row.get("body_text")?,
        images: row.get("images")?,
        metadata: row.get("metadata")?,
        created_at: row.get("created_at")?,
    })
}
