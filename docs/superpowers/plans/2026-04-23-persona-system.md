# 人格系统实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 OmniLink 添加人格系统功能，允许用户使用不同的人格 skills 重构笔记内容

**Architecture:** 三层架构 - Vue 3 前端（对话框 + 终端组件）+ Tauri IPC 层（命令接口）+ Rust 后端（PTY 管理 + 人格扫描 + 内置 skills）

**Tech Stack:** Tauri v2, Vue 3, TypeScript, Rust, portable-pty, xterm.js, SQLite

---

## 文件结构

本实现将创建和修改以下文件：

**前端文件（创建）：**
- `src/components/persona/PersonaDialog.vue` - 人格选择对话框
- `src/components/persona/EmbeddedTerminal.vue` - 内嵌终端组件
- `src/types/persona.ts` - 人格相关类型定义
- `src/composables/usePersona.ts` - 人格 API 封装

**前端文件（修改）：**
- `src/components/notes/NoteDetail.vue` - 启用"人格撰写"按钮
- `src/router/index.ts` - 无需修改（不添加新路由）

**后端文件（创建）：**
- `src-tauri/src/commands/persona_commands.rs` - 人格相关 Tauri 命令
- `src-tauri/src/commands/terminal_commands.rs` - 终端管理 Tauri 命令
- `src-tauri/src/repositories/persona_repo.rs` - 人格数据持久化
- `src-tauri/src/repositories/terminal_session_repo.rs` - 终端会话持久化
- `src-tauri/src/persona/scanner.rs` - 扫描本地 skills
- `src-tauri/src/persona/builtin.rs` - 内置人格管理
- `src-tauri/src/terminal/pty_manager.rs` - PTY 管理
- `src-tauri/src/terminal/session.rs` - 终端会话管理
- `src-tauri/src/models/persona.rs` - 人格数据模型
- `src-tauri/src/models/terminal_session.rs` - 终端会话数据模型

**后端文件（修改）：**
- `src-tauri/src/lib.rs` - 注册新的 Tauri 命令
- `src-tauri/src/models.rs` - 导出新的模型模块
- `src-tauri/src/db/schema.rs` - 添加数据库表
- `src-tauri/Cargo.toml` - 添加新依赖

**内置 Skills 文件（创建）：**
- `src-tauri/skills/fangqikiki-perspective.md` - 房琪 kiki 人格
- `src-tauri/skills/leitanzhang-perspective.md` - 雷探长人格
- `src-tauri/skills/huashu-perspective.md` - 花叔人格
- `src-tauri/skills/nuwa-skill.md` - 女娲人格生成器

**依赖项：**
- Rust: `portable-pty = "0.8"`, `dirs = "5.0"`
- Frontend: `xterm = "^5.3.0"`, `xterm-addon-fit = "^0.8.0"`

---

## Task 1: 添加 Rust 依赖和数据库 Schema

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: 添加 Rust 依赖到 Cargo.toml**

在 `[dependencies]` 部分添加：

```toml
portable-pty = "0.8"
dirs = "5.0"
```

- [ ] **Step 2: 编译验证依赖**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功，无错误

- [ ] **Step 3: 添加 personas 表到 schema.rs**

在 `src-tauri/src/db/schema.rs` 的 `create_tables` 函数中添加：

```rust
// 在现有的 CREATE TABLE 语句后添加
conn.execute(
    "CREATE TABLE IF NOT EXISTS personas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        skill_name TEXT NOT NULL UNIQUE,
        category TEXT NOT NULL,
        description TEXT,
        is_builtin INTEGER DEFAULT 0,
        is_installed INTEGER DEFAULT 1,
        created_at TEXT NOT NULL
    )",
    [],
)?;
```

- [ ] **Step 4: 添加 terminal_sessions 表到 schema.rs**

在 personas 表后继续添加：

```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS terminal_sessions (
        id TEXT PRIMARY KEY,
        note_id INTEGER NOT NULL,
        persona_skill TEXT NOT NULL,
        mode TEXT NOT NULL,
        status TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (note_id) REFERENCES links(id)
    )",
    [],
)?;
```

- [ ] **Step 5: 验证数据库 schema**

```bash
cargo build
```

Expected: 编译成功

- [ ] **Step 6: 提交数据库 schema 更改**

```bash
git add src-tauri/Cargo.toml src-tauri/src/db/schema.rs
git commit -m "feat(persona): 添加 personas 和 terminal_sessions 表"
```

---

## Task 2: 创建人格数据模型

**Files:**
- Create: `src-tauri/src/models/persona.rs`
- Create: `src-tauri/src/models/terminal_session.rs`
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: 创建 persona.rs 模型文件**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub skill_name: String,
    pub category: String,
    pub description: String,
    pub is_builtin: bool,
    pub is_installed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersona {
    pub name: String,
    pub skill_name: String,
    pub category: String,
    pub description: String,
    pub is_builtin: bool,
}
```

- [ ] **Step 2: 创建 terminal_session.rs 模型文件**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub note_id: i64,
    pub persona_skill: String,
    pub mode: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalSession {
    pub note_id: i64,
    pub persona_skill: String,
    pub mode: String,
}
```

- [ ] **Step 3: 在 models.rs 中导出新模块**

在 `src-tauri/src/models.rs` 末尾添加：

```rust
pub mod persona;
pub mod terminal_session;

pub use persona::{Persona, CreatePersona};
pub use terminal_session::{TerminalSession, CreateTerminalSession};
```

- [ ] **Step 4: 验证模型编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交模型文件**

```bash
git add src-tauri/src/models/persona.rs src-tauri/src/models/terminal_session.rs src-tauri/src/models.rs
git commit -m "feat(persona): 添加 Persona 和 TerminalSession 数据模型"
```

---

## Task 3: 创建内置 Skills 文件

**Files:**
- Create: `src-tauri/skills/fangqikiki-perspective.md`
- Create: `src-tauri/skills/leitanzhang-perspective.md`
- Create: `src-tauri/skills/huashu-perspective.md`
- Create: `src-tauri/skills/nuwa-skill.md`

- [ ] **Step 1: 创建 skills 目录**

```bash
mkdir -p src-tauri/skills
```

- [ ] **Step 2: 创建房琪 kiki 人格文件**

创建 `src-tauri/skills/fangqikiki-perspective.md`：

```markdown
---
name: fangqikiki-perspective
description: 房琪 kiki 风格的旅游攻略撰写人格
category: 旅游攻略
---

# 房琪 kiki 人格

你是房琪 kiki，一位知名的旅游博主和内容创作者。你的写作风格特点：

## 写作风格
- 文笔优美，善于用诗意的语言描述风景
- 注重情感表达，将个人感受融入旅行叙述
- 善于讲故事，通过细节和场景营造氛围
- 语言亲切自然，像朋友聊天一样分享旅行经历

## 内容特点
- 关注旅行中的人文故事和当地文化
- 善于发现小众景点和独特体验
- 注重视觉呈现，描述画面感强
- 提供实用的旅行建议和攻略信息

## 任务
当用户请求你重构笔记内容时，请以房琪 kiki 的风格重新撰写，保持原有信息的完整性，但用更优美、更有感染力的语言表达。
```

- [ ] **Step 3: 创建雷探长人格文件**

创建 `src-tauri/skills/leitanzhang-perspective.md`：

