use rusqlite::Connection;

use crate::error::AppResult;

pub fn init_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            title TEXT,
            platform TEXT,
            source TEXT DEFAULT 'manual',
            status TEXT DEFAULT 'pending' CHECK(status IN ('pending','parsing','parsed','failed')),
            category_id INTEGER,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS contents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            link_id INTEGER NOT NULL UNIQUE REFERENCES links(id) ON DELETE CASCADE,
            title TEXT,
            body_html TEXT,
            body_text TEXT,
            images TEXT DEFAULT '[]',
            metadata TEXT DEFAULT '{}',
            content_status TEXT DEFAULT 'success',
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT DEFAULT '#6366f1',
            type TEXT DEFAULT 'manual' CHECK(type IN ('auto','manual')),
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS content_tags (
            content_id INTEGER NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (content_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS ai_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_id INTEGER NOT NULL UNIQUE REFERENCES contents(id) ON DELETE CASCADE,
            summary TEXT,
            tags TEXT DEFAULT '[]',
            classification TEXT,
            provider TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS import_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            imported_at TEXT DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_links_status ON links(status);
        CREATE INDEX IF NOT EXISTS idx_links_platform ON links(platform);
        CREATE INDEX IF NOT EXISTS idx_links_created ON links(created_at);
        CREATE INDEX IF NOT EXISTS idx_contents_link ON contents(link_id);",
    )?;

    // Migrate existing DB: add category_id column if missing, then create index
    migrate(conn)?;

    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    let has_category_id: bool = conn
        .prepare("SELECT category_id FROM links LIMIT 0")
        .is_ok();
    if !has_category_id {
        conn.execute_batch(
            "ALTER TABLE links ADD COLUMN category_id INTEGER;
             CREATE INDEX IF NOT EXISTS idx_links_category ON links(category_id);",
        )?;
    } else {
        conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_links_category ON links(category_id);")?;
    }

    // new: content_status migration
    let has_content_status: bool = conn
        .prepare("SELECT content_status FROM contents LIMIT 0")
        .is_ok();
    if !has_content_status {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN content_status TEXT DEFAULT 'success';")?;
    }

    Ok(())
}
