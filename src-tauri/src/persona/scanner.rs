use std::fs;
use std::path::Path;
use crate::error::{AppError, AppResult};
use crate::models::persona::Persona;
use super::builtin::get_builtin_skills_dir;

pub fn scan_local_skills() -> AppResult<Vec<Persona>> {
    let mut personas = Vec::new();

    let builtin_dir = get_builtin_skills_dir();
    if builtin_dir.exists() {
        personas.extend(scan_skills_directory(&builtin_dir, true)?);
    }

    let cache_dir = dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".claude/plugins/cache");

    if cache_dir.exists() {
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let plugin_dir = entry.path();
            if plugin_dir.is_dir() {
                let skills_path = plugin_dir.join("skills");
                if skills_path.exists() {
                    personas.extend(scan_skills_directory(&skills_path, false)?);
                }
            }
        }
    }

    Ok(personas)
}

fn scan_skills_directory(dir: &Path, is_builtin: bool) -> AppResult<Vec<Persona>> {
    let mut personas = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with("-perspective.md") {
                    if let Ok(persona) = parse_skill_file(&path, is_builtin) {
                        personas.push(persona);
                    }
                }
            }
        }
    }

    Ok(personas)
}

fn parse_skill_file(path: &Path, is_builtin: bool) -> AppResult<Persona> {
    let content = fs::read_to_string(path)?;
    let (name, description, category) = parse_frontmatter(&content)?;

    let skill_name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(Persona {
        id: 0,
        name,
        skill_name,
        category,
        description,
        is_builtin,
        is_installed: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn parse_frontmatter(content: &str) -> AppResult<(String, String, String)> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || !lines[0].starts_with("---") {
        return Err(AppError::Parse("Missing frontmatter".to_string()));
    }

    let mut name = String::new();
    let mut description = String::new();
    let mut category = String::from("其他");

    for line in lines.iter().skip(1) {
        if line.starts_with("---") {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "name" => name = value.trim().to_string(),
                "description" => description = value.trim().to_string(),
                "category" => category = value.trim().to_string(),
                _ => {}
            }
        }
    }

    if name.is_empty() {
        return Err(AppError::Parse("Frontmatter missing 'name' field".to_string()));
    }

    Ok((name, description, category))
}
