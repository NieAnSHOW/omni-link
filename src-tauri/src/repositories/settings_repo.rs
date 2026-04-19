use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::error::AppResult;

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let result = conn.query_row(
        "SELECT value FROM user_settings WHERE key = ?",
        params![key],
        |row| row.get(0),
    );
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO user_settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?",
        params![key, value, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> AppResult<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM user_settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut result = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        result.insert(k, v);
    }
    Ok(result)
}
