use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::{Category, CategoryNode};

pub fn create_category(conn: &Connection, name: &str, parent_id: Option<i64>) -> AppResult<Category> {
    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?, ?)",
        params![name, parent_id],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Category {
        id,
        name: name.to_string(),
        parent_id,
        created_at: String::new(),
    })
}

pub fn update_category(conn: &Connection, id: i64, name: &str, parent_id: Option<i64>) -> AppResult<()> {
    conn.execute(
        "UPDATE categories SET name = ?, parent_id = ? WHERE id = ?",
        params![name, parent_id, id],
    )?;
    Ok(())
}

pub fn get_all_categories(conn: &Connection) -> AppResult<Vec<Category>> {
    let mut categories = Vec::new();
    let mut stmt = conn.prepare("SELECT * FROM categories ORDER BY id")?;
    let rows = stmt.query_map([], |row| Ok(Category {
        id: row.get("id")?,
        name: row.get("name")?,
        parent_id: row.get("parent_id")?,
        created_at: row.get("created_at")?,
    }))?;
    for row in rows {
        categories.push(row?);
    }
    Ok(categories)
}

pub fn get_category_tree(conn: &Connection) -> AppResult<Vec<CategoryNode>> {
    let all = get_all_categories(conn)?;
    Ok(build_tree(&all, None))
}

pub fn delete_category(conn: &Connection, id: i64) -> AppResult<bool> {
    conn.execute("DELETE FROM categories WHERE id = ?", params![id])?;
    Ok(conn.changes() > 0)
}

pub fn find_or_create_by_path(conn: &Connection, path: &str) -> AppResult<Option<i64>> {
    let segments: Vec<&str> = path.split('/').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Ok(None);
    }

    let mut parent_id: Option<i64> = None;
    for segment in &segments {
        let existing: Option<Category> = if let Some(pid) = parent_id {
            let mut stmt = conn.prepare("SELECT * FROM categories WHERE name = ? AND parent_id = ?")?;
            stmt.query_row(params![segment, pid], |row| Ok(Category {
                id: row.get("id")?,
                name: row.get("name")?,
                parent_id: row.get("parent_id")?,
                created_at: row.get("created_at")?,
            })).ok()
        } else {
            let mut stmt = conn.prepare("SELECT * FROM categories WHERE name = ? AND parent_id IS NULL")?;
            stmt.query_row(params![segment], |row| Ok(Category {
                id: row.get("id")?,
                name: row.get("name")?,
                parent_id: row.get("parent_id")?,
                created_at: row.get("created_at")?,
            })).ok()
        };

        match existing {
            Some(cat) => parent_id = Some(cat.id),
            None => {
                let cat = create_category(conn, segment, parent_id)?;
                parent_id = Some(cat.id);
            }
        }
    }

    Ok(parent_id)
}

fn build_tree(categories: &[Category], parent_id: Option<i64>) -> Vec<CategoryNode> {
    categories
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .map(|c| CategoryNode {
            id: c.id,
            name: c.name.clone(),
            parent_id: c.parent_id,
            children: build_tree(categories, Some(c.id)),
        })
        .collect()
}
