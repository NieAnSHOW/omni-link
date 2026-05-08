use std::fs;
use std::path::Path;
use crate::error::{AppError, AppResult};
use crate::models::persona::Persona;
use super::builtin::get_builtin_personas;

pub fn scan_local_skills() -> AppResult<Vec<Persona>> {
    let mut personas = get_builtin_personas();

    // Scan pi skills directory
    let pi_skills_dir = dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".pi/agent/skills");

    if pi_skills_dir.exists() {
        if let Ok(extra) = scan_skills_directory(&pi_skills_dir, true) {
            for p in extra {
                if !personas.iter().any(|existing| existing.skill_name == p.skill_name) {
                    personas.push(p);
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

        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                if let Ok(persona) = parse_skill_file(&skill_md, is_builtin) {
                    personas.push(persona);
                }
            }
        }
    }

    Ok(personas)
}

fn parse_skill_file(path: &Path, is_builtin: bool) -> AppResult<Persona> {
    let content = fs::read_to_string(path)?;
    let (name, description, category) = parse_frontmatter(&content)?;

    let skill_name = path.parent()
        .and_then(|p| p.file_name())
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
