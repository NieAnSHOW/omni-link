use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::Link;

pub fn create_link(conn: &Connection, url: &str, title: Option<&str>, source: Option<&str>) -> AppResult<Link> {
    let src = source.unwrap_or("manual");
    tracing::debug!("Executing SQL: INSERT INTO links (url={}, title={:?}, source={})", url, title, src);

    conn.execute(
        "INSERT OR IGNORE INTO links (url, title, source) VALUES (?, ?, ?)",
        params![url, title, src],
    ).map_err(|e| {
        tracing::error!("Database error on create_link: {}", e);
        e
    })?;

    if conn.changes() == 0 {
        get_link_by_url(conn, url)?.ok_or_else(|| AppError::Db(rusqlite::Error::QueryReturnedNoRows))
    } else {
        let id = conn.last_insert_rowid();
        get_link_by_id(conn, id)?.ok_or_else(|| AppError::Db(rusqlite::Error::QueryReturnedNoRows))
    }
}

pub fn get_link_by_url(conn: &Connection, url: &str) -> AppResult<Option<Link>> {
    let mut stmt = conn.prepare("SELECT * FROM links WHERE url = ?")?;
    let link = stmt.query_row(params![url], |row| row_to_link(row)).ok();
    Ok(link)
}

pub fn get_link_by_id(conn: &Connection, id: i64) -> AppResult<Option<Link>> {
    let mut stmt = conn.prepare("SELECT * FROM links WHERE id = ?")?;
    let link = stmt.query_row(params![id], |row| row_to_link(row)).ok();
    Ok(link)
}

pub fn get_links(conn: &Connection, limit: i64, offset: i64, status: Option<&str>) -> AppResult<Vec<Link>> {
    tracing::debug!("Executing SQL: SELECT links (limit={}, offset={}, status={:?})", limit, offset, status);

    let mut links = Vec::new();
    if let Some(s) = status {
        let mut stmt = conn.prepare(
            "SELECT * FROM links WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
        ).map_err(|e| {
            tracing::error!("Database error on get_links prepare: {}", e);
            e
        })?;
        let rows = stmt.query_map(params![s, limit, offset], |row| row_to_link(row)).map_err(|e| {
            tracing::error!("Database error on get_links query_map: {}", e);
            e
        })?;
        for row in rows {
            links.push(row.map_err(|e| {
                tracing::error!("Database error on get_links row mapping: {}", e);
                e
            })?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT * FROM links ORDER BY created_at DESC LIMIT ? OFFSET ?",
        ).map_err(|e| {
            tracing::error!("Database error on get_links prepare: {}", e);
            e
        })?;
        let rows = stmt.query_map(params![limit, offset], |row| row_to_link(row)).map_err(|e| {
            tracing::error!("Database error on get_links query_map: {}", e);
            e
        })?;
        for row in rows {
            links.push(row.map_err(|e| {
                tracing::error!("Database error on get_links row mapping: {}", e);
                e
            })?);
        }
    }
    Ok(links)
}

pub fn update_link_status(conn: &Connection, id: i64, status: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE links SET status = ?, updated_at = datetime('now') WHERE id = ?",
        params![status, id],
    )?;
    Ok(())
}

pub fn update_link_title(conn: &Connection, id: i64, title: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE links SET title = ?, updated_at = datetime('now') WHERE id = ?",
        params![title, id],
    )?;
    Ok(())
}

pub fn delete_link(conn: &Connection, id: i64) -> AppResult<bool> {
    tracing::debug!("Executing SQL: DELETE FROM links WHERE id={}", id);
    conn.execute("DELETE FROM links WHERE id = ?", params![id]).map_err(|e| {
        tracing::error!("Database error on delete_link: {}", e);
        e
    })?;
    Ok(conn.changes() > 0)
}

pub fn get_links_count(conn: &Connection, status: Option<&str>) -> AppResult<i64> {
    let count: i64 = if let Some(s) = status {
        conn.query_row(
            "SELECT COUNT(*) FROM links WHERE status = ?",
            params![s],
            |row| row.get(0),
        )?
    } else {
        conn.query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))?
    };
    Ok(count)
}

fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<Link> {
    Ok(Link {
        id: row.get("id")?,
        url: row.get("url")?,
        title: row.get("title")?,
        platform: row.get("platform")?,
        source: row.get("source")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}
