use tauri::ipc::InvokeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("AI error: {0}")]
    Ai(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),
}

impl From<AppError> for InvokeError {
    fn from(err: AppError) -> Self {
        InvokeError::from(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
