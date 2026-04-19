use rusqlite::{params, Connection};

use crate::error::AppResult;

pub struct AiResultRow {
    pub id: i64,
    pub content_id: i64,
    pub summary: Option<String>,
    pub tags: String,
    pub classification: Option<String>,
    pub provider: Option<String>,
    pub created_at: String,
}

pub fn create_ai_result(
    conn: &Connection,
    content_id: i64,
    summary: Option<&str>,
    tags: &[String],
    classification: Option<&str>,
    provider: Option<&str>,
) -> AppResult<()> {
    let tags_json = serde_json::to_string(tags)?;
    conn.execute(
        "INSERT INTO ai_results (content_id, summary, tags, classification, provider)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(content_id) DO UPDATE SET
           summary = excluded.summary,
           tags = excluded.tags,
           classification = excluded.classification,
           provider = excluded.provider",
        params![content_id, summary, tags_json, classification, provider],
    )?;
    Ok(())
}

pub fn get_ai_result_by_content_id(conn: &Connection, content_id: i64) -> AppResult<Option<AiResultRow>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_results WHERE content_id = ?")?;
    let result = stmt.query_row(params![content_id], |row| {
        Ok(AiResultRow {
            id: row.get("id")?,
            content_id: row.get("content_id")?,
            summary: row.get("summary")?,
            tags: row.get("tags")?,
            classification: row.get("classification")?,
            provider: row.get("provider")?,
            created_at: row.get("created_at")?,
        })
    }).ok();
    Ok(result)
}
