use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub skill_name: String,
    pub category: String,
    pub description: String,
    pub is_builtin: bool,
    pub is_installed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersona {
    pub name: String,
    pub skill_name: String,
    pub category: String,
    pub description: String,
    pub is_builtin: bool,
}
