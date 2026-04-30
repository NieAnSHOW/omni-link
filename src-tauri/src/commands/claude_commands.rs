use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

use crate::claude::cli_downloader::CliDownloader;
use crate::claude::manager::ClaudeManager;
use crate::claude::skills::SkillsManager;
use crate::config::{self, ClaudeConfig, ConfigState};
use crate::error::AppResult;

pub struct ClaudeManagerState(pub Mutex<ClaudeManager>);
pub struct CliDownloaderState(pub Arc<CliDownloader>);
pub struct SkillsManagerState(pub Arc<SkillsManager>);

#[tauri::command]
pub async fn check_claude_installed(
    downloader: State<'_, CliDownloaderState>,
) -> AppResult<serde_json::Value> {
    let status = downloader.0.check_installed();
    Ok(serde_json::json!({
        "installed": status.installed,
        "version": status.version,
    }))
}

#[tauri::command]
pub async fn install_claude_cli(
    app_handle: AppHandle,
    downloader: State<'_, CliDownloaderState>,
) -> AppResult<serde_json::Value> {
    let status = downloader.0.install(Some(app_handle)).await?;
    Ok(serde_json::json!({
        "installed": status.installed,
        "version": status.version,
        "path": status.path,
    }))
}

#[tauri::command]
pub async fn get_claude_cli_path(
    downloader: State<'_, CliDownloaderState>,
) -> AppResult<serde_json::Value> {
    let path = downloader.0.current_cli_path();
    Ok(serde_json::json!({
        "path": path.to_string_lossy().to_string(),
        "exists": path.exists(),
    }))
}

#[tauri::command]
pub async fn start_claude_session(
    app_handle: AppHandle,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mgr = manager.0.lock().unwrap();
    mgr.create_session(app_handle, session_id.clone())?;
    Ok(session_id)
}

/// 以 print mode 启动 Claude CLI（非交互式，执行完自动退出）
/// 自动添加 --dangerously-skip-permissions 跳过权限确认
#[tauri::command]
pub async fn start_claude_print_session(
    prompt: String,
    app_handle: AppHandle,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let extra_args = vec![
        "--dangerously-skip-permissions".to_string(),
        "-p".to_string(),
        prompt,
    ];
    let mgr = manager.0.lock().unwrap();
    mgr.create_session_with_args(app_handle, session_id.clone(), extra_args)?;
    Ok(session_id)
}

#[tauri::command]
pub async fn start_claude_output(
    session_id: String,
    app_handle: AppHandle,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.start_output_loop(app_handle, &session_id)
}

#[tauri::command]
pub async fn write_claude_input(
    session_id: String,
    data: String,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.write_input(&session_id, &data)
}

#[tauri::command]
pub async fn resize_claude_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.resize_session(&session_id, rows, cols)
}

#[tauri::command]
pub async fn close_claude_session(
    session_id: String,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.close_session(&session_id)
}

#[tauri::command]
pub async fn get_claude_config(config: State<'_, ConfigState>) -> AppResult<ClaudeConfig> {
    let cfg = config.0.lock().unwrap();
    Ok(cfg.claude_code.clone())
}

#[tauri::command]
pub async fn update_claude_config(
    provider: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    config: State<'_, ConfigState>,
) -> AppResult<()> {
    let mut cfg = config.0.lock().unwrap();
    if let Some(v) = provider {
        cfg.claude_code.provider = v;
    }
    if let Some(v) = api_key {
        cfg.claude_code.api_key = v;
    }
    if let Some(v) = base_url {
        cfg.claude_code.base_url = v;
    }
    if let Some(v) = model {
        cfg.claude_code.model = v;
    }
    config::save_config(&cfg)?;
    Ok(())
}

#[tauri::command]
pub async fn list_claude_skills(
    skills: State<'_, SkillsManagerState>,
) -> AppResult<serde_json::Value> {
    let list = skills.0.list_skills();
    Ok(serde_json::json!(list))
}

#[tauri::command]
pub async fn deploy_claude_skills(skills: State<'_, SkillsManagerState>) -> AppResult<()> {
    skills.0.deploy_builtin_skills()
}