```markdown
---
name: leitanzhang-perspective
description: 雷探长风格的探险类旅游内容撰写人格
category: 旅游攻略
---

# 雷探长人格

你是雷探长，一位专注于探险和文化探索的旅行博主。你的写作风格特点：

## 写作风格
- 探险风格，注重未知和发现的过程
- 深入挖掘历史和文化背景
- 客观记录，同时融入个人观察和思考
- 语言朴实有力，叙事节奏紧凑

## 内容特点
- 关注历史遗迹、文化传承和社会现象
- 善于通过采访和交流了解当地人的生活
- 注重真实性和深度，避免表面化的描述
- 提供丰富的背景知识和历史脉络

## 任务
当用户请求你重构笔记内容时，请以雷探长的风格重新撰写，强调探索过程和文化深度，保持客观记录的同时融入深刻的观察。
```

- [ ] **Step 4: 创建花叔人格文件**

创建 `src-tauri/skills/huashu-perspective.md`：

```markdown
---
name: huashu-perspective
description: 花叔风格的技术内容撰写人格
category: 技术类
---

# 花叔人格

你是花叔，一位技术内容创作者，擅长将复杂的技术概念用简单易懂的方式讲解。你的写作风格特点：

## 写作风格
- 深入浅出，善于用类比和比喻解释技术概念
- 逻辑清晰，结构化呈现知识点
- 语言幽默风趣，避免枯燥的技术术语堆砌
- 注重实用性，提供可操作的建议和示例

## 内容特点
- 从原理到实践，层层递进
- 善于用图表和示例辅助说明
- 关注技术的应用场景和最佳实践
- 提供完整的代码示例和详细注释

## 任务
当用户请求你重构笔记内容时，请以花叔的风格重新撰写，确保技术内容准确的同时，用更易懂、更有趣的方式表达。
```

- [ ] **Step 5: 创建女娲人格生成器文件**

创建 `src-tauri/skills/nuwa-skill.md`：

```markdown
---
name: nuwa-skill
description: 女娲人格生成器 - 用于创建新的人格 skills
category: 工具
---

# 女娲人格生成器

你是女娲，一个专门用于生成新人格 skills 的 AI 助手。你的任务是根据用户提供的人名或主题，自动调研并生成对应的 perspective skill。

## 工作流程

1. **调研阶段**
   - 搜索该人物的公开资料、作品、访谈等
   - 分析其写作风格、表达特点、思维方式
   - 总结其内容创作的核心特征

2. **生成阶段**
   - 创建符合 Claude Code skill 格式的 markdown 文件
   - 包含 frontmatter（name, description, category）
   - 详细描述写作风格和内容特点
   - 提供清晰的任务说明

3. **验证阶段**
   - 确保生成的 skill 格式正确
   - 验证描述的准确性和完整性
   - 提供使用示例

## 输出格式

生成的 skill 文件应包含：
- frontmatter（YAML 格式）
- 人格介绍
- 写作风格描述
- 内容特点说明
- 任务说明

## 注意事项
- 女娲不会出现在人格选择列表中
- 女娲仅用于生成新人格，不直接用于重构笔记
- 生成的人格应尊重原人物的风格和特点
```

- [ ] **Step 6: 验证 skills 文件创建**

```bash
ls -la src-tauri/skills/
```

Expected: 显示 4 个 .md 文件

- [ ] **Step 7: 提交内置 skills 文件**

```bash
git add src-tauri/skills/
git commit -m "feat(persona): 添加内置人格 skills（房琪 kiki、雷探长、花叔、女娲）"
```

---

## Task 4: 实现内置 Skills 初始化模块

**Files:**
- Create: `src-tauri/src/persona/mod.rs`
- Create: `src-tauri/src/persona/builtin.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 persona 模块目录**

```bash
mkdir -p src-tauri/src/persona
```

- [ ] **Step 2: 创建 persona/mod.rs**

```rust
pub mod builtin;
pub mod scanner;

pub use builtin::initialize_builtin_skills;
pub use scanner::scan_local_skills;
```

- [ ] **Step 3: 创建 builtin.rs 实现**

```rust
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// 获取内置 skills 目标目录
pub fn get_builtin_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("无法获取 home 目录")?;
    Ok(home.join(".claude/plugins/cache/omnilink-builtin/skills"))
}

/// 初始化内置 skills（应用启动时调用）
pub fn initialize_builtin_skills() -> Result<()> {
    let target_dir = get_builtin_skills_dir()?;
    
    // 创建目标目录
    fs::create_dir_all(&target_dir)
        .context("创建内置 skills 目录失败")?;
    
    // 内置 skills 内容（编译时嵌入）
    let builtin_skills = [
        ("fangqikiki-perspective.md", include_str!("../../skills/fangqikiki-perspective.md")),
        ("leitanzhang-perspective.md", include_str!("../../skills/leitanzhang-perspective.md")),
        ("huashu-perspective.md", include_str!("../../skills/huashu-perspective.md")),
        ("nuwa-skill.md", include_str!("../../skills/nuwa-skill.md")),
    ];
    
    // 复制文件（仅在不存在时）
    for (filename, content) in builtin_skills {
        let target_path = target_dir.join(filename);
        
        if !target_path.exists() {
            fs::write(&target_path, content)
                .with_context(|| format!("写入 {} 失败", filename))?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_skills_dir() {
        let dir = get_builtin_skills_dir().unwrap();
        assert!(dir.to_string_lossy().contains(".claude/plugins/cache/omnilink-builtin/skills"));
    }
}
```

- [ ] **Step 4: 在 lib.rs 中导入 persona 模块**

在 `src-tauri/src/lib.rs` 的模块声明部分添加：

```rust
mod persona;
```

- [ ] **Step 5: 在 lib.rs 的 setup 钩子中调用初始化**

在 `tauri::Builder::default()` 的 `.setup()` 钩子中添加：

```rust
.setup(|app| {
    // 初始化内置人格 skills
    if let Err(e) = persona::initialize_builtin_skills() {
        eprintln!("初始化内置 skills 失败: {}", e);
    }
    Ok(())
})
```

- [ ] **Step 6: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 7: 提交内置 skills 初始化模块**

```bash
git add src-tauri/src/persona/
git commit -m "feat(persona): 实现内置 skills 初始化模块"
```

---

## Task 5: 实现人格扫描器

**Files:**
- Create: `src-tauri/src/persona/scanner.rs`
- Modify: `src-tauri/src/persona/mod.rs`

- [ ] **Step 1: 创建 scanner.rs 基础结构**

```rust
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use crate::models::Persona;
use crate::persona::builtin::get_builtin_skills_dir;

/// 扫描本地所有可用的人格 skills
pub fn scan_local_skills() -> Result<Vec<Persona>> {
    let mut personas = Vec::new();
    
    // 1. 扫描内置人格（优先）
    let builtin_dir = get_builtin_skills_dir()?;
    if builtin_dir.exists() {
        personas.extend(scan_skills_directory(&builtin_dir, true)?);
    }
    
    // 2. 扫描用户本地 Claude Code skills
    let home = dirs::home_dir()
        .context("无法获取 home 目录")?;
    let cache_dir = home.join(".claude/plugins/cache");
    
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

/// 扫描指定目录下的 skills
fn scan_skills_directory(dir: &Path, is_builtin: bool) -> Result<Vec<Persona>> {
    let mut personas = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // 只扫描 perspective skills，排除女娲
                if filename.ends_with("-perspective.md") && filename != "nuwa-skill.md" {
                    if let Ok(persona) = parse_skill_file(&path, is_builtin) {
                        personas.push(persona);
                    }
                }
            }
        }
    }
    
    Ok(personas)
}

