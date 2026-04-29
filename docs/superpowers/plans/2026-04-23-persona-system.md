# 人格系统实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 OmniLink 添加人格系统功能，允许用户使用不同的人格 skills 重构笔记内容

**Architecture:** 三层架构 - Vue 3 前端（shadcn Dialog + 可折叠终端面板）+ Tauri IPC 层（命令接口）+ Rust 后端（PTY 管理 + 人格扫描 + 内置 skills）。前端已完成 Tailwind v4 + shadcn-vue 迁移，所有 UI 组件使用 shadcn 组件库和 Tailwind 类。

**Tech Stack:** Tauri v2, Vue 3, TypeScript, Rust, shadcn-vue, Tailwind CSS v4, portable-pty, xterm.js, SQLite

---

## 文件结构

**前端文件（创建）：**
- `src/types/persona.ts` - 人格相关类型定义
- `src/composables/usePersona.ts` - 人格 API 封装
- `src/components/persona/PersonaDialog.vue` - 人格选择对话框（shadcn Dialog + Tabs）
- `src/components/persona/TerminalPanel.vue` - 可折叠内嵌终端面板

**前端文件（修改）：**
- `src/components/notes/NoteDetail.vue` - 启用"人格撰写"按钮，集成 Dialog 和 TerminalPanel
- `src/types/index.ts` - 导出 persona 类型

**后端文件（创建）：**
- `src-tauri/src/commands/persona_commands.rs` - 人格相关 Tauri 命令
- `src-tauri/src/commands/terminal_commands.rs` - 终端管理 Tauri 命令
- `src-tauri/src/repositories/persona_repo.rs` - 人格数据持久化
- `src-tauri/src/repositories/terminal_session_repo.rs` - 终端会话持久化
- `src-tauri/src/persona/scanner.rs` - 扫描本地 skills
- `src-tauri/src/persona/builtin.rs` - 内置人格管理
- `src-tauri/src/persona/mod.rs` - persona 模块入口
- `src-tauri/src/terminal/pty_manager.rs` - PTY 管理
- `src-tauri/src/terminal/session.rs` - 终端会话管理
- `src-tauri/src/terminal/mod.rs` - terminal 模块入口
- `src-tauri/src/models/persona.rs` - 人格数据模型
- `src-tauri/src/models/terminal_session.rs` - 终端会话数据模型

**后端文件（修改）：**
- `src-tauri/src/lib.rs` - 注册新的 Tauri 命令和模块
- `src-tauri/src/models.rs` - 导出新的模型模块
- `src-tauri/src/db/schema.rs` - 添加数据库表
- `src-tauri/src/commands/mod.rs` - 导出命令模块
- `src-tauri/src/repositories/mod.rs` - 导出仓库模块
- `src-tauri/Cargo.toml` - 添加新依赖

**内置 Skills 文件（创建）：**
- `src-tauri/skills/fangqikiki-perspective.md` - 房琪 kiki 人格
- `src-tauri/skills/leitanzhang-perspective.md` - 雷探长人格
- `src-tauri/skills/huashu-perspective.md` - 花叔人格
- `src-tauri/skills/nuwa-skill.md` - 女娲人格生成器

**依赖项：**
- Rust: `portable-pty = "0.8"`, `uuid = { version = "1.0", features = ["v4"] }`
- Frontend: `xterm = "^5.3.0"`, `xterm-addon-fit = "^0.8.0"`
- shadcn: `tabs` 组件

---

## Task 1: 添加 Rust 依赖和数据库 Schema

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: 添加 Rust 依赖到 Cargo.toml**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 末尾添加：

```toml
portable-pty = "0.8"
uuid = { version = "1.0", features = ["v4"] }
```

- [ ] **Step 2: 编译验证依赖**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功，无错误

- [ ] **Step 3: 添加 personas 表到 schema.rs**

在 `src-tauri/src/db/schema.rs` 的 `init_schema` 函数中，`CREATE INDEX IF NOT EXISTS idx_notes_created_at` 行之后、`");` 之前添加：

