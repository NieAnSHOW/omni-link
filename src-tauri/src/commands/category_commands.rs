use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::{CategoryNode, UpsertCategoryInput};
use crate::repositories::{category_repo, link_repo};

#[tauri::command]
pub async fn get_categories(state: State<'_, DbState>) -> AppResult<Vec<CategoryNode>> {
    let conn = state.0.lock().unwrap();
    category_repo::get_category_tree(&conn)
}

#[tauri::command]
pub async fn upsert_category(
    state: State<'_, DbState>,
    input: UpsertCategoryInput,
) -> AppResult<CategoryNode> {
    let conn = state.0.lock().unwrap();
    if let Some(id) = input.id {
        category_repo::update_category(&conn, id, &input.name, input.parent_id)?;
        let tree = category_repo::get_category_tree(&conn)?;
        find_node(&tree, id).cloned()
            .ok_or_else(|| crate::error::AppError::NotFound("Category not found".into()))
    } else {
        let cat = category_repo::create_category(&conn, &input.name, input.parent_id)?;
        Ok(CategoryNode {
            id: cat.id,
            name: cat.name,
            parent_id: cat.parent_id,
            children: vec![],
        })
    }
}

#[tauri::command]
pub async fn delete_category(state: State<'_, DbState>, id: i64) -> AppResult<bool> {
    let conn = state.0.lock().unwrap();
    category_repo::delete_category(&conn, id)
}

#[tauri::command]
pub async fn update_link_category(
    state: State<'_, DbState>,
    link_id: i64,
    category_id: Option<i64>,
) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    link_repo::update_link_category(&conn, link_id, category_id)
}

fn find_node<'a>(nodes: &'a [CategoryNode], id: i64) -> Option<&'a CategoryNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let found @ Some(_) = find_node(&node.children, id) {
            return found;
        }
    }
    None
}
