use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::{CreateTagInput, TagWithCount};
use crate::repositories::tag_repo;

const TAG_COLORS: &[&str] = &[
    "#50fa7b", "#8be9fd", "#ff79c6", "#f1fa8c",
    "#ffb86c", "#bd93f9", "#ff5555", "#6272a4",
];

fn auto_color(index: usize) -> String {
    TAG_COLORS[index % TAG_COLORS.len()].to_string()
}

#[tauri::command]
pub async fn get_tags(state: State<'_, DbState>) -> AppResult<Vec<TagWithCount>> {
    let conn = state.0.lock().unwrap();
    tag_repo::get_tags_with_count(&conn)
}

#[tauri::command]
pub async fn create_tag(state: State<'_, DbState>, input: CreateTagInput) -> AppResult<TagWithCount> {
    let conn = state.0.lock().unwrap();
    let color = input.color.unwrap_or_else(|| auto_color(
        tag_repo::get_all_tags(&conn).map(|t| t.len()).unwrap_or(0)
    ));
    let tag = tag_repo::create_tag(&conn, &input.name, &color, "manual")?;
    Ok(TagWithCount {
        id: tag.id,
        name: tag.name,
        color: tag.color,
        tag_type: tag.tag_type,
        content_count: 0,
        created_at: tag.created_at,
    })
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, DbState>, id: i64) -> AppResult<bool> {
    let conn = state.0.lock().unwrap();
    tag_repo::delete_tag(&conn, id)
}

#[tauri::command]
pub async fn update_content_tags(
    state: State<'_, DbState>,
    content_id: i64,
    tag_ids: Vec<i64>,
) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    tag_repo::set_content_tags(&conn, content_id, &tag_ids)
}