```sql
        CREATE TABLE IF NOT EXISTS personas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            skill_name TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL,
            description TEXT,
            is_builtin INTEGER DEFAULT 0,
            is_installed INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS terminal_sessions (
            id TEXT PRIMARY KEY,
            note_id INTEGER NOT NULL,
            persona_skill TEXT NOT NULL,
            mode TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'running',
            created_at TEXT NOT NULL,
            FOREIGN KEY (note_id) REFERENCES notes(id)
        );
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交**

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

创建 `src-tauri/src/models/persona.rs`：

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

创建 `src-tauri/src/models/terminal_session.rs`：

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

- [ ] **Step 3: 在 models.rs 末尾添加导出**

在 `src-tauri/src/models.rs` 末尾添加：

```rust
pub mod persona;
pub mod terminal_session;
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交**

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

1. **调研阶段** — 搜索该人物的公开资料、作品、访谈等，分析其写作风格、表达特点、思维方式
2. **生成阶段** — 创建符合 Claude Code skill 格式的 markdown 文件，包含 frontmatter 和详细描述
3. **验证阶段** — 确保格式正确，描述准确完整

## 输出格式

生成的 skill 文件应包含：frontmatter（YAML 格式）、人格介绍、写作风格描述、内容特点说明、任务说明。

## 注意事项
- 女娲不会出现在人格选择列表中
- 女娲仅用于生成新人格，不直接用于重构笔记
- 生成的人格应尊重原人物的风格和特点
```

- [ ] **Step 6: 提交**

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

创建 `src-tauri/src/persona/mod.rs`：

```rust
pub mod builtin;
pub mod scanner;

pub use builtin::initialize_builtin_skills;
pub use scanner::scan_local_skills;
```

- [ ] **Step 3: 创建 persona/builtin.rs**

创建 `src-tauri/src/persona/builtin.rs`：

```rust
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
```

- [ ] **Step 4: 在 lib.rs 中导入 persona 模块并调用初始化**

在 `src-tauri/src/lib.rs` 的模块声明部分（`mod ai;` 之后）添加：

```rust
mod persona;
mod terminal;
```

在 `lib.rs` 的 `.setup()` 钩子中，`tracing::info!("OmniLink application setup completed");` 之前添加：

```rust
            persona::initialize_builtin_skills()?;
            tracing::info!("Built-in persona skills initialized");
```

- [ ] **Step 5: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功（terminal 模块暂时为空会报错，先创建空的 mod 文件）

创建 `src-tauri/src/terminal/mod.rs`：

```rust
pub mod pty_manager;
pub mod session;

pub use pty_manager::PtyManager;
pub use session::PtySession;
```

创建 `src-tauri/src/terminal/session.rs`：

```rust
// Task 7 will implement PtySession
```

创建 `src-tauri/src/terminal/pty_manager.rs`：

```rust
// Task 7 will implement PtyManager
```

- [ ] **Step 6: 再次验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 7: 提交**

```bash
git add src-tauri/src/persona/ src-tauri/src/terminal/ src-tauri/src/lib.rs
git commit -m "feat(persona): 实现内置 skills 初始化模块和 terminal 模块骨架"
```

---

## Task 5: 实现人格扫描器

**Files:**
- Create: `src-tauri/src/persona/scanner.rs`
- Modify: `src-tauri/src/persona/mod.rs`（已在 Task 4 导出）

- [ ] **Step 1: 创建 scanner.rs**

创建 `src-tauri/src/persona/scanner.rs`：

```rust
use std::fs;
use std::path::Path;
use crate::error::AppResult;
use crate::models::Persona;
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
        anyhow::bail!("Missing frontmatter");
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
        anyhow::bail!("Frontmatter missing 'name' field");
    }

    Ok((name, description, category))
}
```

- [ ] **Step 2: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/persona/scanner.rs
git commit -m "feat(persona): 实现人格扫描器"
```

---

## Task 6: 实现人格仓库

**Files:**
- Create: `src-tauri/src/repositories/persona_repo.rs`
- Modify: `src-tauri/src/repositories/mod.rs`

- [ ] **Step 1: 创建 persona_repo.rs**

创建 `src-tauri/src/repositories/persona_repo.rs`：

```rust
use rusqlite::{params, Connection};
use crate::error::AppResult;
use crate::models::{Persona, CreatePersona};

pub fn insert_persona(conn: &Connection, persona: &CreatePersona) -> AppResult<i64> {
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

pub fn get_all_personas(conn: &Connection) -> AppResult<Vec<Persona>> {
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
    .collect::<Result<Vec<_>, _>>()?;

    Ok(personas)
}

pub fn get_persona_by_skill_name(conn: &Connection, skill_name: &str) -> AppResult<Option<Persona>> {
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

pub fn delete_persona(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM personas WHERE id = ?1", params![id])?;
    Ok(())
}
```

