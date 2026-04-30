use std::fs;
use std::path::PathBuf;
use crate::error::AppResult;
use crate::models::persona::Persona;

pub fn get_builtin_skills_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".claude/plugins/cache/omnilink-builtin/skills")
}

pub fn initialize_builtin_skills() -> AppResult<()> {
    let target_dir = get_builtin_skills_dir();
    fs::create_dir_all(&target_dir)?;

    let builtin_skills = [
        ("fangqikiki-perspective.md", include_str!("../../skills/fangqikiki-perspective/SKILL.md")),
        ("leitanzhang-perspective.md", include_str!("../../skills/leitanzhang-perspective/SKILL.md")),
        ("huashu-perspective.md", include_str!("../../skills/huashu-perspective/SKILL.md")),
        ("nuwa-skill.md", include_str!("../../skills/nuwa-skill/SKILL.md")),
    ];

    for (filename, content) in builtin_skills {
        let target_path = target_dir.join(filename);
        if !target_path.exists() {
            fs::write(&target_path, content)?;
        }
    }

    Ok(())
}

pub fn get_builtin_personas() -> Vec<Persona> {
    vec![
        Persona {
            id: 0,
            name: "fangqikiki-perspective".into(),
            skill_name: "fangqikiki-perspective".into(),
            category: "旅游攻略".into(),
            description: "房琪 kiki 风格的旅游攻略撰写人格".into(),
            is_builtin: true,
            is_installed: true,
            created_at: String::new(),
        },
        Persona {
            id: 0,
            name: "leitanzhang-perspective".into(),
            skill_name: "leitanzhang-perspective".into(),
            category: "旅游攻略".into(),
            description: "雷探长风格的探险类旅游内容撰写人格".into(),
            is_builtin: true,
            is_installed: true,
            created_at: String::new(),
        },
        Persona {
            id: 0,
            name: "huashu-perspective".into(),
            skill_name: "huashu-perspective".into(),
            category: "技术类".into(),
            description: "花叔风格的技术内容撰写人格".into(),
            is_builtin: true,
            is_installed: true,
            created_at: String::new(),
        },
    ]
}