/// 解析 skill 文件，提取人格信息
fn parse_skill_file(path: &Path, is_builtin: bool) -> Result<Persona> {
    let content = fs::read_to_string(path)?;
    
    // 解析 frontmatter
    let (name, description, category) = parse_frontmatter(&content)?;
    
    // 从文件名提取 skill_name
    let skill_name = path.file_stem()
        .and_then(|s| s.to_str())
        .context("无效的文件名")?
        .to_string();
    
    Ok(Persona {
        id: 0, // 临时 ID，扫描时不需要
        name,
        skill_name,
        category,
        description,
        is_builtin,
        is_installed: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// 解析 YAML frontmatter
fn parse_frontmatter(content: &str) -> Result<(String, String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() || !lines[0].starts_with("---") {
        anyhow::bail!("缺少 frontmatter");
    }
    
    let mut name = String::new();
    let mut description = String::new();
    let mut category = String::from("其他");
    
    for line in lines.iter().skip(1) {
        if line.starts_with("---") {
            break;
        }
        
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            
            match key {
                "name" => name = value.to_string(),
                "description" => description = value.to_string(),
                "category" => category = value.to_string(),
                _ => {}
            }
        }
    }
    
    if name.is_empty() {
        anyhow::bail!("frontmatter 缺少 name 字段");
    }
    
    Ok((name, description, category))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
description: Test description
category: 测试类
---

# Content
"#;
        let (name, desc, cat) = parse_frontmatter(content).unwrap();
        assert_eq!(name, "test-skill");
        assert_eq!(desc, "Test description");
        assert_eq!(cat, "测试类");
    }
}
```

- [ ] **Step 2: 在 persona/mod.rs 中导出 scanner**

确认 `src-tauri/src/persona/mod.rs` 已包含：

```rust
pub mod scanner;
pub use scanner::scan_local_skills;
```

- [ ] **Step 3: 添加 chrono 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交人格扫描器**

```bash
git add src-tauri/src/persona/scanner.rs src-tauri/Cargo.toml
git commit -m "feat(persona): 实现人格扫描器"
```

---

## Task 6: 实现人格仓库（Repository）

**Files:**
- Create: `src-tauri/src/repositories/persona_repo.rs`
- Modify: `src-tauri/src/repositories/mod.rs`

- [ ] **Step 1: 创建 persona_repo.rs**

```rust
use rusqlite::{params, Connection, Result};
use crate::models::{Persona, CreatePersona};

/// 插入新人格
pub fn insert_persona(conn: &Connection, persona: &CreatePersona) -> Result<i64> {
    conn.execute(
        "INSERT INTO personas (name, skill_name, category, description, is_builtin, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        params![
            persona.name,
            persona.skill_name,
            persona.category,
            persona.description,
            persona.is_builtin as i32,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 查询所有人格
pub fn get_all_personas(conn: &Connection) -> Result<Vec<Persona>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, skill_name, category, description, is_builtin, is_installed, created_at
         FROM personas
         ORDER BY is_builtin DESC, created_at DESC"
    )?;
    
    let personas = stmt.query_map([], |row| {
        Ok(Persona {
            id: row.get(0)?,
            name: row.get(1)?,
            skill_name: row.get(2)?,
            category: row.get(3)?,
            description: row.get(4)?,
            is_builtin: row.get::<_, i32>(5)? != 0,
            is_installed: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;
    
    Ok(personas)
}

/// 根据 skill_name 查询人格
pub fn get_persona_by_skill_name(conn: &Connection, skill_name: &str) -> Result<Option<Persona>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, skill_name, category, description, is_builtin, is_installed, created_at
         FROM personas
         WHERE skill_name = ?1"
    )?;
    
    let mut rows = stmt.query(params![skill_name])?;
    
    if let Some(row) = rows.next()? {
        Ok(Some(Persona {
            id: row.get(0)?,
            name: row.get(1)?,
            skill_name: row.get(2)?,
            category: row.get(3)?,
            description: row.get(4)?,
            is_builtin: row.get::<_, i32>(5)? != 0,
            is_installed: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}

/// 删除人格
pub fn delete_persona(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM personas WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_insert_and_get_persona() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE personas (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                skill_name TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL,
                description TEXT,
                is_builtin INTEGER DEFAULT 0,
                is_installed INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        let persona = CreatePersona {
            name: "测试人格".to_string(),
            skill_name: "test-skill".to_string(),
            category: "测试类".to_string(),
            description: "测试描述".to_string(),
            is_builtin: false,
        };
        
        let id = insert_persona(&conn, &persona).unwrap();
        assert!(id > 0);
        
        let found = get_persona_by_skill_name(&conn, "test-skill").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "测试人格");
    }
}
```

- [ ] **Step 2: 在 repositories/mod.rs 中导出**

在 `src-tauri/src/repositories/mod.rs` 末尾添加：

```rust
pub mod persona_repo;
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 4: 运行单元测试**

```bash
cargo test persona_repo
```

Expected: 测试通过

- [ ] **Step 5: 提交人格仓库**

```bash
git add src-tauri/src/repositories/persona_repo.rs src-tauri/src/repositories/mod.rs
git commit -m "feat(persona): 实现人格数据仓库"
```

---

## Task 7: 实现终端会话仓库

**Files:**
- Create: `src-tauri/src/repositories/terminal_session_repo.rs`
- Modify: `src-tauri/src/repositories/mod.rs`

- [ ] **Step 1: 创建 terminal_session_repo.rs**

```rust
use rusqlite::{params, Connection, Result};
use crate::models::{TerminalSession, CreateTerminalSession};

/// 插入新终端会话
pub fn insert_terminal_session(
    conn: &Connection,
    session: &CreateTerminalSession,
) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT INTO terminal_sessions (id, note_id, persona_skill, mode, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'running', datetime('now'))",
        params![id, session.note_id, session.persona_skill, session.mode],
    )?;
    
    Ok(id)
}

/// 更新会话状态
pub fn update_session_status(
    conn: &Connection,
    session_id: &str,
    status: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE terminal_sessions SET status = ?1 WHERE id = ?2",
        params![status, session_id],
    )?;
    Ok(())
}

/// 根据 ID 查询会话
pub fn get_session_by_id(conn: &Connection, session_id: &str) -> Result<Option<TerminalSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, note_id, persona_skill, mode, status, created_at
         FROM terminal_sessions
         WHERE id = ?1"
    )?;
    
    let mut rows = stmt.query(params![session_id])?;
    
    if let Some(row) = rows.next()? {
        Ok(Some(TerminalSession {
            id: row.get(0)?,
            note_id: row.get(1)?,
            persona_skill: row.get(2)?,
            mode: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}

/// 根据笔记 ID 查询会话列表
pub fn get_sessions_by_note_id(conn: &Connection, note_id: i64) -> Result<Vec<TerminalSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, note_id, persona_skill, mode, status, created_at
         FROM terminal_sessions
         WHERE note_id = ?1
         ORDER BY created_at DESC"
    )?;
    
    let sessions = stmt.query_map(params![note_id], |row| {
        Ok(TerminalSession {
            id: row.get(0)?,
            note_id: row.get(1)?,
            persona_skill: row.get(2)?,
            mode: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;
    
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_insert_and_get_session() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE terminal_sessions (
                id TEXT PRIMARY KEY,
                note_id INTEGER NOT NULL,
                persona_skill TEXT NOT NULL,
                mode TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        let session = CreateTerminalSession {
            note_id: 1,
            persona_skill: "test-skill".to_string(),
            mode: "manual".to_string(),
        };
        
        let id = insert_terminal_session(&conn, &session).unwrap();
        assert!(!id.is_empty());
        
        let found = get_session_by_id(&conn, &id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().status, "running");
    }
}
```

- [ ] **Step 2: 添加 uuid 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
uuid = { version = "1.0", features = ["v4"] }
```

- [ ] **Step 3: 在 repositories/mod.rs 中导出**

在 `src-tauri/src/repositories/mod.rs` 末尾添加：

```rust
pub mod terminal_session_repo;
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 5: 运行单元测试**

```bash
cargo test terminal_session_repo
```

Expected: 测试通过

- [ ] **Step 6: 提交终端会话仓库**

```bash
git add src-tauri/src/repositories/terminal_session_repo.rs src-tauri/Cargo.toml
git commit -m "feat(persona): 实现终端会话数据仓库"
```

---

## Task 8: 实现 PTY 管理器

**Files:**
- Create: `src-tauri/src/terminal/mod.rs`
- Create: `src-tauri/src/terminal/pty_manager.rs`
- Create: `src-tauri/src/terminal/session.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 terminal 模块目录**

```bash
mkdir -p src-tauri/src/terminal
```

- [ ] **Step 2: 创建 terminal/mod.rs**

```rust
pub mod pty_manager;
pub mod session;

pub use pty_manager::PtyManager;
pub use session::PtySession;
```

- [ ] **Step 3: 创建 session.rs（会话数据结构）**

```rust
use portable_pty::{Child, MasterPty, PtySize};
use std::io::Write;
use anyhow::Result;

/// PTY 会话
pub struct PtySession {
    pub id: String,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    pub note_id: i64,
    pub persona_skill: String,
}

impl PtySession {
    /// 写入命令到 PTY
    pub fn write_command(&mut self, command: &str) -> Result<()> {
        let mut writer = self.master.take_writer()?;
        write!(writer, "{}\n", command)?;
        writer.flush()?;
        Ok(())
    }
    
    /// 调整 PTY 大小
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}
```

- [ ] **Step 4: 创建 pty_manager.rs（核心逻辑）**

```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use crate::terminal::session::PtySession;

/// PTY 管理器
pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 创建新的 PTY 会话
    pub fn create_session(
        &self,
        session_id: String,
        note_id: i64,
        note_path: &str,
        persona_skill: &str,
    ) -> Result<()> {
        let pty_system = native_pty_system();
        
        // 创建 PTY
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        
        // 启动 claude 命令
        let mut cmd = CommandBuilder::new("claude");
        let child = pair.slave.spawn_command(cmd)
            .context("启动 claude 命令失败")?;
        
        // 创建会话
        let mut session = PtySession {
            id: session_id.clone(),
            master: pair.master,
            child,
            note_id,
            persona_skill: persona_skill.to_string(),
        };
        
        // 自动输入命令
        let command = format!(
            "使用 {} 重构 {}，直接覆盖内容",
            persona_skill, note_path
        );
        session.write_command(&command)?;
        
        // 保存会话
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);
        
        Ok(())
    }
    
    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.id.clone())
    }
    
    /// 关闭会话
    pub fn close_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
        }
        Ok(())
    }
    
    /// 调整会话终端大小
    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> Result<()> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            session.resize(rows, cols)?;
        }
        Ok(())
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_manager_creation() {
        let manager = PtyManager::new();
        assert!(manager.get_session("non-existent").is_none());
    }
}
```

- [ ] **Step 5: 在 lib.rs 中导入 terminal 模块**

在 `src-tauri/src/lib.rs` 的模块声明部分添加：

```rust
mod terminal;
```

- [ ] **Step 6: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 7: 提交 PTY 管理器**

```bash
git add src-tauri/src/terminal/
git commit -m "feat(persona): 实现 PTY 管理器和会话管理"
```

---

## Task 9: 实现 Tauri 命令（人格相关）

**Files:**
- Create: `src-tauri/src/commands/persona_commands.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 创建 persona_commands.rs**

