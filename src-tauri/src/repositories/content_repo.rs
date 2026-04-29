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

pub struct CreateContentParams<'a> {
    pub link_id: i64,
    pub title: Option<&'a str>,
    pub body_html: Option<&'a str>,
    pub body_text: Option<&'a str>,
    pub images: &'a [String],
    pub metadata: &'a serde_json::Value,
    pub content_status: Option<&'a str>,
}

pub fn create_content(
    conn: &Connection,
    params: CreateContentParams,
) -> AppResult<ContentRow> {
    let images_json = serde_json::to_string(params.images)?;
    let metadata_json = serde_json::to_string(params.metadata)?;

    conn.execute("DELETE FROM contents WHERE link_id = ?", rusqlite::params![params.link_id])?;
    conn.execute(
        "INSERT INTO contents (link_id, title, body_html, body_text, images, metadata, content_status) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![params.link_id, params.title, params.body_html, params.body_text, images_json, metadata_json, params.content_status],
    )?;

    let id = conn.last_insert_rowid();
    get_content_by_id(conn, id)?.ok_or_else(|| crate::error::AppError::Db(rusqlite::Error::QueryReturnedNoRows))
}

pub fn get_content_by_link_id(conn: &Connection, link_id: i64) -> AppResult<Option<ContentRow>> {
    let mut stmt = conn.prepare("SELECT * FROM contents WHERE link_id = ?")?;
    let content = stmt.query_row(rusqlite::params![link_id], row_to_content).ok();
    Ok(content)
}

pub fn get_content_by_id(conn: &Connection, id: i64) -> AppResult<Option<ContentRow>> {
    let mut stmt = conn.prepare("SELECT * FROM contents WHERE id = ?")?;
    let content = stmt.query_row(rusqlite::params![id], row_to_content).ok();
    Ok(content)
}

pub fn update_content(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    body_text: Option<&str>,
    body_html: Option<&str>,
) -> AppResult<()> {
    let rows = conn.execute(
        "UPDATE contents SET title = ?, body_text = ?, body_html = ?, updated_at = datetime('now') WHERE id = ?",
        params![title, body_text, body_html, id],
    )?;
    if rows == 0 {
        return Err(crate::error::AppError::NotFound(format!("Content id {} not found", id)));
    }
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
