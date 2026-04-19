use rusqlite::Connection;
use std::sync::Mutex;

use crate::config::data_dir;
use crate::error::AppResult;

pub struct DbState(pub Mutex<Connection>);

pub fn init_connection() -> AppResult<Connection> {
    std::fs::create_dir_all(data_dir())?;
    let db_path = data_dir().join("omnilink.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}
