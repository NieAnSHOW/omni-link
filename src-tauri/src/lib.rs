mod ai;
mod commands;
mod config;
mod db;
mod error;
mod logger;
mod models;
mod parser;
mod pi;
mod persona;
mod repositories;

use commands::pi_commands::{
    PiManagerState, PiRuntimeState, PiSkillsState,
};
use config::ConfigState;
use db::DbState;
use pi::PiManager;
use pi::PiRuntime;
use pi::PiSkillsManager;
use tauri::Manager;

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

            if let Err(e) = logger::init_logger(&app_config) {
                eprintln!("Failed to initialize logger: {}", e);
            }

            app.manage(DbState(std::sync::Arc::new(std::sync::Mutex::new(conn))));
            app.manage(ConfigState(std::sync::Mutex::new(app_config)));

            // Initialize pi module
            let skills_manager = PiSkillsManager::new();
            skills_manager.deploy_builtin_skills()?;

            let runtime = PiRuntime::new();
            let manager = PiManager::new(runtime);

            app.manage(PiRuntimeState(std::sync::Arc::new(manager.runtime().clone())));
            app.manage(PiSkillsState(std::sync::Arc::new(skills_manager)));
            app.manage(PiManagerState(std::sync::Mutex::new(manager)));
            tracing::info!("Pi agent modules initialized");

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
            commands::pi_commands::check_pi_installed,
            commands::pi_commands::install_pi,
            commands::pi_commands::start_pi_session,
            commands::pi_commands::start_pi_output,
            commands::pi_commands::write_pi_input,
            commands::pi_commands::resize_pi_terminal,
            commands::pi_commands::close_pi_session,
            commands::pi_commands::get_agent_config,
            commands::pi_commands::update_agent_config,
            commands::pi_commands::start_persona_rewrite,
            commands::pi_commands::update_session_status,
            commands::pi_commands::list_pi_skills,
            commands::pi_commands::deploy_pi_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
