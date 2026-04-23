use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::persona::{CreatePersona, Persona};

pub fn insert_persona(conn: &Connection, persona: &CreatePersona) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO personas (name, skill_name, category, description, is_builtin, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        params![
            persona.name,
            persona.skill_name,
            persona.category,
            persona.description,
            persona.is_builtin as i32,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_all_personas(conn: &Connection) -> AppResult<Vec<Persona>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, skill_name, category, description, is_builtin, is_installed, created_at
         FROM personas
         ORDER BY is_builtin DESC, created_at DESC",
    )?;

    let personas = stmt
        .query_map([], |row| {
            Ok(Persona {
                id: row.get(0)?,
                name: row.get(1)?,
                skill_name: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                is_builtin: row.get::<_, i32>(5)? != 0,
                is_installed: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(personas)
}

pub fn get_persona_by_skill_name(
    conn: &Connection,
    skill_name: &str,
) -> AppResult<Option<Persona>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, skill_name, category, description, is_builtin, is_installed, created_at
         FROM personas
         WHERE skill_name = ?1",
    )?;

    let mut rows = stmt.query(params![skill_name])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Persona {
            id: row.get(0)?,
            name: row.get(1)?,
            skill_name: row.get(2)?,
            category: row.get(3)?,
            description: row.get(4)?,
            is_builtin: row.get::<_, i32>(5)? != 0,
            is_installed: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_persona(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM personas WHERE id = ?1", params![id])?;
    Ok(())
}
