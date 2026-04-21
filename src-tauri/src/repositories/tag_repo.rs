use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::{Tag, TagWithCount};

pub fn create_tag(conn: &Connection, name: &str, color: &str, tag_type: &str) -> AppResult<Tag> {
    conn.execute(
        "INSERT INTO tags (name, color, type) VALUES (?, ?, ?)",
        params![name, color, tag_type],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: name.to_string(),
        color: color.to_string(),
        tag_type: tag_type.to_string(),
        created_at: String::new(),
    })
}

pub fn get_all_tags(conn: &Connection) -> AppResult<Vec<Tag>> {
    let mut tags = Vec::new();
    let mut stmt = conn.prepare("SELECT * FROM tags ORDER BY created_at")?;
    let rows = stmt.query_map([], |row| Ok(Tag {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        tag_type: row.get("type")?,
        created_at: row.get("created_at")?,
    }))?;
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn get_tags_with_count(conn: &Connection) -> AppResult<Vec<TagWithCount>> {
    let mut tags = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT t.*, COUNT(ct.content_id) as content_count
         FROM tags t
         LEFT JOIN content_tags ct ON t.id = ct.tag_id
         GROUP BY t.id
         ORDER BY t.created_at"
    )?;
    let rows = stmt.query_map([], |row| Ok(TagWithCount {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        tag_type: row.get("type")?,
        content_count: row.get("content_count")?,
        created_at: row.get("created_at")?,
    }))?;
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn get_tag_by_name(conn: &Connection, name: &str) -> AppResult<Option<Tag>> {
    let mut stmt = conn.prepare("SELECT * FROM tags WHERE name = ?")?;
    let tag = stmt.query_row(params![name], |row| Ok(Tag {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        tag_type: row.get("type")?,
        created_at: row.get("created_at")?,
    })).ok();
    Ok(tag)
}

pub fn delete_tag(conn: &Connection, id: i64) -> AppResult<bool> {
    conn.execute("DELETE FROM tags WHERE id = ?", params![id])?;
    Ok(conn.changes() > 0)
}

pub fn ensure_tag(conn: &Connection, name: &str, color: &str) -> AppResult<i64> {
    if let Some(tag) = get_tag_by_name(conn, name)? {
        return Ok(tag.id);
    }
    create_tag(conn, name, color, "auto")?;
    Ok(conn.last_insert_rowid())
}

pub fn set_content_tags(conn: &Connection, content_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    conn.execute("DELETE FROM content_tags WHERE content_id = ?", params![content_id])?;
    for &tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO content_tags (content_id, tag_id) VALUES (?, ?)",
            params![content_id, tag_id],
        )?;
    }
    Ok(())
}
