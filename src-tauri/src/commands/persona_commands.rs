use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::persona::{CreatePersona, Persona};
use crate::persona::scan_local_skills;
use crate::repositories::persona_repo;

#[tauri::command]
pub async fn scan_local_personas() -> AppResult<Vec<Persona>> {
    scan_local_skills()
}

#[tauri::command]
pub async fn get_all_personas(state: State<'_, DbState>) -> AppResult<Vec<Persona>> {
    let conn = state.0.lock().unwrap();
    persona_repo::get_all_personas(&conn)
}

#[tauri::command]
pub async fn get_persona_by_skill(
    skill_name: String,
    state: State<'_, DbState>,
) -> AppResult<Option<Persona>> {
    let conn = state.0.lock().unwrap();
    persona_repo::get_persona_by_skill_name(&conn, &skill_name)
}

#[tauri::command]
pub async fn save_persona(
    persona: CreatePersona,
    state: State<'_, DbState>,
) -> AppResult<i64> {
    let conn = state.0.lock().unwrap();
    persona_repo::insert_persona(&conn, &persona)
}

#[tauri::command]
pub async fn delete_persona(id: i64, state: State<'_, DbState>) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    persona_repo::delete_persona(&conn, id)
}
