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
            ai_processing_status TEXT DEFAULT 'idle' CHECK(ai_processing_status IN ('idle','organizing','expanding','both')),
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
            color TEXT DEFAULT '#8b9dc3',
            type TEXT DEFAULT 'manual' CHECK(type IN ('auto','manual')),
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS content_tags (
            content_id INTEGER NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (content_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS ai_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_id INTEGER NOT NULL UNIQUE REFERENCES contents(id) ON DELETE CASCADE,
            summary TEXT,
            tags TEXT DEFAULT '[]',
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

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            file_name TEXT NOT NULL UNIQUE,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            file_size INTEGER DEFAULT 0,
            word_count INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_links_status ON links(status);
        CREATE INDEX IF NOT EXISTS idx_links_platform ON links(platform);
        CREATE INDEX IF NOT EXISTS idx_links_created ON links(created_at);
        CREATE INDEX IF NOT EXISTS idx_contents_link ON contents(link_id);
        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);",
    )?;

    // Migrate existing DB
    migrate(conn)?;

    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    // content_status migration
    let has_content_status: bool = conn
        .prepare("SELECT content_status FROM contents LIMIT 0")
        .is_ok();
    if !has_content_status {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN content_status TEXT DEFAULT 'success';")?;
    }

    // updated_at for contents
    let has_updated_at: bool = conn
        .prepare("SELECT updated_at FROM contents LIMIT 0")
        .is_ok();
    if !has_updated_at {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN updated_at TEXT;")?;
        conn.execute_batch("UPDATE contents SET updated_at = datetime('now') WHERE updated_at IS NULL;")?;
    }

    // Drop categories table and category_id column from links
    drop_categories(conn)?;

    // Drop classification column from ai_results
    drop_classification(conn)?;

    // ai_processing_status migration
    migrate_ai_processing_status(conn)?;

    // Fix ai_results foreign key constraint
    fix_ai_results_foreign_key(conn)?;

    // notes_storage_path migration
    conn.execute(
        "INSERT OR IGNORE INTO user_settings (key, value) VALUES ('notes_storage_path', '~/.omnilink/notes/')",
        [],
    )?;

    Ok(())
}

fn drop_categories(conn: &Connection) -> AppResult<()> {
    // Drop categories table if exists
    conn.execute_batch("DROP TABLE IF EXISTS categories;")?;

    // Rebuild links table without category_id (SQLite < 3.35.0 safe)
    let has_category_id: bool = conn
        .prepare("SELECT category_id FROM links LIMIT 0")
        .is_ok();
    if has_category_id {
        conn.execute_batch(
            "CREATE TABLE links_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                platform TEXT,
                source TEXT DEFAULT 'manual',
                status TEXT DEFAULT 'pending' CHECK(status IN ('pending','parsing','parsed','failed')),
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );
            INSERT INTO links_new (id, url, title, platform, source, status, created_at, updated_at)
                SELECT id, url, title, platform, source, status, created_at, updated_at FROM links;
            DROP TABLE links;
            ALTER TABLE links_new RENAME TO links;
            CREATE INDEX IF NOT EXISTS idx_links_status ON links(status);
            CREATE INDEX IF NOT EXISTS idx_links_platform ON links(platform);
            CREATE INDEX IF NOT EXISTS idx_links_created ON links(created_at);",
        )?;
    }

    Ok(())
}

fn drop_classification(conn: &Connection) -> AppResult<()> {
    let has_classification: bool = conn
        .prepare("SELECT classification FROM ai_results LIMIT 0")
        .is_ok();
    if has_classification {
        conn.execute_batch(
            "CREATE TABLE ai_results_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_id INTEGER NOT NULL UNIQUE REFERENCES contents(id) ON DELETE CASCADE,
                summary TEXT,
                tags TEXT DEFAULT '[]',
                provider TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );
            INSERT INTO ai_results_new (id, content_id, summary, tags, provider, created_at)
                SELECT id, content_id, summary, tags, provider, created_at FROM ai_results;
            DROP TABLE ai_results;
            ALTER TABLE ai_results_new RENAME TO ai_results;",
        )?;
    }
    Ok(())
}

fn migrate_ai_processing_status(conn: &Connection) -> AppResult<()> {
    let has_ai_processing: bool = conn
        .prepare("SELECT ai_processing_status FROM links LIMIT 0")
        .is_ok();
    if !has_ai_processing {
        conn.execute_batch(
            "ALTER TABLE links ADD COLUMN ai_processing_status TEXT NOT NULL DEFAULT 'idle'
             CHECK(ai_processing_status IN ('idle','organizing','expanding','both'));"
        )?;
    }
    Ok(())
}

fn fix_ai_results_foreign_key(conn: &Connection) -> AppResult<()> {
    // Check if ai_results foreign key has CASCADE on delete
    let mut stmt = conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='ai_results'")?;
    let sql: String = stmt.query_row([], |row| row.get(0))?;

    // If the foreign key doesn't have ON DELETE CASCADE, rebuild the table
    if !sql.contains("ON DELETE CASCADE") {
        tracing::info!("Fixing ai_results foreign key constraint to add ON DELETE CASCADE");
        conn.execute_batch(
            "CREATE TABLE ai_results_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_id INTEGER NOT NULL UNIQUE REFERENCES contents(id) ON DELETE CASCADE,
                summary TEXT,
                tags TEXT DEFAULT '[]',
                provider TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );
            INSERT INTO ai_results_new SELECT * FROM ai_results;
            DROP TABLE ai_results;
            ALTER TABLE ai_results_new RENAME TO ai_results;",
        )?;
        tracing::info!("Successfully fixed ai_results foreign key constraint");
    }
    Ok(())
}