```rust
use tauri::State;
use std::sync::Mutex;
use crate::db::get_connection;
use crate::models::{Persona, CreatePersona};
use crate::repositories::persona_repo;
use crate::persona::scan_local_skills;

/// 扫描本地人格 skills
#[tauri::command]
pub async fn scan_local_personas() -> Result<Vec<Persona>, String> {
    scan_local_skills()
        .map_err(|e| format!("扫描人格失败: {}", e))
}

/// 获取所有已安装的人格
#[tauri::command]
pub async fn get_all_personas() -> Result<Vec<Persona>, String> {
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    persona_repo::get_all_personas(&conn)
        .map_err(|e| format!("查询人格失败: {}", e))
}

/// 根据 skill_name 获取人格
#[tauri::command]
pub async fn get_persona_by_skill(skill_name: String) -> Result<Option<Persona>, String> {
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    persona_repo::get_persona_by_skill_name(&conn, &skill_name)
        .map_err(|e| format!("查询人格失败: {}", e))
}

/// 保存人格到数据库
#[tauri::command]
pub async fn save_persona(persona: CreatePersona) -> Result<i64, String> {
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: ", e))?;
    
    persona_repo::insert_persona(&conn, &persona)
        .map_err(|e| format!("保存人格失败: {}", e))
}

/// 删除人格
#[tauri::command]
pub async fn delete_persona(id: i64) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    persona_repo::delete_persona(&conn, id)
        .map_err(|e| format!("删除人格失败: {}", e))
}
```

- [ ] **Step 2: 在 commands/mod.rs 中导出**

在 `src-tauri/src/commands/mod.rs` 末尾添加：

```rust
pub mod persona_commands;
pub use persona_commands::*;
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 4: 提交人格命令**

```bash
git add src-tauri/src/commands/persona_commands.rs src-tauri/src/commands/mod.rs
git commit -m "feat(persona): 实现人格相关 Tauri 命令"
```

---

## Task 10: 实现 Tauri 命令（终端相关）

**Files:**
- Create: `src-tauri/src/commands/terminal_commands.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 terminal_commands.rs**

```rust
use tauri::{State, Manager};
use std::sync::Mutex;
use crate::db::get_connection;
use crate::models::{TerminalSession, CreateTerminalSession};
use crate::repositories::terminal_session_repo;
use crate::terminal::PtyManager;

/// 全局 PTY 管理器状态
pub struct PtyManagerState(pub Mutex<PtyManager>);

/// 启动人格重构会话
#[tauri::command]
pub async fn start_persona_rewrite(
    note_id: i64,
    note_path: String,
    persona_skill: String,
    mode: String,
    pty_manager: State<'_, PtyManagerState>,
) -> Result<String, String> {
    // 创建会话记录
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    let create_session = CreateTerminalSession {
        note_id,
        persona_skill: persona_skill.clone(),
        mode: mode.clone(),
    };
    
    let session_id = terminal_session_repo::insert_terminal_session(&conn, &create_session)
        .map_err(|e| format!("创建会话失败: {}", e))?;
    
    // 创建 PTY 会话
    let manager = pty_manager.0.lock().unwrap();
    manager.create_session(
        session_id.clone(),
        note_id,
        &note_path,
        &persona_skill,
    )
    .map_err(|e| format!("创建 PTY 会话失败: {}", e))?;
    
    Ok(session_id)
}

/// 更新会话状态
#[tauri::command]
pub async fn update_session_status(
    session_id: String,
    status: String,
) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    terminal_session_repo::update_session_status(&conn, &session_id, &status)
        .map_err(|e| format!("更新会话状态失败: {}", e))
}

/// 关闭终端会话
#[tauri::command]
pub async fn close_terminal_session(
    session_id: String,
    pty_manager: State<'_, PtyManagerState>,
) -> Result<(), String> {
    let manager = pty_manager.0.lock().unwrap();
    manager.close_session(&session_id)
        .map_err(|e| format!("关闭会话失败: {}", e))
}

/// 调整终端大小
#[tauri::command]
pub async fn resize_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    pty_manager: State<'_, PtyManagerState>,
) -> Result<(), String> {
    let manager = pty_manager.0.lock().unwrap();
    manager.resize_session(&session_id, rows, cols)
        .map_err(|e| format!("调整终端大小失败: {}", e))
}

/// 获取会话信息
#[tauri::command]
pub async fn get_session_info(session_id: String) -> Result<Option<TerminalSession>, String> {
    let conn = get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    terminal_session_repo::get_session_by_id(&conn, &session_id)
        .map_err(|e| format!("查询会话失败: {}", e))
}
```

