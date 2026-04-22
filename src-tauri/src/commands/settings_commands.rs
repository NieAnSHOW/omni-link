use std::collections::HashMap;
use tauri::State;

use crate::config::{self, AiConfig, ConfigState};
use crate::db::DbState;
use crate::error::AppResult;
use crate::repositories::settings_repo;

#[tauri::command]
pub async fn get_settings(state: State<'_, DbState>) -> AppResult<HashMap<String, String>> {
    let conn = state.0.lock().unwrap();
    settings_repo::get_all_settings(&conn)
}

#[tauri::command]
pub async fn get_ai_config(state: State<'_, ConfigState>) -> AppResult<AiConfig> {
    let mut config = state.0.lock().unwrap();
    if let Ok(reloaded) = config::load_config() {
        *config = reloaded;
    }
    Ok(config.ai.clone())
}

#[tauri::command]
pub async fn update_ai_config(
    state: State<'_, ConfigState>,
    provider: String,
    openai_api_key: Option<String>,
    openai_base_url: Option<String>,
    openai_model: Option<String>,
    ollama_base_url: Option<String>,
    ollama_model: Option<String>,
) -> AppResult<()> {
    let mut config = state.0.lock().unwrap();
    config.ai.provider = provider;

    if let Some(key) = openai_api_key {
        config.ai.openai.api_key = key;
    }
    if let Some(url) = openai_base_url {
        config.ai.openai.base_url = url;
    }
    if let Some(model) = openai_model {
        config.ai.openai.model = model;
    }
    if let Some(url) = ollama_base_url {
        config.ai.ollama.base_url = url;
    }
    if let Some(model) = ollama_model {
        config.ai.ollama.model = model;
    }

    config::save_config(&config)?;
    Ok(())
}