- [ ] **Step 2: 在 repositories/mod.rs 末尾添加导出**

在 `src-tauri/src/repositories/mod.rs` 末尾添加：

```rust
pub mod persona_repo;
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 4: 提交**

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

创建 `src-tauri/src/repositories/terminal_session_repo.rs`：

```rust
use rusqlite::{params, Connection};
use crate::error::AppResult;
use crate::models::{TerminalSession, CreateTerminalSession};

pub fn insert_terminal_session(
    conn: &Connection,
    session: &CreateTerminalSession,
) -> AppResult<String> {
    let id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO terminal_sessions (id, note_id, persona_skill, mode, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'running', datetime('now'))",
        params![id, session.note_id, session.persona_skill, session.mode],
    )?;

    Ok(id)
}

pub fn update_session_status(
    conn: &Connection,
    session_id: &str,
    status: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE terminal_sessions SET status = ?1 WHERE id = ?2",
        params![status, session_id],
    )?;
    Ok(())
}

pub fn get_session_by_id(conn: &Connection, session_id: &str) -> AppResult<Option<TerminalSession>> {
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
```

- [ ] **Step 2: 在 repositories/mod.rs 末尾添加导出**

在 `src-tauri/src/repositories/mod.rs` 末尾添加：

```rust
pub mod terminal_session_repo;
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/repositories/terminal_session_repo.rs src-tauri/src/repositories/mod.rs
git commit -m "feat(persona): 实现终端会话数据仓库"
```

---

## Task 8: 实现 PTY 管理器

**Files:**
- Modify: `src-tauri/src/terminal/session.rs`（替换占位）
- Modify: `src-tauri/src/terminal/pty_manager.rs`（替换占位）

- [ ] **Step 1: 实现 terminal/session.rs**

替换 `src-tauri/src/terminal/session.rs` 内容为：

```rust
use portable_pty::{Child, MasterPty, PtySize};
use std::io::Write;
use crate::error::AppResult;

pub struct PtySession {
    pub id: String,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    pub note_id: i64,
    pub persona_skill: String,
}

impl PtySession {
    pub fn write_command(&mut self, command: &str) -> AppResult<()> {
        let mut writer = self.master.take_writer()?;
        write!(writer, "{}\n", command)?;
        writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> AppResult<()> {
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

- [ ] **Step 2: 实现 terminal/pty_manager.rs**

替换 `src-tauri/src/terminal/pty_manager.rs` 内容为：

```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error::AppResult;
use super::session::PtySession;

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(
        &self,
        session_id: String,
        note_id: i64,
        note_path: &str,
        persona_skill: &str,
    ) -> AppResult<()> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new("claude");
        let child = pair.slave.spawn_command(cmd)?;

        let mut session = PtySession {
            id: session_id.clone(),
            master: pair.master,
            child,
            note_id,
            persona_skill: persona_skill.to_string(),
        };

        let command = format!(
            "使用 {} 重构 {}，直接覆盖内容",
            persona_skill, note_path
        );
        session.write_command(&command)?;

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);

        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.id.clone())
    }

    pub fn close_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
        }
        Ok(())
    }

    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> AppResult<()> {
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
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/terminal/
git commit -m "feat(persona): 实现 PTY 管理器和会话管理"
```

---

## Task 9: 实现 Tauri 命令（人格 + 终端）

**Files:**
- Create: `src-tauri/src/commands/persona_commands.rs`
- Create: `src-tauri/src/commands/terminal_commands.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 创建 persona_commands.rs**

创建 `src-tauri/src/commands/persona_commands.rs`：

```rust
use tauri::State;
use crate::db::DbState;
use crate::models::{Persona, CreatePersona};
use crate::repositories::persona_repo;
use crate::persona::scan_local_skills;

#[tauri::command]
pub async fn scan_local_personas() -> Result<Vec<Persona>, String> {
    scan_local_skills().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_personas(db: State<'_, DbState>) -> Result<Vec<Persona>, String> {
    let conn = db.0.lock().unwrap();
    persona_repo::get_all_personas(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_persona_by_skill(
    skill_name: String,
    db: State<'_, DbState>,
) -> Result<Option<Persona>, String> {
    let conn = db.0.lock().unwrap();
    persona_repo::get_persona_by_skill_name(&conn, &skill_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_persona(
    persona: CreatePersona,
    db: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    persona_repo::insert_persona(&conn, &persona).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_persona(
    id: i64,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    persona_repo::delete_persona(&conn, id).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 创建 terminal_commands.rs**

创建 `src-tauri/src/commands/terminal_commands.rs`：

```rust
use std::sync::Mutex;
use tauri::State;
use crate::db::DbState;
use crate::models::{TerminalSession, CreateTerminalSession};
use crate::repositories::terminal_session_repo;
use crate::terminal::PtyManager;

pub struct PtyManagerState(pub Mutex<PtyManager>);

#[tauri::command]
pub async fn start_persona_rewrite(
    note_id: i64,
    note_path: String,
    persona_skill: String,
    mode: String,
    db: State<'_, DbState>,
    pty_manager: State<'_, PtyManagerState>,
) -> Result<String, String> {
    let conn = db.0.lock().unwrap();

    let create_session = CreateTerminalSession {
        note_id,
        persona_skill: persona_skill.clone(),
        mode: mode.clone(),
    };

    let session_id = terminal_session_repo::insert_terminal_session(&conn, &create_session)
        .map_err(|e| e.to_string())?;

    let manager = pty_manager.0.lock().unwrap();
    manager.create_session(
        session_id.clone(),
        note_id,
        &note_path,
        &persona_skill,
    ).map_err(|e| e.to_string())?;

    Ok(session_id)
}

#[tauri::command]
pub async fn update_session_status(
    session_id: String,
    status: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    terminal_session_repo::update_session_status(&conn, &session_id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_terminal_session(
    session_id: String,
    pty_manager: State<'_, PtyManagerState>,
) -> Result<(), String> {
    let manager = pty_manager.0.lock().unwrap();
    manager.close_session(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resize_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    pty_manager: State<'_, PtyManagerState>,
) -> Result<(), String> {
    let manager = pty_manager.0.lock().unwrap();
    manager.resize_session(&session_id, rows, cols).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_info(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Option<TerminalSession>, String> {
    let conn = db.0.lock().unwrap();
    terminal_session_repo::get_session_by_id(&conn, &session_id).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 在 commands/mod.rs 中导出**

在 `src-tauri/src/commands/mod.rs` 末尾添加：

```rust
pub mod persona_commands;
pub mod terminal_commands;
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands/persona_commands.rs src-tauri/src/commands/terminal_commands.rs src-tauri/src/commands/mod.rs
git commit -m "feat(persona): 实现人格和终端 Tauri 命令"
```

---

## Task 10: 注册所有 Tauri 命令并初始化 PTY 管理器

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 lib.rs 中添加 use 语句**

在 `src-tauri/src/lib.rs` 的 `use config::ConfigState;` 之后添加：

```rust
use commands::terminal_commands::PtyManagerState;
use terminal::PtyManager;
```

- [ ] **Step 2: 在 setup 钩子中初始化 PTY 管理器状态**

在 `src-tauri/src/lib.rs` 的 `.setup()` 钩子中，`app.manage(ConfigState(...))` 之后添加：

```rust
            app.manage(PtyManagerState(std::sync::Mutex::new(PtyManager::new())));
```

- [ ] **Step 3: 在 invoke_handler 中注册新命令**

在 `src-tauri/src/lib.rs` 的 `.invoke_handler(tauri::generate_handler![...])` 中，`commands::note_commands::delete_note,` 之后添加：

```rust
            commands::persona_commands::scan_local_personas,
            commands::persona_commands::get_all_personas,
            commands::persona_commands::get_persona_by_skill,
            commands::persona_commands::save_persona,
            commands::persona_commands::delete_persona,
            commands::terminal_commands::start_persona_rewrite,
            commands::terminal_commands::update_session_status,
            commands::terminal_commands::close_terminal_session,
            commands::terminal_commands::resize_terminal,
            commands::terminal_commands::get_session_info,
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(persona): 注册所有人格和终端 Tauri 命令"
```

---

## Task 11: 添加前端依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 xterm.js 依赖**

```bash
npm install xterm@^5.3.0 xterm-addon-fit@^0.8.0
```

- [ ] **Step 2: 安装 shadcn tabs 组件**

```bash
npx shadcn-vue@latest add tabs
```

Expected: `src/components/ui/tabs/` 目录生成，包含 TabsList、TabsTrigger、TabsContent 等组件。

- [ ] **Step 3: 验证依赖安装**

```bash
npm list xterm xterm-addon-fit
ls src/components/ui/tabs/
```

Expected: 显示已安装版本，tabs 目录存在

- [ ] **Step 4: 提交**

```bash
git add package.json package-lock.json src/components/ui/tabs/
git commit -m "feat(persona): 添加 xterm.js 终端依赖和 shadcn tabs 组件"
```

---

## Task 12: 创建前端类型定义

**Files:**
- Create: `src/types/persona.ts`
- Modify: `src/types/index.ts`

- [ ] **Step 1: 创建 persona.ts 类型文件**

创建 `src/types/persona.ts`：

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

- [ ] **Step 2: 在 types/index.ts 末尾添加导出**

在 `src/types/index.ts` 末尾添加：

```typescript
export * from './persona';
```

- [ ] **Step 3: 验证类型检查**

```bash
npm run build
```

Expected: 无类型错误

- [ ] **Step 4: 提交**

```bash
git add src/types/persona.ts src/types/index.ts
git commit -m "feat(persona): 添加人格和终端会话类型定义"
```

---

## Task 13: 创建前端 API 封装

**Files:**
- Create: `src/composables/usePersona.ts`

- [ ] **Step 1: 创建 usePersona.ts**

创建 `src/composables/usePersona.ts`：

```typescript
import { invoke } from '@tauri-apps/api/core';
import type {
  Persona,
  CreatePersona,
  TerminalSession,
  StartRewriteParams,
} from '../types/persona';

export function usePersona() {
  const scanLocalPersonas = async (): Promise<Persona[]> => {
    return await invoke<Persona[]>('scan_local_personas');
  };

  const getAllPersonas = async (): Promise<Persona[]> => {
    return await invoke<Persona[]>('get_all_personas');
  };

  const getPersonaBySkill = async (skillName: string): Promise<Persona | null> => {
    return await invoke<Persona | null>('get_persona_by_skill', { skillName });
  };

  const savePersona = async (persona: CreatePersona): Promise<number> => {
    return await invoke<number>('save_persona', { persona });
  };

  const deletePersona = async (id: number): Promise<void> => {
    await invoke('delete_persona', { id });
  };

  const startPersonaRewrite = async (params: StartRewriteParams): Promise<string> => {
    return await invoke<string>('start_persona_rewrite', params);
  };

  const updateSessionStatus = async (sessionId: string, status: string): Promise<void> => {
    await invoke('update_session_status', { sessionId, status });
  };

  const closeTerminalSession = async (sessionId: string): Promise<void> => {
    await invoke('close_terminal_session', { sessionId });
  };

  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    await invoke('resize_terminal', { sessionId, rows, cols });
  };

  const getSessionInfo = async (sessionId: string): Promise<TerminalSession | null> => {
    return await invoke<TerminalSession | null>('get_session_info', { sessionId });
  };

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
    getSessionInfo,
  };
}
```

- [ ] **Step 2: 验证类型检查**

```bash
npm run build
```

Expected: 无类型错误

- [ ] **Step 3: 提交**

```bash
git add src/composables/usePersona.ts
git commit -m "feat(persona): 添加人格 API 封装"
```

---

## Task 14: 创建人格选择对话框组件

**Files:**
- Create: `src/components/persona/PersonaDialog.vue`

- [ ] **Step 1: 创建 persona 组件目录**

```bash
mkdir -p src/components/persona
```

- [ ] **Step 2: 创建 PersonaDialog.vue**

创建 `src/components/persona/PersonaDialog.vue`：

```vue
<script setup lang="ts">
import { ref, watch } from 'vue';
import { usePersona } from '@/composables/usePersona';
import type { Persona } from '@/types/persona';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

interface Props {
  open: boolean;
  noteId: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:open': [value: boolean];
  confirm: [persona: Persona, mode: 'smart' | 'manual'];
}>();

const { scanLocalPersonas } = usePersona();

const personas = ref<Persona[]>([]);
const selectedPersona = ref<Persona | null>(null);
const loading = ref(false);
const error = ref('');

const loadPersonas = async () => {
  loading.value = true;
  error.value = '';
  try {
    personas.value = await scanLocalPersonas();
  } catch (e) {
    error.value = e instanceof Error ? e.message : '扫描人格失败';
  } finally {
    loading.value = false;
  }
};

const handleConfirm = () => {
  if (selectedPersona.value) {
    emit('confirm', selectedPersona.value, 'manual');
    emit('update:open', false);
  }
};

const handleOpenChange = (value: boolean) => {
  emit('update:open', value);
};

watch(() => props.open, (open) => {
  if (open) {
    loadPersonas();
    selectedPersona.value = null;
  }
});
</script>

<template>
  <Dialog :open="open" @update:open="handleOpenChange">
    <DialogContent class="sm:max-w-[560px]">
      <DialogHeader>
        <DialogTitle>人格撰写</DialogTitle>
      </DialogHeader>

      <Tabs default-value="manual">
        <TabsList>
          <TabsTrigger value="manual">手动选择</TabsTrigger>
          <TabsTrigger value="smart" disabled>智能选择</TabsTrigger>
        </TabsList>

        <TabsContent value="manual" class="mt-4">
          <div v-if="loading" class="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <div class="mb-3 h-8 w-8 animate-spin rounded-full border-4 border-muted border-t-primary" />
            <p class="text-sm">正在扫描本地人格...</p>
          </div>

          <div v-else-if="error" class="flex flex-col items-center justify-center py-12 text-destructive">
            <p class="mb-3">{{ error }}</p>
            <Button variant="outline" size="sm" @click="loadPersonas">重试</Button>
          </div>

          <div v-else-if="personas.length === 0" class="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <p>未找到可用的人格</p>
            <p class="mt-1 text-xs">请确保已安装 Claude Code 并配置了 perspective skills</p>
          </div>

          <div v-else class="grid grid-cols-2 gap-2.5">
            <div
              v-for="persona in personas"
              :key="persona.skillName"
              class="cursor-pointer rounded-lg border-2 p-3 transition-all hover:border-primary/50"
              :class="selectedPersona?.skillName === persona.skillName
                ? 'border-primary bg-primary/5 ring-1 ring-primary'
                : 'border-border'"
              @click="selectedPersona = persona"
            >
              <div class="mb-1 flex items-center justify-between">
                <h3 class="text-sm font-semibold">{{ persona.name }}</h3>
                <Badge v-if="persona.isBuiltin" variant="secondary" class="text-[10px]">内置</Badge>
              </div>
              <p class="mb-1 text-xs text-muted-foreground">{{ persona.category }}</p>
              <p class="line-clamp-2 text-xs text-foreground/70">{{ persona.description }}</p>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="smart" class="mt-4">
          <div class="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <div class="mb-3 text-4xl">🚧</div>
            <h3 class="mb-1 text-sm font-semibold">智能选择即将开放</h3>
            <p class="text-xs">AI 将自动分析笔记内容并推荐最合适的人格风格</p>
          </div>
        </TabsContent>
      </Tabs>

      <DialogFooter>
        <Button variant="outline" @click="handleOpenChange(false)">取消</Button>
        <Button :disabled="!selectedPersona" @click="handleConfirm">开始重构</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
```

- [ ] **Step 3: 验证编译**

```bash
npm run build
```

Expected: 无类型错误

- [ ] **Step 4: 提交**

```bash
git add src/components/persona/PersonaDialog.vue
git commit -m "feat(persona): 创建人格选择对话框组件"
```

---

## Task 15: 创建内嵌终端面板组件

**Files:**
- Create: `src/components/persona/TerminalPanel.vue`

- [ ] **Step 1: 创建 TerminalPanel.vue**

创建 `src/components/persona/TerminalPanel.vue`：

```vue
<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { usePersona } from '@/composables/usePersona';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import 'xterm/css/xterm.css';

interface Props {
  sessionId: string;
  noteId: number;
  personaName: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
  completed: [];
  failed: [error: string];
}>();

const { updateSessionStatus, closeTerminalSession, resizeTerminal } = usePersona();

const terminalRef = ref<HTMLElement | null>(null);
const expanded = ref(true);
const status = ref<'running' | 'completed' | 'failed'>('running');

let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlistenOutput: UnlistenFn | null = null;
let unlistenStatus: UnlistenFn | null = null;

const statusVariant = computed(() => {
  switch (status.value) {
    case 'running': return 'secondary' as const;
    case 'completed': return 'default' as const;
    case 'failed': return 'destructive' as const;
  }
});

const statusText = computed(() => {
  switch (status.value) {
    case 'running': return '执行中';
    case 'completed': return '已完成';
    case 'failed': return '失败';
  }
});

const initTerminal = () => {
  if (!terminalRef.value) return;

  terminal = new Terminal({
    cursorBlink: false,
    disableStdin: true,
    fontSize: 13,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
    },
  });

  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.open(terminalRef.value);
  nextTick(() => fitAddon?.fit());
};

const handleResize = () => {
  if (fitAddon && terminal) {
    fitAddon.fit();
    const { rows, cols } = terminal;
    resizeTerminal(props.sessionId, rows, cols).catch(console.error);
  }
};

const setupEventListeners = async () => {
  unlistenOutput = await listen<string>('pty-output', (event) => {
    if (terminal && event.payload) {
      terminal.write(event.payload);
    }
  });

  unlistenStatus = await listen<{ sessionId: string; status: string; error?: string }>(
    'session-status',
    async (event) => {
      if (event.payload.sessionId === props.sessionId) {
        const newStatus = event.payload.status as 'running' | 'completed' | 'failed';
        status.value = newStatus;
        if (newStatus === 'completed') {
          await updateSessionStatus(props.sessionId, 'completed');
          emit('completed');
        } else if (newStatus === 'failed') {
          await updateSessionStatus(props.sessionId, 'failed');
          emit('failed', event.payload.error || '执行失败');
        }
      }
    }
  );
};

const handleClose = async () => {
  await closeTerminalSession(props.sessionId).catch(console.error);
  cleanup();
  emit('close');
};

const cleanup = () => {
  unlistenOutput?.();
  unlistenStatus?.();
  unlistenOutput = null;
  unlistenStatus = null;
  if (terminal) {
    terminal.dispose();
    terminal = null;
  }
  window.removeEventListener('resize', handleResize);
};

onMounted(() => {
  initTerminal();
  setupEventListeners();
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  cleanup();
});
</script>

<template>
  <div class="border-t-2 border-primary bg-[#1e1e1e]">
    <!-- Header bar -->
    <div class="flex items-center justify-between border-b border-[#3e3e3e] bg-[#2d2d2d] px-4 py-1.5">
      <div class="flex items-center gap-2">
        <span
          class="inline-block h-2 w-2 rounded-full"
          :class="status === 'running' ? 'animate-pulse bg-yellow-400' : status === 'completed' ? 'bg-green-400' : 'bg-red-400'"
        />
        <span class="text-[13px] font-medium text-[#d4d4d4]">⚡ {{ personaName }}</span>
        <Badge variant="secondary" class="text-[11px] text-[#a0a0a0]">
          {{ statusText }}
        </Badge>
      </div>
      <div class="flex items-center gap-1">
        <Button
          v-if="status === 'completed'"
          variant="ghost"
          size="icon-xs"
          class="text-[#a0a0a0] hover:text-[#d4d4d4]"
          @click="handleClose"
        >
          <span class="text-xs">关闭</span>
        </Button>
        <Button
          v-if="status === 'failed'"
          variant="ghost"
          size="icon-xs"
          class="text-[#a0a0a0] hover:text-[#d4d4d4]"
          @click="handleClose"
        >
          <span class="text-xs">关闭</span>
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          class="text-[#a0a0a0] hover:text-[#d4d4d4]"
          @click="expanded = !expanded"
        >
          {{ expanded ? '收起' : '展开' }}
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          class="text-[#a0a0a0] hover:text-[#d4d4d4]"
          @click="handleClose"
        >
          ✕
        </Button>
      </div>
    </div>

    <!-- Terminal body -->
    <div v-show="expanded" ref="terminalRef" class="overflow-hidden p-2" style="max-height: 200px" />
  </div>
</template>
```

- [ ] **Step 2: 验证编译**

```bash
npm run build
```

Expected: 无类型错误

- [ ] **Step 3: 提交**

```bash
git add src/components/persona/TerminalPanel.vue
git commit -m "feat(persona): 创建内嵌终端面板组件"
```

---

## Task 16: 集成人格功能到笔记详情页面

**Files:**
- Modify: `src/components/notes/NoteDetail.vue`

- [ ] **Step 1: 添加导入**

在 `src/components/notes/NoteDetail.vue` 的 `<script setup>` 中，在 `import { Badge } from '@/components/ui/badge';` 之后添加：

```typescript
import PersonaDialog from '@/components/persona/PersonaDialog.vue';
import TerminalPanel from '@/components/persona/TerminalPanel.vue';
import { usePersona } from '@/composables/usePersona';
import type { Persona } from '@/types/persona';
```

- [ ] **Step 2: 添加人格相关状态**

在 `const emit = defineEmits<...>();` 之后添加：

```typescript
const { startPersonaRewrite } = usePersona();

const showPersonaDialog = ref(false);
const showTerminal = ref(false);
const currentSessionId = ref('');
const currentPersonaName = ref('');
```

- [ ] **Step 3: 添加人格处理函数**

在 `function formatDate(...)` 之后添加：

```typescript
async function handlePersonaConfirm(persona: Persona) {
  showPersonaDialog.value = false;
  try {
    const notePath = `~/.omnilink/notes/${props.noteDetail.note.file_name}`;
    const sessionId = await startPersonaRewrite({
      noteId: props.noteDetail.note.id,
      notePath,
      personaSkill: persona.skillName,
      mode: 'manual',
    });
    currentSessionId.value = sessionId;
    currentPersonaName.value = persona.name;
    showTerminal.value = true;
  } catch (error) {
    console.error('启动人格重构失败:', error);
  }
}

function handleTerminalClose() {
  showTerminal.value = false;
  currentSessionId.value = '';
  currentPersonaName.value = '';
}

async function handleRewriteCompleted() {
  const detail = await notesApi.getNote(props.noteDetail.note.id);
  props.noteDetail.note = detail.note;
  props.noteDetail.content = detail.content;
}

function handleRewriteFailed(error: string) {
  console.error('人格重构失败:', error);
}
```

- [ ] **Step 4: 启用"人格撰写"按钮**

将模板中的：

```vue
<Button variant="secondary" disabled title="功能开发中">
  人格撰写
</Button>
```

替换为：

```vue
<Button variant="secondary" @click="showPersonaDialog = true">
  ✨ 人格撰写
</Button>
```

- [ ] **Step 5: 在模板中添加 TerminalPanel**

在模板中 `<div class="flex-1">` 的 MdEditor div 之后、`</template>` 之前添加：

```vue
  <TerminalPanel
    v-if="showTerminal"
    :session-id="currentSessionId"
    :note-id="noteDetail.note.id"
    :persona-name="currentPersonaName"
    @close="handleTerminalClose"
    @completed="handleRewriteCompleted"
    @failed="handleRewriteFailed"
  />

  <PersonaDialog
    :open="showPersonaDialog"
    :note-id="noteDetail.note.id"
    @update:open="showPersonaDialog = $event"
    @confirm="handlePersonaConfirm"
  />
```

- [ ] **Step 6: 验证编译**

```bash
npm run build
```

Expected: 无类型错误

- [ ] **Step 7: 提交**

```bash
git add src/components/notes/NoteDetail.vue
git commit -m "feat(persona): 集成人格功能到笔记详情页面"
```

---

## 实现完成总结

**后端（Rust）：**
- 数据库 schema（personas、terminal_sessions 表）
- 数据模型（Persona、TerminalSession）
- 内置 skills（4 个人格文件 + 初始化逻辑）
- 人格扫描器（扫描内置 + 本地 Claude Code skills）
- 数据仓库（persona_repo、terminal_session_repo）
- PTY 管理器（创建、管理、关闭终端会话）
- Tauri 命令（10 个命令接口）

**前端（Vue 3 + shadcn-vue + Tailwind）：**
- 类型定义（Persona、TerminalSession 等）
- API 封装（usePersona composable）
- 人格选择对话框（shadcn Dialog + Tabs）
- 内嵌终端面板（xterm.js + 可折叠）
- 笔记详情页面集成

**下一步（Phase 2 - 人格商店）：**
- 独立的人格管理页面
- 人格分类、搜索、详情查看
- 从本地 skills 目录导入人格

**下一步（Phase 3 - 女娲造人 + 智能选择）：**
- 创建新人格功能
- 启用智能选择 Tab
- AI 自动匹配最合适的人格




