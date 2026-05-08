use std::fs;
use std::path::Path;

use tauri::State;

use crate::config::{save_config, ConfigState};
use crate::error::{AppError, AppResult};
use crate::models::FileEntry;

/// Validate that a given path is within the workspace directory.
/// Uses canonicalize to resolve symlinks and `..` components,
/// then checks that the canonical path starts with the canonical workspace path.
fn validate_path_in_workspace(path: &Path, workspace_path: &Path) -> AppResult<()> {
    let canonical_target = path
        .canonicalize()
        .map_err(|e| AppError::Config(format!("Invalid path: {} - {}", path.display(), e)))?;

    let canonical_workspace = workspace_path.canonicalize().map_err(|e| {
        AppError::Config(format!(
            "Invalid workspace path: {} - {}",
            workspace_path.display(),
            e
        ))
    })?;

    if !canonical_target.starts_with(&canonical_workspace) {
        return Err(AppError::Config(
            "Path traversal detected: path is outside workspace".into(),
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn set_workspace(path: String, config: State<'_, ConfigState>) -> AppResult<()> {
    let workspace = Path::new(&path);
    if !workspace.exists() || !workspace.is_dir() {
        return Err(AppError::Config(format!(
            "Workspace path does not exist or is not a directory: {}",
            path
        )));
    }

    let canonical = workspace
        .canonicalize()
        .map_err(|e| AppError::Config(format!("Failed to canonicalize workspace path: {}", e)))?;

    let canonical_str = canonical
        .to_str()
        .ok_or_else(|| AppError::Config("Workspace path contains invalid UTF-8".into()))?
        .to_string();

    let mut cfg = config.0.lock().unwrap();
    cfg.workspace_path = Some(canonical_str);
    save_config(&cfg)?;

    Ok(())
}

#[tauri::command]
pub fn get_workspace(config: State<'_, ConfigState>) -> AppResult<Option<String>> {
    let cfg = config.0.lock().unwrap();
    Ok(cfg.workspace_path.clone())
}

#[tauri::command]
pub fn list_files(dir: String, config: State<'_, ConfigState>) -> AppResult<Vec<FileEntry>> {
    let cfg = config.0.lock().unwrap();
    let workspace_path = cfg
        .workspace_path
        .as_ref()
        .ok_or_else(|| AppError::Config("No workspace path configured".into()))?;

    let root = Path::new(&dir);
    validate_path_in_workspace(root, Path::new(workspace_path))?;

    drop(cfg); // Release lock before IO

    if !root.exists() || !root.is_dir() {
        return Err(AppError::Config(format!(
            "Directory does not exist or is not a directory: {}",
            dir
        )));
    }

    Ok(scan_directory(root, 0))
}

/// Recursively scan a directory, returning only .md files and directories.
/// Skips hidden files/directories (starting with `.`).
/// Recursion is limited to a max depth of 10 levels.
fn scan_directory(dir: &Path, depth: usize) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    if depth >= 10 {
        return entries;
    }

    let Ok(read_dir) = fs::read_dir(dir) else {
        return entries;
    };

    let mut dir_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    dir_entries.sort_by_key(|e| e.file_name());

    for entry in dir_entries {
        let file_name = entry
            .file_name()
            .to_str()
            .unwrap_or("")
            .to_string();

        // Skip hidden files and directories
        if file_name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let is_dir = path.is_dir();

        if is_dir {
            let children = scan_directory(&path, depth + 1);
            // Include directory even if it has no children (user may want to create files there)
            entries.push(FileEntry {
                name: file_name,
                path: path.to_str().unwrap_or("").to_string(),
                is_dir: true,
                children,
            });
        } else if file_name.ends_with(".md") {
            entries.push(FileEntry {
                name: file_name,
                path: path.to_str().unwrap_or("").to_string(),
                is_dir: false,
                children: vec![],
            });
        }
    }

    entries
}

#[tauri::command]
pub fn read_file(path: String, config: State<'_, ConfigState>) -> AppResult<String> {
    let cfg = config.0.lock().unwrap();
    let workspace_path = cfg
        .workspace_path
        .as_ref()
        .ok_or_else(|| AppError::Config("No workspace path configured".into()))?;

    let file_path = Path::new(&path);
    validate_path_in_workspace(file_path, Path::new(workspace_path))?;

    drop(cfg); // Release lock before IO

    let content = fs::read_to_string(file_path)?;
    Ok(content)
}

#[tauri::command]
pub fn write_file(path: String, content: String, config: State<'_, ConfigState>) -> AppResult<()> {
    let cfg = config.0.lock().unwrap();
    let workspace_path = cfg
        .workspace_path
        .as_ref()
        .ok_or_else(|| AppError::Config("No workspace path configured".into()))?;

    let file_path = Path::new(&path);
    // For write_file, the file may not exist yet, so we validate the parent directory
    // or the file itself if it exists.
    if file_path.exists() {
        validate_path_in_workspace(file_path, Path::new(workspace_path))?;
    } else {
        // Validate parent directory exists and is within workspace
        let parent = file_path
            .parent()
            .ok_or_else(|| AppError::Config("Invalid file path: no parent directory".into()))?;

        // If parent exists, validate it. If not, we'll create it below, but first
        // validate the ultimate target path won't escape workspace by checking the
        // path components.
        if parent.exists() {
            validate_path_in_workspace(parent, Path::new(workspace_path))?;
        } else {
            // Walk up to find an existing ancestor and validate it
            let mut ancestor = parent;
            while !ancestor.exists() && ancestor.parent().is_some() {
                ancestor = ancestor.parent().unwrap();
            }
            if ancestor.exists() {
                validate_path_in_workspace(ancestor, Path::new(workspace_path))?;
            }
        }
    }

    drop(cfg); // Release lock before IO

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(file_path, content)?;
    Ok(())
}
