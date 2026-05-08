use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub note_id: Option<i64>,
    pub persona_skill: String,
    pub mode: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalSession {
    pub note_id: Option<i64>,
    pub persona_skill: String,
    pub mode: String,
}
