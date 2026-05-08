// src-tauri/src/pi/skills.rs
use std::path::PathBuf;

use crate::error::AppResult;

/// Built-in skill entry: compiled into binary
struct BuiltinSkill {
    name: &'static str,
    content: &'static str,
}

/// Compiled-in built-in skills (SKILL.md files)
const BUILTIN_SKILLS: &[BuiltinSkill] = &[
    BuiltinSkill {
        name: "fangqikiki-perspective",
        content: include_str!("../../skills/fangqikiki-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "leitanzhang-perspective",
        content: include_str!("../../skills/leitanzhang-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "huashu-perspective",
        content: include_str!("../../skills/huashu-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "nuwa-skill",
        content: include_str!("../../skills/nuwa-skill/SKILL.md"),
    },
    BuiltinSkill {
        name: "luo-yonghao-perspective",
        content: include_str!("../../skills/luo-yonghao-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "zhangxiaolong-perspective",
        content: include_str!("../../skills/zhangxiaolong-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "unified-search",
        content: include_str!("../../skills/unified-search/SKILL.md"),
    },
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub builtin: bool,
}

/// Manages deployment of built-in skills to pi's auto-discovery path
pub struct PiSkillsManager {
    /// Target: ~/.pi/agent/skills/
    deploy_dir: PathBuf,
}

impl PiSkillsManager {
    pub fn new() -> Self {
        let deploy_dir = dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".pi/agent/skills");
        Self { deploy_dir }
    }

    /// Deploy built-in skills to ~/.pi/agent/skills/ (skip if already exists)
    pub fn deploy_builtin_skills(&self) -> AppResult<()> {
        std::fs::create_dir_all(&self.deploy_dir)?;

        for builtin in BUILTIN_SKILLS {
            let skill_dir = self.deploy_dir.join(builtin.name);
            std::fs::create_dir_all(&skill_dir)?;
            let skill_md = skill_dir.join("SKILL.md");
            if !skill_md.exists() {
                std::fs::write(&skill_md, builtin.content)?;
                tracing::info!("Deployed builtin skill: {}", builtin.name);
            }
        }

        tracing::info!("Skills deployed to {}", self.deploy_dir.display());
        Ok(())
    }

    /// List all deployed skills (built-in + user-added)
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        let mut skills: Vec<SkillInfo> = BUILTIN_SKILLS
            .iter()
            .map(|s| SkillInfo {
                name: s.name.to_string(),
                builtin: true,
            })
            .collect();

        if let Ok(entries) = std::fs::read_dir(&self.deploy_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !skills.iter().any(|s| s.name == name) {
                            skills.push(SkillInfo {
                                name: name.to_string(),
                                builtin: false,
                            });
                        }
                    }
                }
            }
        }

        skills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin_skills() {
        let manager = PiSkillsManager::new();
        let skills = manager.list_skills();
        let builtin: Vec<_> = skills.iter().filter(|s| s.builtin).collect();
        assert!(builtin.len() >= 4);
        assert!(builtin.iter().any(|s| s.name == "fangqikiki-perspective"));
    }
}
