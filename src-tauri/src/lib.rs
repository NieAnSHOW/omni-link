mod ai;
mod claude;
mod commands;
mod config;
mod db;
mod error;
mod logger;
mod models;
mod parser;
mod persona;
mod repositories;
mod terminal;

use claude::cli_downloader::CliDownloader;
use claude::manager::ClaudeManager;
use claude::skills::SkillsManager;
use commands::claude_commands::{ClaudeManagerState, CliDownloaderState, SkillsManagerState};
use commands::terminal_commands::PtyManagerState;
use config::ConfigState;
use db::DbState;
use tauri::Manager;
use terminal::PtyManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tracing::info!("OmniLink application starting");
            tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));

            let conn = db::database::init_connection()?;
            db::schema::init_schema(&conn)?;
            tracing::info!("Database initialized successfully");

            config::migrate_ai_config_from_db(&conn)?;
            let app_config = config::load_config()?;
            tracing::info!("Configuration loaded from: ~/.omnilink/config.json");

            // 初始化日志系统
            if let Err(e) = logger::init_logger(&app_config) {
                eprintln!("Failed to initialize logger: {}", e);
            }

            app.manage(DbState(std::sync::Arc::new(std::sync::Mutex::new(conn))));
            app.manage(ConfigState(std::sync::Mutex::new(app_config)));
            app.manage(PtyManagerState(std::sync::Mutex::new(PtyManager::new())));

            persona::initialize_builtin_skills()?;
            tracing::info!("Built-in persona skills initialized");

            // 初始化 Claude Code 模块
            let skills_manager = SkillsManager::new();
            skills_manager.deploy_builtin_skills()?;
            let cli_downloader = CliDownloader::new();
            let claude_manager = ClaudeManager::new(cli_downloader, skills_manager);
            // 从 ClaudeManager 获取共享 Arc，避免重复实例化
            app.manage(CliDownloaderState(claude_manager.downloader_arc()));
            app.manage(SkillsManagerState(claude_manager.skills_arc()));
            app.manage(ClaudeManagerState(std::sync::Mutex::new(claude_manager)));
            tracing::info!("Claude Code modules initialized");

            tracing::info!("OmniLink application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::note_commands::create_note,
            commands::note_commands::get_note,
            commands::note_commands::list_notes,
            commands::note_commands::update_note,
            commands::note_commands::delete_note,
            commands::note_commands::create_note_from_link,
            commands::settings_commands::get_settings,
            commands::settings_commands::get_ai_config,
            commands::settings_commands::update_ai_config,
            commands::persona_commands::scan_local_personas,
            commands::persona_commands::get_all_personas,
            commands::persona_commands::get_persona_by_skill,
            commands::persona_commands::save_persona,
            commands::persona_commands::delete_persona,
            commands::terminal_commands::start_persona_rewrite,
            commands::terminal_commands::start_reading,
            commands::terminal_commands::update_session_status,
            commands::terminal_commands::close_terminal_session,
            commands::terminal_commands::resize_terminal,
            commands::terminal_commands::get_session_info,
            commands::claude_commands::check_claude_installed,
            commands::claude_commands::install_claude_cli,
            commands::claude_commands::get_claude_cli_path,
            commands::claude_commands::start_claude_session,
            commands::claude_commands::start_claude_print_session,
            commands::claude_commands::start_claude_output,
            commands::claude_commands::write_claude_input,
            commands::claude_commands::resize_claude_terminal,
            commands::claude_commands::close_claude_session,
            commands::claude_commands::get_claude_config,
            commands::claude_commands::update_claude_config,
            commands::claude_commands::list_claude_skills,
            commands::claude_commands::deploy_claude_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
