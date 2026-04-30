use rusqlite::Connection;

use crate::error::AppResult;

pub fn init_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            file_name TEXT NOT NULL UNIQUE,
            source_url TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            file_size INTEGER DEFAULT 0,
            word_count INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);

        CREATE TABLE IF NOT EXISTS personas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            skill_name TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL,
            description TEXT,
            is_builtin INTEGER DEFAULT 0,
            is_installed INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS terminal_sessions (
            id TEXT PRIMARY KEY,
            note_id INTEGER NOT NULL,
            persona_skill TEXT NOT NULL,
            mode TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'running',
            created_at TEXT NOT NULL,
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );",
    )?;

    migrate(conn)?;
    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO user_settings (key, value) VALUES ('notes_storage_path', '~/.omnilink/notes/')",
        [],
    )?;

    migrate_links_to_notes(conn)?;
    drop_old_tables(conn)?;

    Ok(())
}

fn migrate_links_to_notes(conn: &Connection) -> AppResult<()> {
    let has_links: bool = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='links'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if !has_links {
        return Ok(());
    }

    let notes_dir: String = conn
        .query_row(
            "SELECT value FROM user_settings WHERE key = 'notes_storage_path'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "~/.omnilink/notes/".to_string());

    let expanded_dir = shellexpand::tilde(&notes_dir).to_string();
    let notes_path = std::path::PathBuf::from(&expanded_dir);
    std::fs::create_dir_all(&notes_path)?;

    let mut stmt = conn.prepare(
        "SELECT l.id, l.url, l.title, c.body_text
         FROM links l
         LEFT JOIN contents c ON c.link_id = l.id
         WHERE c.body_text IS NOT NULL AND c.body_text != ''",
    )?;

    let rows: Vec<(i64, String, Option<String>, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (link_id, url, title, body_text) in rows {
        let note_title = title.unwrap_or_else(|| format!("链接笔记 {}", link_id));
        let content = body_text.unwrap_or_default();
        let timestamp = chrono::Utc::now().timestamp();
        let file_name = format!("note_from_link_{}_{}.md", timestamp, link_id);

        conn.execute(
            "INSERT INTO notes (title, file_name, source_url, created_at, updated_at, file_size, word_count)
             VALUES (?, ?, ?, datetime('now'), datetime('now'), ?, ?)",
            rusqlite::params![
                note_title, file_name, url,
                content.len() as i64,
                content.split_whitespace().count() as i64
            ],
        )?;

        let file_path = notes_path.join(&file_name);
        std::fs::write(&file_path, &content)?;
    }

    tracing::info!("Migrated links+contents to notes");
    Ok(())
}

fn drop_old_tables(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS content_tags;
         DROP TABLE IF EXISTS ai_results;
         DROP TABLE IF EXISTS contents;
         DROP TABLE IF EXISTS tags;
         DROP TABLE IF EXISTS import_history;
         DROP TABLE IF EXISTS links;
         DROP INDEX IF EXISTS idx_links_status;
         DROP INDEX IF EXISTS idx_links_platform;
         DROP INDEX IF EXISTS idx_links_created;
         DROP INDEX IF EXISTS idx_contents_link;",
    )?;
    tracing::info!("Dropped old tables");
    Ok(())
}
