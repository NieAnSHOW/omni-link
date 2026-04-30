use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub file_name: String,
    pub source_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_size: i64,
    pub word_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteDetail {
    pub note: Note,
    pub content: String,
}

pub mod persona;
pub mod terminal_session;
