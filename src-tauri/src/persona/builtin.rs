use crate::error::AppResult;
use crate::models::persona::Persona;

/// No-op: skills are now managed by PiSkillsManager
pub fn initialize_builtin_skills() -> AppResult<()> {
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
