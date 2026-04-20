use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::Link;

pub fn create_link(conn: &Connection, url: &str, title: Option<&str>, source: Option<&str>) -> AppResult<Link> {
    let src = source.unwrap_or("manual");
    conn.execute(
        "INSERT OR IGNORE INTO links (url, title, source) VALUES (?, ?, ?)",
        params![url, title, src],
    )?;

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

pub fn get_links(conn: &Connection, limit: i64, offset: i64, status: Option<&str>, category_id: Option<i64>) -> AppResult<Vec<Link>> {
    let mut links = Vec::new();
    match (status, category_id) {
        (Some(s), Some(c)) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links WHERE status = ? AND category_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![s, c, limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
        (Some(s), None) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![s, limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
        (None, Some(c)) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links WHERE category_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![c, limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
        (None, None) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
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
    conn.execute("DELETE FROM links WHERE id = ?", params![id])?;
    Ok(conn.changes() > 0)
}

pub fn get_links_count(conn: &Connection, status: Option<&str>, category_id: Option<i64>) -> AppResult<i64> {
    let count: i64 = match (status, category_id) {
        (Some(s), Some(c)) => conn.query_row(
            "SELECT COUNT(*) FROM links WHERE status = ? AND category_id = ?",
            params![s, c],
            |row| row.get(0),
        )?,
        (Some(s), None) => conn.query_row(
            "SELECT COUNT(*) FROM links WHERE status = ?",
            params![s],
            |row| row.get(0),
        )?,
        (None, Some(c)) => conn.query_row(
            "SELECT COUNT(*) FROM links WHERE category_id = ?",
            params![c],
            |row| row.get(0),
        )?,
        (None, None) => conn.query_row(
            "SELECT COUNT(*) FROM links",
            [],
            |row| row.get(0),
        )?,
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
        category_id: row.get("category_id")?,
    })
}

pub fn update_link_category(conn: &Connection, id: i64, category_id: Option<i64>) -> AppResult<()> {
    conn.execute(
        "UPDATE links SET category_id = ?, updated_at = datetime('now') WHERE id = ?",
        params![category_id, id],
    )?;
    Ok(())
}

pub fn get_links_by_category(conn: &Connection, category_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Link>> {
    let mut links = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT * FROM links WHERE category_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )?;
    let rows = stmt.query_map(params![category_id, limit, offset], |row| row_to_link(row))?;
    for row in rows {
        links.push(row?);
    }
    Ok(links)
}
