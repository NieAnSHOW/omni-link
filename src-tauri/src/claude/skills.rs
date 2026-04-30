use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::AppResult;

/// 内置 Skill 条目：编译时嵌入
struct BuiltinSkill {
    name: &'static str,
    content: &'static str,
}

/// 编译时嵌入的内置 Skills（目录中的 SKILL.md）
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
];

/// Skills Manager — 管理内置和用户自定义 Skills
pub struct SkillsManager {
    /// 用户自定义 Skills 目录（~/.omnilink/skills/）
    user_skills_dir: PathBuf,
}

impl SkillsManager {
    pub fn new() -> Self {
        let user_skills_dir = dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".omnilink/skills");
        SkillsManager { user_skills_dir }
    }

    /// 获取所有可用 Skills 的 (name, description) 列表
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        let mut skills: Vec<SkillInfo> = BUILTIN_SKILLS
            .iter()
            .map(|s| SkillInfo {
                name: s.name.to_string(),
                builtin: true,
            })
            .collect();

        // 扫描用户自定义 Skills
        if let Ok(entries) = std::fs::read_dir(&self.user_skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
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

    /// 获取指定 Skill 的完整内容（SKILL.md 的原始 Markdown）
    pub fn get_skill_content(&self, name: &str) -> AppResult<Option<String>> {
        // 优先查内置
        for builtin in BUILTIN_SKILLS {
            if builtin.name == name {
                return Ok(Some(builtin.content.to_string()));
            }
        }

        // 再查用户目录
        let skill_md = self.user_skills_dir.join(name).join("SKILL.md");
        if skill_md.exists() {
            let content = std::fs::read_to_string(&skill_md)?;
            return Ok(Some(content));
        }

        Ok(None)
    }

    /// 获取多个 Skills 的合并内容，用于注入 Claude Code 的系统提示
    pub fn get_skills_content(&self, names: &[String]) -> AppResult<String> {
        let mut parts = Vec::new();
        for name in names {
            if let Some(content) = self.get_skill_content(name)? {
                parts.push(format!("## Skill: {}\n\n{}", name, content));
            }
        }
        Ok(parts.join("\n\n---\n\n"))
    }
}

/// Skill 元信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub builtin: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin_skills() {
        let manager = SkillsManager::new();
        let skills = manager.list_skills();
        let builtin: Vec<_> = skills.iter().filter(|s| s.builtin).collect();
        assert!(builtin.len() >= 4, "Should have at least 4 builtin skills");
        assert!(builtin.iter().any(|s| s.name == "fangqikiki-perspective"));
    }

    #[test]
    fn test_get_builtin_skill_content() {
        let manager = SkillsManager::new();
        let content = manager.get_skill_content("fangqikiki-perspective").unwrap();
        assert!(content.is_some());
        assert!(content.unwrap().contains("fangqikiki-perspective"));
    }

    #[test]
    fn test_get_nonexistent_skill() {
        let manager = SkillsManager::new();
        let content = manager.get_skill_content("nonexistent-skill").unwrap();
        assert!(content.is_none());
    }
}
