use std::fs;
use std::path::PathBuf;
use crate::error::AppResult;

pub fn get_builtin_skills_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".claude/plugins/cache/omnilink-builtin/skills")
}

pub fn initialize_builtin_skills() -> AppResult<()> {
    let target_dir = get_builtin_skills_dir();
    fs::create_dir_all(&target_dir)?;

    let builtin_skills = [
        ("fangqikiki-perspective.md", include_str!("../../skills/fangqikiki-perspective.md")),
        ("leitanzhang-perspective.md", include_str!("../../skills/leitanzhang-perspective.md")),
        ("huashu-perspective.md", include_str!("../../skills/huashu-perspective.md")),
        ("nuwa-skill.md", include_str!("../../skills/nuwa-skill.md")),
    ];

    for (filename, content) in builtin_skills {
        let target_path = target_dir.join(filename);
        if !target_path.exists() {
            fs::write(&target_path, content)?;
        }
    }

    Ok(())
}