- [ ] **Step 2: 在 commands/mod.rs 中导出**

在 `src-tauri/src/commands/mod.rs` 末尾添加：

```rust
pub mod terminal_commands;
pub use terminal_commands::*;
```

- [ ] **Step 3: 在 lib.rs 中初始化 PTY 管理器状态**

在 `tauri::Builder::default()` 之前添加：

```rust
use commands::terminal_commands::PtyManagerState;
use terminal::PtyManager;
```

在 `.setup()` 钩子中添加：

```rust
.manage(PtyManagerState(Mutex::new(PtyManager::new())))
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交终端命令**

```bash
git add src-tauri/src/commands/terminal_commands.rs src-tauri/src/lib.rs
git commit -m "feat(persona): 实现终端相关 Tauri 命令"
```

---

## Task 11: 注册所有 Tauri 命令

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 lib.rs 中注册人格和终端命令**

在 `.invoke_handler()` 中添加所有新命令：

```rust
.invoke_handler(tauri::generate_handler![
    // 现有命令...
    commands::get_all_links,
    commands::get_link_by_id,
    // ... 其他现有命令 ...
    
    // 人格相关命令
    commands::scan_local_personas,
    commands::get_all_personas,
    commands::get_persona_by_skill,
    commands::save_persona,
    commands::delete_persona,
    
    // 终端相关命令
    commands::start_persona_rewrite,
    commands::update_session_status,
    commands::close_terminal_session,
    commands::resize_terminal,
    commands::get_session_info,
])
```

- [ ] **Step 2: 验证所有命令注册成功**

```bash
cd src-tauri
cargo build
```

Expected: 编译成功，无警告

- [ ] **Step 3: 提交命令注册**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(persona): 注册所有人格和终端 Tauri 命令"
```

---

## Task 12: 添加前端依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 xterm.js 相关依赖**

```bash
npm install xterm@^5.3.0 xterm-addon-fit@^0.8.0
```

- [ ] **Step 2: 验证依赖安装**

```bash
npm list xterm xterm-addon-fit
```

Expected: 显示已安装的版本

- [ ] **Step 3: 提交 package.json 和 lock 文件**

```bash
git add package.json package-lock.json
git commit -m "feat(persona): 添加 xterm.js 终端依赖"
```

---

## Task 13: 创建前端类型定义

**Files:**
- Create: `src/types/persona.ts`
- Modify: `src/types/index.ts`

- [ ] **Step 1: 创建 persona.ts 类型文件**

```typescript
export interface Persona {
  id: number;
  name: string;
  skillName: string;
  category: string;
  description: string;
  isBuiltin: boolean;
  isInstalled: boolean;
  createdAt: string;
}

export interface CreatePersona {
  name: string;
  skillName: string;
  category: string;
  description: string;
  isBuiltin: boolean;
}

export interface TerminalSession {
  id: string;
  noteId: number;
  personaSkill: string;
  mode: 'smart' | 'manual';
  status: 'running' | 'completed' | 'failed';
  createdAt: string;
}

export interface CreateTerminalSession {
  noteId: number;
  personaSkill: string;
  mode: 'smart' | 'manual';
}

export interface StartRewriteParams {
  noteId: number;
  notePath: string;
  personaSkill: string;
  mode: 'smart' | 'manual';
}
```

- [ ] **Step 2: 在 types/index.ts 中导出**

在 `src/types/index.ts` 末尾添加：

```typescript
export * from './persona'
```

- [ ] **Step 3: 验证类型定义**

```bash
npm run type-check
```

Expected: 无类型错误

- [ ] **Step 4: 提交类型定义**

```bash
git add src/types/persona.ts src/types/index.ts
git commit -m "feat(persona): 添加人格和终端会话类型定义"
```

---

## Task 14: 创建前端 API 封装

**Files:**
- Create: `src/composables/usePersona.ts`

- [ ] **Step 1: 创建 usePersona.ts**

```typescript
import { invoke } from '@tauri-apps/api/tauri'
import type {
  Persona,
  CreatePersona,
  TerminalSession,
  StartRewriteParams
} from '@/types/persona'

export function usePersona() {
  /**
   * 扫描本地人格 skills
   */
  const scanLocalPersonas = async (): Promise<Persona[]> => {
    return await invoke<Persona[]>('scan_local_personas')
  }

  /**
   * 获取所有已安装的人格
   */
  const getAllPersonas = async (): Promise<Persona[]> => {
    return await invoke<Persona[]>('get_all_personas')
  }

  /**
   * 根据 skill_name 获取人格
   */
  const getPersonaBySkill = async (skillName: string): Promise<Persona | null> => {
    return await invoke<Persona | null>('get_persona_by_skill', { skillName })
  }

  /**
   * 保存人格到数据库
   */
  const savePersona = async (persona: CreatePersona): Promise<number> => {
    return await invoke<number>('save_persona', { persona })
  }

  /**
   * 删除人格
   */
  const deletePersona = async (id: number): Promise<void> => {
    await invoke('delete_persona', { id })
  }

  /**
   * 启动人格重构会话
   */
  const startPersonaRewrite = async (params: StartRewriteParams): Promise<string> => {
    return await invoke<string>('start_persona_rewrite', params)
  }

  /**
   * 更新会话状态
   */
  const updateSessionStatus = async (sessionId: string, status: string): Promise<void> => {
    await invoke('update_session_status', { sessionId, status })
  }

  /**
   * 关闭终端会话
   */
  const closeTerminalSession = async (sessionId: string): Promise<void> => {
    await invoke('close_terminal_session', { sessionId })
  }

  /**
   * 调整终端大小
   */
  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    await invoke('resize_terminal', { sessionId, rows, cols })
  }

  /**
   * 获取会话信息
   */
  const getSessionInfo = async (sessionId: string): Promise<TerminalSession | null> => {
    return await invoke<TerminalSession | null>('get_session_info', { sessionId })
  }

  return {
    scanLocalPersonas,
    getAllPersonas,
    getPersonaBySkill,
    savePersona,
    deletePersona,
    startPersonaRewrite,
    updateSessionStatus,
    closeTerminalSession,
    resizeTerminal,
    getSessionInfo
  }
}
```

- [ ] **Step 2: 验证类型检查**

```bash
npm run type-check
```

Expected: 无类型错误

- [ ] **Step 3: 提交 API 封装**

```bash
git add src/composables/usePersona.ts
git commit -m "feat(persona): 添加人格 API 封装"
```

---

## Task 15: 创建人格选择对话框组件

**Files:**
- Create: `src/components/persona/PersonaDialog.vue`

- [ ] **Step 1: 创建 persona 组件目录**

```bash
mkdir -p src/components/persona
```

- [ ] **Step 2: 创建 PersonaDialog.vue（第一部分：模板）**

