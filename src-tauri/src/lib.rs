mod ai;
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

            tracing::info!("OmniLink application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::link_commands::get_links,
            commands::link_commands::create_link,
            commands::link_commands::get_link,
            commands::link_commands::delete_link,
            commands::link_commands::parse_link_cmd,
            commands::link_commands::start_ai_process,
            commands::content_commands::get_link_detail,
            commands::content_commands::analyze_content_cmd,
            commands::content_commands::update_content_cmd,
            commands::content_commands::ai_process_content_cmd,
            commands::settings_commands::get_settings,
            commands::settings_commands::get_ai_config,
            commands::settings_commands::update_ai_config,
            commands::tag_commands::get_tags,
            commands::tag_commands::create_tag,
            commands::tag_commands::delete_tag,
            commands::tag_commands::update_content_tags,
            commands::note_commands::create_note,
            commands::note_commands::get_note,
            commands::note_commands::list_notes,
            commands::note_commands::update_note,
            commands::note_commands::delete_note,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
