mod ai;
mod commands;
mod config;
mod db;
mod error;
mod models;
mod parser;
mod repositories;

use config::ConfigState;
use db::DbState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = db::database::init_connection()?;
            db::schema::init_schema(&conn)?;
            config::migrate_ai_config_from_db(&conn)?;
            let app_config = config::load_config()?;

            app.manage(DbState(std::sync::Mutex::new(conn)));
            app.manage(ConfigState(std::sync::Mutex::new(app_config)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::link_commands::get_links,
            commands::link_commands::create_link,
            commands::link_commands::get_link,
            commands::link_commands::delete_link,
            commands::link_commands::parse_link_cmd,
            commands::content_commands::get_link_detail,
            commands::content_commands::analyze_content_cmd,
            commands::settings_commands::get_settings,
            commands::settings_commands::get_ai_config,
            commands::settings_commands::update_ai_config,
            commands::tag_commands::get_tags,
            commands::tag_commands::create_tag,
            commands::tag_commands::delete_tag,
            commands::tag_commands::update_content_tags,
            commands::category_commands::get_categories,
            commands::category_commands::upsert_category,
            commands::category_commands::delete_category,
            commands::category_commands::update_link_category,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