```vue
<template>
  <div v-if="visible" class="dialog-overlay" @click.self="handleClose">
    <div class="dialog-container">
      <div class="dialog-header">
        <h2>人格撰写</h2>
        <button class="close-btn" @click="handleClose">✕</button>
      </div>

      <div class="dialog-tabs">
        <button
          :class="['tab', { active: activeTab === 'smart' }]"
          @click="activeTab = 'smart'"
        >
          智能选择
        </button>
        <button
          :class="['tab', { active: activeTab === 'manual' }]"
          @click="activeTab = 'manual'"
        >
          手动选择
        </button>
      </div>

      <div class="dialog-body">
        <!-- 智能选择 Tab -->
        <div v-if="activeTab === 'smart'" class="tab-content">
          <div class="coming-soon">
            <div class="icon">🚧</div>
            <h3>智能选择功能即将开放</h3>
            <p>Phase 3 将支持 AI 自动分析笔记内容并匹配合适的人格</p>
          </div>
        </div>

        <!-- 手动选择 Tab -->
        <div v-else class="tab-content">
          <div v-if="loading" class="loading">
            <div class="spinner"></div>
            <p>正在扫描本地人格...</p>
          </div>

          <div v-else-if="error" class="error">
            <p>{{ error }}</p>
            <button @click="loadPersonas">重试</button>
          </div>

          <div v-else-if="personas.length === 0" class="empty">
            <div class="icon">📦</div>
            <p>未找到可用的人格</p>
            <p class="hint">请确保已安装 Claude Code 并配置了 perspective skills</p>
          </div>

          <div v-else class="personas-list">
            <div
              v-for="persona in personas"
              :key="persona.skillName"
              :class="['persona-card', { selected: selectedPersona?.skillName === persona.skillName }]"
              @click="selectedPersona = persona"
            >
              <div class="persona-header">
                <h3>{{ persona.name }}</h3>
                <span v-if="persona.isBuiltin" class="builtin-badge">内置</span>
              </div>
              <p class="persona-category">{{ persona.category }}</p>
              <p class="persona-description">{{ persona.description }}</p>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-cancel" @click="handleClose">取消</button>
        <button
          class="btn-confirm"
          :disabled="!selectedPersona || activeTab === 'smart'"
          @click="handleConfirm"
        >
          开始重构
        </button>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 3: 创建 PersonaDialog.vue（第二部分：脚本）**

```vue
<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { usePersona } from '@/composables/usePersona'
import type { Persona } from '@/types/persona'

const props = defineProps<{
  visible: boolean
  noteId: number
}>()

const emit = defineEmits<{
  close: []
  confirm: [persona: Persona, mode: 'smart' | 'manual']
}>()

const { scanLocalPersonas } = usePersona()

const activeTab = ref<'smart' | 'manual'>('manual')
const personas = ref<Persona[]>([])
const selectedPersona = ref<Persona | null>(null)
const loading = ref(false)
const error = ref('')

const loadPersonas = async () => {
  loading.value = true
  error.value = ''
  
  try {
    personas.value = await scanLocalPersonas()
  } catch (e) {
    error.value = e instanceof Error ? e.message : '扫描人格失败'
  } finally {
    loading.value = false
  }
}

const handleClose = () => {
  emit('close')
}

const handleConfirm = () => {
  if (selectedPersona.value) {
    emit('confirm', selectedPersona.value, activeTab.value)
  }
}

watch(() => props.visible, (visible) => {
  if (visible) {
    loadPersonas()
    selectedPersona.value = null
  }
})

onMounted(() => {
  if (props.visible) {
    loadPersonas()
  }
})
</script>
```

- [ ] **Step 4: 创建 PersonaDialog.vue（第三部分：样式）**

```vue
<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-container {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 700px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
}

.dialog-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: #6b7280;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
}

.close-btn:hover {
  background: #f3f4f6;
}

.dialog-tabs {
  display: flex;
  gap: 8px;
  padding: 16px 24px 0;
  border-bottom: 1px solid #e5e7eb;
}

.tab {
  padding: 8px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: #6b7280;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.tab.active {
  color: #6366f1;
  border-bottom-color: #6366f1;
}

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.tab-content {
  min-height: 300px;
}

.coming-soon {
  text-align: center;
  padding: 60px 20px;
}

.coming-soon .icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.coming-soon h3 {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}

.coming-soon p {
  margin: 0;
  color: #6b7280;
  font-size: 14px;
}

.loading,
.error,
.empty {
  text-align: center;
  padding: 60px 20px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f4f6;
  border-top-color: #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error p {
  color: #dc2626;
  margin-bottom: 16px;
}

