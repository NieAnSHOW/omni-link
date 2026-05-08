use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};

use crate::config::{self, ClaudeConfig, ConfigState};
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::terminal_session::CreateTerminalSession;
use crate::pi::manager::PiManager;
use crate::pi::runtime::PiRuntime;
use crate::pi::skills::PiSkillsManager;
use crate::repositories::terminal_session_repo;

pub struct PiManagerState(pub Mutex<PiManager>);
pub struct PiRuntimeState(pub Arc<PiRuntime>);
pub struct PiSkillsState(pub Arc<PiSkillsManager>);

#[tauri::command]
pub async fn check_pi_installed(
    runtime: State<'_, PiRuntimeState>,
) -> AppResult<serde_json::Value> {
    let status = runtime.0.check_installed();
    Ok(serde_json::json!({
        "nodeInstalled": status.node_installed,
        "piInstalled": status.pi_installed,
        "nodeVersion": status.node_version,
        "piVersion": status.pi_version,
    }))
}

#[tauri::command]
pub async fn install_pi(
    runtime: State<'_, PiRuntimeState>,
) -> AppResult<serde_json::Value> {
    runtime.0.install_node()?;
    runtime.0.install_pi()?;
    let status = runtime.0.check_installed();
    Ok(serde_json::json!({
        "nodeInstalled": status.node_installed,
        "piInstalled": status.pi_installed,
        "piVersion": status.pi_version,
    }))
}

#[tauri::command]
pub async fn start_pi_session(
    app_handle: AppHandle,
    manager: State<'_, PiManagerState>,
) -> AppResult<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mgr = manager.0.lock().unwrap();
    mgr.create_session(app_handle, session_id.clone())?;
    Ok(session_id)
}

#[tauri::command]
pub async fn start_pi_output(
    session_id: String,
    app_handle: AppHandle,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.start_output_loop(app_handle, &session_id)
}

#[tauri::command]
pub async fn write_pi_input(
    session_id: String,
    data: String,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.write_input(&session_id, &data)
}

#[tauri::command]
pub async fn resize_pi_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.resize_session(&session_id, rows, cols)
}

// === Persona Rewrite ===

#[tauri::command]
pub async fn start_persona_rewrite(
    note_id: i64,
    note_path: String,
    persona_skill: String,
    mode: String,
    app_handle: AppHandle,
    db: State<'_, DbState>,
    manager: State<'_, PiManagerState>,
) -> AppResult<String> {
    tracing::info!("start_persona_rewrite: note_id={}, persona={}", note_id, persona_skill);

    let conn = db.0.lock().unwrap();
    let create_session = CreateTerminalSession {
        note_id,
        persona_skill: persona_skill.clone(),
        mode,
    };
    let session_id = terminal_session_repo::insert_terminal_session(&conn, &create_session)?;
    drop(conn);

    let prompt = format!("使用 {} 重构 {}，直接覆盖内容", persona_skill, note_path);
    let mgr = manager.0.lock().unwrap();
    mgr.create_persona_session(app_handle, session_id.clone(), &prompt)?;
    Ok(session_id)
}

#[tauri::command]
pub async fn update_session_status(
    session_id: String,
    status: String,
    db: State<'_, DbState>,
) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    terminal_session_repo::update_session_status(&conn, &session_id, &status)
}

#[tauri::command]
pub async fn close_pi_session(
    session_id: String,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.close_session(&session_id)
}

#[tauri::command]
pub async fn get_agent_config(config: State<'_, ConfigState>) -> AppResult<ClaudeConfig> {
    let cfg = config.0.lock().unwrap();
    Ok(cfg.claude_code.clone())
}

#[tauri::command]
pub async fn update_agent_config(
    provider: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    config: State<'_, ConfigState>,
) -> AppResult<()> {
    let mut cfg = config.0.lock().unwrap();
    if let Some(v) = provider { cfg.claude_code.provider = v; }
    if let Some(v) = api_key { cfg.claude_code.api_key = v; }
    if let Some(v) = base_url { cfg.claude_code.base_url = v; }
    if let Some(v) = model { cfg.claude_code.model = v; }
    config::save_config(&cfg)?;
    Ok(())
}

#[tauri::command]
pub async fn list_pi_skills(
    skills: State<'_, PiSkillsState>,
) -> AppResult<serde_json::Value> {
    let list = skills.0.list_skills();
    Ok(serde_json::json!(list))
}

#[tauri::command]
pub async fn deploy_pi_skills(skills: State<'_, PiSkillsState>) -> AppResult<()> {
    skills.0.deploy_builtin_skills()
}