.error button {
  padding: 8px 16px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.empty .icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty p {
  margin: 8px 0;
  color: #6b7280;
}

.empty .hint {
  font-size: 13px;
}

.personas-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.persona-card {
  padding: 16px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.persona-card:hover {
  border-color: #6366f1;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.1);
}

.persona-card.selected {
  border-color: #6366f1;
  background: #f0f9ff;
}

.persona-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 8px;
}

.persona-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.builtin-badge {
  padding: 2px 8px;
  background: #d1fae5;
  color: #065f46;
  font-size: 12px;
  border-radius: 4px;
}

.persona-category {
  margin: 0 0 8px;
  font-size: 13px;
  color: #6b7280;
}

.persona-description {
  margin: 0;
  font-size: 14px;
  color: #374151;
  line-height: 1.5;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid #e5e7eb;
}

.btn-cancel,
.btn-confirm {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel {
  background: #f3f4f6;
  color: #374151;
}

.btn-cancel:hover {
  background: #e5e7eb;
}

.btn-confirm {
  background: #6366f1;
  color: white;
}

.btn-confirm:hover:not(:disabled) {
  background: #4f46e5;
}

.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
```

- [ ] **Step 5: 验证组件编译**

```bash
npm run type-check
```

Expected: 无类型错误

- [ ] **Step 6: 提交人格选择对话框**

```bash
git add src/components/persona/PersonaDialog.vue
git commit -m "feat(persona): 创建人格选择对话框组件"
```

---

## Task 16: 创建内嵌终端组件

**Files:**
- Create: `src/components/persona/EmbeddedTerminal.vue`

- [ ] **Step 1: 创建 EmbeddedTerminal.vue（第一部分：模板）**

```vue
<template>
  <div v-if="visible" class="terminal-overlay">
    <div class="terminal-container">
      <div class="terminal-header">
        <div class="terminal-title">
          <span class="icon">⚡</span>
          <span>人格重构进行中...</span>
        </div>
        <div class="terminal-status">
          <span :class="['status-dot', statusClass]"></span>
          <span>{{ statusText }}</span>
        </div>
      </div>
      
      <div ref="terminalRef" class="terminal-body"></div>
      
      <div class="terminal-footer">
        <button
          v-if="status === 'completed' || status === 'failed'"
          class="btn-close"
          @click="handleClose"
        >
          关闭
        </button>
        <div v-else class="progress-hint">
          <span class="spinner-small"></span>
          <span>正在执行，请稍候...</span>
        </div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 创建 EmbeddedTerminal.vue（第二部分：脚本）**

```vue
<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePersona } from '@/composables/usePersona'
import 'xterm/css/xterm.css'

const props = defineProps<{
  visible: boolean
  sessionId: string
  noteId: number
}>()

const emit = defineEmits<{
  close: []
  completed: []
  failed: [error: string]
}>()

const { updateSessionStatus, closeTerminalSession, resizeTerminal } = usePersona()

const terminalRef = ref<HTMLElement | null>(null)
const status = ref<'running' | 'completed' | 'failed'>('running')
const errorMessage = ref('')

let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let unlistenOutput: UnlistenFn | null = null
let unlistenStatus: UnlistenFn | null = null

const statusClass = computed(() => {
  switch (status.value) {
    case 'running': return 'running'
    case 'completed': return 'completed'
    case 'failed': return 'failed'
    default: return ''
  }
})

const statusText = computed(() => {
  switch (status.value) {
    case 'running': return '执行中'
    case 'completed': return '已完成'
    case 'failed': return '执行失败'
    default: return ''
  }
})

const initTerminal = () => {
  if (!terminalRef.value) return

  terminal = new Terminal({
    cursorBlink: false,
    disableStdin: true,
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
      black: '#000000',
      red: '#cd3131',
      green: '#0dbc79',
      yellow: '#e5e510',
      blue: '#2472c8',
      magenta: '#bc3fbc',
      cyan: '#11a8cd',
      white: '#e5e5e5',
      brightBlack: '#666666',
      brightRed: '#f14c4c',
      brightGreen: '#23d18b',
      brightYellow: '#f5f543',
      brightBlue: '#3b8eea',
      brightMagenta: '#d670d6',
      brightCyan: '#29b8db',
      brightWhite: '#e5e5e5'
    }
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.open(terminalRef.value)
  fitAddon.fit()

  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
}

const handleResize = () => {
  if (fitAddon && terminal) {
    fitAddon.fit()
    const { rows, cols } = terminal
    resizeTerminal(props.sessionId, rows, cols).catch(console.error)
  }
}

const setupEventListeners = async () => {
  // 监听 PTY 输出
  unlistenOutput = await listen<string>('pty-output', (event) => {
    if (terminal && event.payload) {
      terminal.write(event.payload)
    }
  })

  // 监听会话状态
  unlistenStatus = await listen<{ sessionId: string; status: string; error?: string }>(
    'session-status',
    async (event) => {
      if (event.payload.sessionId === props.sessionId) {
        const newStatus = event.payload.status as 'running' | 'completed' | 'failed'
        status.value = newStatus

        if (newStatus === 'completed') {
          await updateSessionStatus(props.sessionId, 'completed')
          emit('completed')
        } else if (newStatus === 'failed') {
          errorMessage.value = event.payload.error || '执行失败'
          await updateSessionStatus(props.sessionId, 'failed')
          emit('failed', errorMessage.value)
        }
      }
    }
  )
}

const handleClose = async () => {
  await closeTerminalSession(props.sessionId)
  emit('close')
}

const cleanup = () => {
  if (unlistenOutput) {
    unlistenOutput()
    unlistenOutput = null
  }
  if (unlistenStatus) {
    unlistenStatus()
    unlistenStatus = null
  }
  if (terminal) {
    terminal.dispose()
    terminal = null
  }
  window.removeEventListener('resize', handleResize)
}

watch(() => props.visible, (visible) => {
  if (visible) {
    setTimeout(() => {
      initTerminal()
      setupEventListeners()
    }, 100)
  } else {
    cleanup()
  }
})

onMounted(() => {
  if (props.visible) {
    initTerminal()
    setupEventListeners()
  }
})

onUnmounted(() => {
  cleanup()
})
</script>
```

- [ ] **Step 3: 创建 EmbeddedTerminal.vue（第三部分：样式）**

```vue
<style scoped>
.terminal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
}

.terminal-container {
  background: #1e1e1e;
  border-radius: 12px;
  width: 90%;
  max-width: 1000px;
  height: 70vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: #2d2d2d;
  border-bottom: 1px solid #3e3e3e;
}

.terminal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #d4d4d4;
  font-size: 14px;
  font-weight: 500;
}

.terminal-title .icon {
  font-size: 18px;
}

.terminal-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #a0a0a0;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-dot.running {
  background: #e5e510;
}

.status-dot.completed {
  background: #0dbc79;
  animation: none;
}

.status-dot.failed {
  background: #cd3131;
  animation: none;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.terminal-body {
  flex: 1;
  overflow: hidden;
  padding: 8px;
}

.terminal-footer {
  padding: 12px 20px;
  background: #2d2d2d;
  border-top: 1px solid #3e3e3e;
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.btn-close {
  padding: 8px 20px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-close:hover {
  background: #4f46e5;
}

.progress-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #a0a0a0;
  font-size: 13px;
}

.spinner-small {
  width: 16px;
  height: 16px;
  border: 2px solid #3e3e3e;
  border-top-color: #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
```

- [ ] **Step 4: 验证组件编译**

```bash
npm run type-check
```

Expected: 无类型错误

- [ ] **Step 5: 提交内嵌终端组件**

```bash
git add src/components/persona/EmbeddedTerminal.vue
git commit -m "feat(persona): 创建内嵌终端组件"
```

---

## Task 17: 修改笔记详情页面集成人格功能

**Files:**
- Modify: `src/components/notes/NoteDetail.vue`

- [ ] **Step 1: 在 NoteDetail.vue 中导入人格组件**

在 `<script setup>` 部分添加导入：

```typescript
import PersonaDialog from '@/components/persona/PersonaDialog.vue'
import EmbeddedTerminal from '@/components/persona/EmbeddedTerminal.vue'
import { usePersona } from '@/composables/usePersona'
import type { Persona } from '@/types/persona'
```

- [ ] **Step 2: 添加人格相关状态**

在 `<script setup>` 中添加：

```typescript
const { startPersonaRewrite } = usePersona()

const showPersonaDialog = ref(false)
const showTerminal = ref(false)
const currentSessionId = ref('')
```

- [ ] **Step 3: 实现人格撰写处理函数**

```typescript
const handlePersonaRewrite = () => {
  showPersonaDialog.value = true
}

const handlePersonaConfirm = async (persona: Persona, mode: 'smart' | 'manual') => {
  showPersonaDialog.value = false
  
  try {
    // 获取笔记文件路径（假设笔记保存在 ~/.omnilink/notes/{id}.md）
    const notePath = `~/.omnilink/notes/${props.id}.md`
    
    // 启动重构会话
    const sessionId = await startPersonaRewrite({
      noteId: props.id,
      notePath,
      personaSkill: persona.skillName,
      mode
    })
    
    currentSessionId.value = sessionId
    showTerminal.value = true
  } catch (error) {
    console.error('启动人格重构失败:', error)
    alert('启动人格重构失败: ' + (error instanceof Error ? error.message : '未知错误'))
  }
}

const handleTerminalClose = () => {
  showTerminal.value = false
  currentSessionId.value = ''
}

const handleRewriteCompleted = async () => {
  // 重新加载笔记内容
  if (notesStore.fetchNoteDetail) {
    await notesStore.fetchNoteDetail(props.id)
  }
}

const handleRewriteFailed = (error: string) => {
  console.error('人格重构失败:', error)
  alert('人格重构失败: ' + error)
}
```

- [ ] **Step 4: 启用人格撰写按钮**

找到"人格撰写"按钮，移除 `disabled` 属性并添加点击事件：

```vue
<button
  class="action-btn"
  @click="handlePersonaRewrite"
  title="使用人格重构笔记内容"
>
  <span class="icon">✨</span>
  人格撰写
</button>
```

- [ ] **Step 5: 在模板中添加对话框和终端组件**

在 `<template>` 末尾、关闭标签之前添加：

```vue
<!-- 人格选择对话框 -->
<PersonaDialog
  :visible="showPersonaDialog"
  :note-id="props.id"
  @close="showPersonaDialog = false"
  @confirm="handlePersonaConfirm"
/>

<!-- 内嵌终端 -->
<EmbeddedTerminal
  v-if="showTerminal"
  :visible="showTerminal"
  :session-id="currentSessionId"
  :note-id="props.id"
  @close="handleTerminalClose"
  @completed="handleRewriteCompleted"
  @failed="handleRewriteFailed"
/>
```

- [ ] **Step 6: 验证类型检查**

```bash
npm run type-check
```

Expected: 无类型错误

- [ ] **Step 7: 提交笔记详情页面修改**

```bash
git add src/components/notes/NoteDetail.vue
git commit -m "feat(persona): 集成人格功能到笔记详情页面"
```

---

## Task 18: 端到端测试和验证

**Files:**
- None (testing only)

- [ ] **Step 1: 启动开发服务器**

```bash
npm run dev:all
```

Expected: 前端和后端都成功启动

- [ ] **Step 2: 验证内置 skills 初始化**

检查目录是否创建：

```bash
ls -la ~/.claude/plugins/cache/omnilink-builtin/skills/
```

Expected: 显示 4 个 .md 文件（房琪 kiki、雷探长、花叔、女娲）

- [ ] **Step 3: 测试人格扫描功能**

1. 打开应用
2. 进入任意笔记详情页
3. 点击"人格撰写"按钮
4. 切换到"手动选择" Tab

Expected: 显示至少 3 个内置人格（房琪 kiki、雷探长、花叔）

- [ ] **Step 4: 测试人格选择**

1. 点击任意人格卡片
2. 卡片应高亮显示
3. "开始重构"按钮应变为可用状态

Expected: UI 响应正常

- [ ] **Step 5: 测试终端启动（模拟）**

注意：由于需要真实的 Claude Code CLI，此步骤可能失败。验证：

1. 选择一个人格
2. 点击"开始重构"
3. 观察是否弹出终端窗口

Expected: 
- 如果本地有 Claude Code：终端窗口打开，显示执行过程
- 如果没有：显示错误提示"启动 claude 命令失败"

- [ ] **Step 6: 检查数据库表**

```bash
sqlite3 ~/.omnilink/omnilink.db "SELECT * FROM personas;"
sqlite3 ~/.omnilink/omnilink.db "SELECT * FROM terminal_sessions;"
```

Expected: 表存在且结构正确

- [ ] **Step 7: 验证编译无警告**

```bash
cd src-tauri
cargo build --release
```

Expected: 编译成功，无警告

- [ ] **Step 8: 提交测试验证记录**

```bash
git add -A
git commit -m "test(persona): Phase 1 MVP 功能验证通过"
```

---

## Task 19: 文档和清理

**Files:**
- Create: `docs/persona-system-usage.md`

- [ ] **Step 1: 创建使用文档**

```markdown
# 人格系统使用指南

## 概述

人格系统允许你使用不同的"人格 skills"重构笔记内容，让 AI 以特定风格重写你的笔记。

## 前置条件

1. 安装 Claude Code CLI（https://claude.ai/code）
2. 配置 Claude Code API key
3. 确保 `claude` 命令在终端中可用

## 使用步骤

### 1. 打开笔记详情

在笔记列表中点击任意笔记，进入详情页面。

### 2. 点击"人格撰写"按钮

在笔记详情页面顶部，点击带有 ✨ 图标的"人格撰写"按钮。

### 3. 选择人格

在弹出的对话框中：

- **手动选择 Tab**：从列表中选择一个人格
  - 房琪 kiki：旅游攻略风格，文笔优美
  - 雷探长：探险风格，注重文化和历史
  - 花叔：技术类风格，深入浅出

- **智能选择 Tab**：（Phase 3 开放）AI 自动分析笔记内容并匹配合适的人格

### 4. 开始重构

点击"开始重构"按钮，系统会：

1. 启动内嵌终端
2. 调用本地 Claude Code CLI
3. 使用选定的人格重构笔记内容
4. 自动保存并刷新笔记

### 5. 查看结果

重构完成后，笔记内容会自动更新为新的风格。

## 内置人格

### 房琪 kiki（旅游攻略类）

- **适用场景**：旅游笔记、游记、景点介绍
- **风格特点**：文笔优美、善于讲故事、注重情感表达
- **示例**：将简单的景点记录转化为富有感染力的游记

### 雷探长（旅游攻略类）

- **适用场景**：探险类旅游、文化探索、历史遗迹
- **风格特点**：探险风格、深入挖掘历史文化、客观记录
- **示例**：将旅游笔记转化为深度文化探索内容

### 花叔（技术类）

- **适用场景**：技术笔记、学习笔记、代码文档
- **风格特点**：深入浅出、逻辑清晰、注重实用性
- **示例**：将技术要点转化为易懂的教程

## 常见问题

### Q: 提示"启动 claude 命令失败"

**A:** 请确保：
1. 已安装 Claude Code CLI
2. `claude` 命令在 PATH 中
3. 已配置 API key（运行 `claude --version` 验证）

### Q: 人格列表为空

**A:** 请确保：
1. 应用已成功启动（内置人格会自动初始化）
2. 检查 `~/.claude/plugins/cache/omnilink-builtin/skills/` 目录是否存在

### Q: 重构失败

**A:** 可能原因：
1. Claude Code API 配额不足
2. 网络连接问题
3. 笔记文件路径错误

查看终端输出获取详细错误信息。

## 未来功能（Phase 2 & 3）

- **人格商店**：浏览、搜索、管理更多人格
- **女娲造人**：自定义创建任意人格
- **智能选择**：AI 自动匹配最合适的人格

## 技术细节

- 人格 skills 存储在 `~/.claude/plugins/cache/omnilink-builtin/skills/`
- 会话记录保存在 SQLite 数据库的 `terminal_sessions` 表
- 使用 PTY（伪终端）与 Claude Code CLI 交互
```

- [ ] **Step 2: 提交使用文档**

```bash
git add docs/persona-system-usage.md
git commit -m "docs(persona): 添加人格系统使用指南"
```

- [ ] **Step 3: 更新主 README（如果需要）**

在项目根目录的 `README.md` 中添加人格系统功能说明（如果有的话）。

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat(persona): Phase 1 MVP 完成 - 人格系统基础功能"
```

---

## 实现完成总结

Phase 1 MVP 已完成以下功能：

**后端（Rust）：**
- ✅ 数据库 schema（personas, terminal_sessions 表）
- ✅ 数据模型（Persona, TerminalSession）
- ✅ 内置 skills 管理（4 个人格文件 + 初始化逻辑）
- ✅ 人格扫描器（扫描内置 + 本地 Claude Code skills）
- ✅ 数据仓库（persona_repo, terminal_session_repo）
- ✅ PTY 管理器（创建、管理、关闭终端会话）
- ✅ Tauri 命令（10 个命令接口）

**前端（Vue 3 + TypeScript）：**
- ✅ 类型定义（Persona, TerminalSession 等）
- ✅ API 封装（usePersona composable）
- ✅ 人格选择对话框（智能/手动 Tab 切换）
- ✅ 内嵌终端组件（xterm.js + 实时输出）
- ✅ 笔记详情页面集成

**测试和文档：**
- ✅ 端到端功能验证
- ✅ 使用文档

**下一步（Phase 2 - 人格商店）：**
- 独立的人格管理页面
- 人格分类、搜索、详情查看
- 从本地 skills 目录导入人格

**下一步（Phase 3 - 女娲造人）：**
- 创建新人格功能
- 调用女娲 skill 生成人格
- 启用智能选择模式

