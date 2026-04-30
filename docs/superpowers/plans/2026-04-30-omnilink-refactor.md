# OmniLink 整体重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 OmniLink 从「链接采集 + 知识管理」重构为极简 Markdown 笔记应用，保留笔记、链接解析（融入笔记）和人格系统。

**Architecture:** 渐进式重构——保留现有笔记组件（NotesView、MarkdownEditor、NoteDetail、PersonaDialog、TerminalPanel），删除知识库/标签相关代码，重写 AppLayout 和路由，新增「从链接创建笔记」功能和主题系统。

**Tech Stack:** Tauri v2 + Vue 3 + Rust + SQLite + md-editor-v3

---

## 文件结构总览

### 后端（src-tauri/src/）

| 文件 | 操作 | 说明 |
|------|------|------|
| `db/schema.rs` | 修改 | 添加 source_url 迁移，删除旧表，清理旧迁移代码 |
| `models/mod.rs` | 修改 | Note 模型添加 source_url，移除 Link/Content/Tag/AiResult 等类型 |
| `repositories/note_repo.rs` | 修改 | 添加 create_note_from_link，更新 row_to_note |
| `parser/pipeline.rs` | 重写 | 不再写入 DB，返回解析结果供上层使用 |
| `commands/note_commands.rs` | 修改 | 添加 create_note_from_link 命令 |
| `commands/link_commands.rs` | 删除 | |
| `commands/content_commands.rs` | 删除 | |
| `commands/tag_commands.rs` | 删除 | |
| `repositories/link_repo.rs` | 删除 | |
| `repositories/content_repo.rs` | 删除 | |
| `repositories/tag_repo.rs` | 删除 | |
| `repositories/mod.rs` | 修改 | 移除已删除模块的引用 |
| `commands/mod.rs` | 修改 | 移除已删除模块的引用 |
| `lib.rs` | 修改 | 移除旧命令注册，添加新命令 |

### 前端（src/）

| 文件 | 操作 | 说明 |
|------|------|------|
| `types/index.ts` | 修改 | 移除 Link/Content/Tag 类型，添加 source_url 到 Note |
| `composables/useApi.ts` | 修改 | 移除旧 API，添加 createNoteFromLink，保留 notesApi |
| `views/LinksView.vue` | 删除 | |
| `views/TagsView.vue` | 删除 | |
| `views/ContentView.vue` | 删除 | |
| `components/LinkCard.vue` | 删除 | |
| `components/LinkCardSkeleton.vue` | 删除 | |
| `components/AddLinkDialog.vue` | 删除 | |
| `components/StickyHeader.vue` | 删除 | |
| `components/TagInput.vue` | 删除 | |
| `stores/links.ts` | 删除 | |
| `stores/tags.ts` | 删除 | |
| `components/AppLayout.vue` | 重写 | 新侧边栏 + 主题切换 + 收起/展开 |
| `router/index.ts` | 重写 | 精简路由 |
| `views/NotesView.vue` | 修改 | 集成新侧边栏布局 |
| `components/notes/NotesList.vue` | 修改 | 添加新建按钮（空白/从链接） |
| `components/notes/NoteDetail.vue` | 修改 | 顶栏添加人格按钮和主题切换 |
| `components/CreateNoteDialog.vue` | 新建 | 从链接创建笔记的对话框 |
| `composables/useTheme.ts` | 新建 | 主题切换 composable |

---

## Task 1: 后端 Schema 迁移

**Files:**
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: 更新 init_schema，移除旧表定义**

将 `init_schema` 中的建表语句精简，只保留 `user_settings`、`notes`、`personas`、`terminal_sessions` 四张表。移除 `links`、`contents`、`tags`、`content_tags`、`ai_results`、`import_history` 的 CREATE TABLE 语句，以及相关的索引。

```rust
// src-tauri/src/db/schema.rs
use rusqlite::Connection;

use crate::error::AppResult;

pub fn init_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            file_name TEXT NOT NULL UNIQUE,
            source_url TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            file_size INTEGER DEFAULT 0,
            word_count INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);

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
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );",
    )?;

    migrate(conn)?;

    Ok(())
}
```

- [ ] **Step 2: 更新 migrate 函数**

替换现有迁移逻辑，添加数据迁移（将旧 links+contents 转为 notes）和旧表清理：

```rust
// src-tauri/src/db/schema.rs (migrate function)
fn migrate(conn: &Connection) -> AppResult<()> {
    // notes_storage_path default
    conn.execute(
        "INSERT OR IGNORE INTO user_settings (key, value) VALUES ('notes_storage_path', '~/.omnilink/notes/')",
        [],
    )?;

    // Migrate existing links+contents to notes (if old tables exist)
    migrate_links_to_notes(conn)?;

    // Drop old tables
    drop_old_tables(conn)?;

    Ok(())
}

fn migrate_links_to_notes(conn: &Connection) -> AppResult<()> {
    // Check if links table exists
    let has_links: bool = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='links'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if !has_links {
        return Ok(());
    }

    // Get notes storage path
    let notes_dir: String = conn
        .query_row(
            "SELECT value FROM user_settings WHERE key = 'notes_storage_path'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "~/.omnilink/notes/".to_string());

    let expanded_dir = shellexpand::tilde(&notes_dir).to_string();
    let notes_path = std::path::PathBuf::from(&expanded_dir);
    std::fs::create_dir_all(&notes_path)?;

    // Migrate each link with content to a note
    let mut stmt = conn.prepare(
        "SELECT l.id, l.url, l.title, c.body_text
         FROM links l
         LEFT JOIN contents c ON c.link_id = l.id
         WHERE c.body_text IS NOT NULL AND c.body_text != ''",
    )?;

    let rows: Vec<(i64, String, Option<String>, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (link_id, url, title, body_text) in rows {
        let note_title = title.unwrap_or_else(|| format!("链接笔记 {}", link_id));
        let content = body_text.unwrap_or_default();
        let timestamp = chrono::Utc::now().timestamp();
        let file_name = format!("note_from_link_{}_{}.md", timestamp, link_id);

        // Insert note
        conn.execute(
            "INSERT INTO notes (title, file_name, source_url, created_at, updated_at, file_size, word_count)
             VALUES (?, ?, ?, datetime('now'), datetime('now'), ?, ?)",
            rusqlite::params![
                note_title,
                file_name,
                url,
                content.len() as i64,
                content.split_whitespace().count() as i64
            ],
        )?;

        // Write file
        let file_path = notes_path.join(&file_name);
        std::fs::write(&file_path, &content)?;
    }

    tracing::info!("Migrated links+contents to notes");
    Ok(())
}

fn drop_old_tables(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS content_tags;
         DROP TABLE IF EXISTS ai_results;
         DROP TABLE IF EXISTS contents;
         DROP TABLE IF EXISTS tags;
         DROP TABLE IF EXISTS import_history;
         DROP TABLE IF EXISTS links;
         DROP INDEX IF EXISTS idx_links_status;
         DROP INDEX IF EXISTS idx_links_platform;
         DROP INDEX IF EXISTS idx_links_created;
         DROP INDEX IF EXISTS idx_contents_link;",
    )?;
    tracing::info!("Dropped old tables");
    Ok(())
}
```

- [ ] **Step 3: 验证编译通过**

Run: `cd src-tauri && cargo check 2>&1 | head -30`
Expected: 会有其他文件的编译错误（因为移除了旧表引用），但 schema.rs 本身应无错误

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/db/schema.rs
git commit -m "refactor(db): 精简 schema，添加 source_url 字段和旧数据迁移"
```

---

## Task 2: 后端 Model 更新

**Files:**
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: 精简 models/mod.rs**

移除 Link、Content、ContentParsed、AiResult、AiResultParsed、LinkDetail、LinksResponse、CreateLinkInput、Tag、TagWithCount、CreateTagInput、UpdateContentInput 等类型。保留 Note 和 NoteDetail，为 Note 添加 `source_url` 字段。

```rust
// src-tauri/src/models/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub file_name: String,
    pub source_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_size: i64,
    pub word_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteDetail {
    pub note: Note,
    pub content: String,
}

pub mod persona;
pub mod terminal_session;
```

- [ ] **Step 2: 更新 note_repo.rs 中的 row_to_note**

在 `row_to_note` 函数中添加 `source_url` 字段读取。注意列索引：id=0, title=1, file_name=2, source_url=3, created_at=4, updated_at=5, file_size=6, word_count=7。

```rust
// src-tauri/src/repositories/note_repo.rs - row_to_note function
fn row_to_note(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        file_name: row.get(2)?,
        source_url: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        file_size: row.get(6)?,
        word_count: row.get(7)?,
    })
}
```

同时更新 `list_notes` 的 SQL，显式列出列名以确保顺序正确：

```rust
// src-tauri/src/repositories/note_repo.rs - list_notes function
pub fn list_notes(conn: &Connection, limit: i64, offset: i64) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, file_name, source_url, created_at, updated_at, file_size, word_count
         FROM notes ORDER BY updated_at DESC LIMIT ? OFFSET ?",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit, offset], row_to_note)?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row?);
    }
    Ok(notes)
}
```

同样更新 `get_note_by_id`：

```rust
// src-tauri/src/repositories/note_repo.rs - get_note_by_id function
pub fn get_note_by_id(conn: &Connection, id: i64) -> AppResult<Option<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, file_name, source_url, created_at, updated_at, file_size, word_count
         FROM notes WHERE id = ?",
    )?;
    let note = stmt.query_row(rusqlite::params![id], row_to_note).ok();
    Ok(note)
}
```

- [ ] **Step 3: 验证编译通过**

Run: `cd src-tauri && cargo check 2>&1 | head -30`
Expected: 仍有旧模块引用的错误，但 models 和 note_repo 应无错

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/models/mod.rs src-tauri/src/repositories/note_repo.rs
git commit -m "refactor(models): 精简数据模型，Note 添加 source_url 字段"
```

---

## Task 3: 适配解析管道

**Files:**
- Modify: `src-tauri/src/parser/pipeline.rs`

- [ ] **Step 1: 重写 pipeline.rs**

将 pipeline 从「写入 DB 的编排器」改为「返回解析结果的纯函数」。不再依赖 link_repo 和 content_repo，而是返回一个 `ParseOutput` 结构体。

```rust
// src-tauri/src/parser/pipeline.rs
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::config::ConfigState;
use crate::error::{AppError, AppResult};

use super::{content_judge, identifier, llm_extractor, webview_extractor};

#[derive(Serialize)]
pub struct ParseOutput {
    pub url: String,
    pub title: String,
    pub markdown: String,
    pub platform: String,
}

pub async fn parse_url_to_markdown(
    app: &AppHandle,
    config_state: &State<'_, ConfigState>,
    url: &str,
) -> AppResult<ParseOutput> {
    tracing::info!("Starting parse for url={}", url);

    let platform = identifier::identify_platform(url).to_string();

    // Step 1: WebView extraction
    let extract_result = webview_extractor::extract_via_webview(app, url).await
        .map_err(|e| {
            tracing::error!("Content extraction failed: url={}, error={}", url, e);
            AppError::Parse(format!("内容提取失败: {}", e))
        })?;

    let title = if extract_result.title.is_empty() {
        url.to_string()
    } else {
        extract_result.title.clone()
    };

    // Step 2: Judge content sufficiency
    let judgement = content_judge::judge_content(&title, &extract_result.markdown);

    let final_markdown = if judgement.is_sufficient {
        extract_result.markdown.clone()
    } else {
        // Step 3: LLM fallback
        tracing::warn!("Content insufficient, falling back to LLM: reason='{}'", judgement.reason);
        let ai_config = config_state.0.lock().unwrap().ai.clone();
        match llm_extractor::extract_via_llm(&ai_config, &extract_result.raw_html, url).await {
            Ok(llm_result) => llm_result.markdown,
            Err(e) => {
                tracing::error!("LLM extraction failed: error={}", e);
                extract_result.markdown.clone()
            }
        }
    };

    Ok(ParseOutput {
        url: url.to_string(),
        title,
        markdown: final_markdown,
        platform,
    })
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cd src-tauri && cargo check 2>&1 | head -30`
Expected: 旧的 `parse_link` 函数被移除，pipeline 不再依赖 link_repo/content_repo

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/parser/pipeline.rs
git commit -m "refactor(parser): pipeline 改为返回解析结果，不再写入 DB"
```

---

## Task 4: 后端新增 create_note_from_link 命令

**Files:**
- Modify: `src-tauri/src/commands/note_commands.rs`
- Modify: `src-tauri/src/repositories/note_repo.rs`

- [ ] **Step 1: 在 note_repo.rs 中添加 create_note_with_content 函数**

```rust
// src-tauri/src/repositories/note_repo.rs - add after delete_note function
pub fn create_note_with_content(
    conn: &Connection,
    title: &str,
    content: &str,
    source_url: &str,
) -> AppResult<Note> {
    let timestamp = chrono::Utc::now().timestamp();
    let temp_file_name = format!("note_{}_temp.md", timestamp);

    conn.execute(
        "INSERT INTO notes (title, file_name, source_url) VALUES (?, ?, ?)",
        rusqlite::params![title, temp_file_name, source_url],
    )?;

    let id = conn.last_insert_rowid();
    let file_name = format!("note_{}_{}.md", timestamp, id);

    conn.execute(
        "UPDATE notes SET file_name = ? WHERE id = ?",
        rusqlite::params![file_name, id],
    )?;

    let notes_dir = get_notes_dir(conn)?;
    fs::create_dir_all(&notes_dir)?;
    let file_path = safe_join(&notes_dir, &file_name)?;
    fs::write(&file_path, content)?;

    let file_size = content.len() as i64;
    let word_count = content.split_whitespace().count() as i64;
    conn.execute(
        "UPDATE notes SET file_size = ?, word_count = ? WHERE id = ?",
        rusqlite::params![file_size, word_count, id],
    )?;

    get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))
}
```

- [ ] **Step 2: 在 note_commands.rs 中添加 create_note_from_link 命令**

```rust
// src-tauri/src/commands/note_commands.rs - add at the end
use crate::config::ConfigState;
use crate::parser::pipeline;

#[tauri::command]
pub async fn create_note_from_link(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    config_state: State<'_, ConfigState>,
    url: String,
) -> AppResult<NoteDetail> {
    let parse_output = pipeline::parse_url_to_markdown(&app, &config_state, &url).await?;

    let conn = state.0.lock().unwrap();
    let note = note_repo::create_note_with_content(
        &conn,
        &parse_output.title,
        &parse_output.markdown,
        &parse_output.url,
    )?;

    let content = note_repo::read_note_content(&conn, note.id)?;
    Ok(NoteDetail { note, content })
}
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/note_commands.rs src-tauri/src/repositories/note_repo.rs
git commit -m "feat(note): 添加 create_note_from_link 命令"
```

---

## Task 5: 后端删除旧模块并更新注册

**Files:**
- Delete: `src-tauri/src/commands/link_commands.rs`
- Delete: `src-tauri/src/commands/content_commands.rs`
- Delete: `src-tauri/src/commands/tag_commands.rs`
- Delete: `src-tauri/src/repositories/link_repo.rs`
- Delete: `src-tauri/src/repositories/content_repo.rs`
- Delete: `src-tauri/src/repositories/tag_repo.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/repositories/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 删除旧文件**

```bash
rm src-tauri/src/commands/link_commands.rs
rm src-tauri/src/commands/content_commands.rs
rm src-tauri/src/commands/tag_commands.rs
rm src-tauri/src/repositories/link_repo.rs
rm src-tauri/src/repositories/content_repo.rs
rm src-tauri/src/repositories/tag_repo.rs
```

- [ ] **Step 2: 更新 commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod note_commands;
pub mod persona_commands;
pub mod settings_commands;
pub mod terminal_commands;
```

- [ ] **Step 3: 更新 repositories/mod.rs**

```rust
// src-tauri/src/repositories/mod.rs
pub mod note_repo;
pub mod persona_repo;
pub mod settings_repo;
pub mod terminal_session_repo;
pub mod ai_result_repo;
```

注：`ai_result_repo.rs` 保留（可能被其他模块引用），如果编译报错再删除。

- [ ] **Step 4: 更新 lib.rs 命令注册**

移除 link_commands、content_commands、tag_commands 的注册，添加 `create_note_from_link`：

```rust
// src-tauri/src/lib.rs - invoke_handler section
.invoke_handler(tauri::generate_handler![
    commands::note_commands::create_note,
    commands::note_commands::get_note,
    commands::note_commands::list_notes,
    commands::note_commands::update_note,
    commands::note_commands::delete_note,
    commands::note_commands::create_note_from_link,
    commands::settings_commands::get_settings,
    commands::settings_commands::get_ai_config,
    commands::settings_commands::update_ai_config,
    commands::persona_commands::scan_local_personas,
    commands::persona_commands::get_all_personas,
    commands::persona_commands::get_persona_by_skill,
    commands::persona_commands::save_persona,
    commands::persona_commands::delete_persona,
    commands::terminal_commands::start_persona_rewrite,
    commands::terminal_commands::start_reading,
    commands::terminal_commands::update_session_status,
    commands::terminal_commands::close_terminal_session,
    commands::terminal_commands::resize_terminal,
    commands::terminal_commands::get_session_info,
])
```

- [ ] **Step 5: 清理 ai_result_repo 如果不再需要**

检查 `ai_result_repo.rs` 是否被任何保留的模块引用。如果无人引用，删除它。

- [ ] **Step 6: 验证编译通过**

Run: `cd src-tauri && cargo check 2>&1 | tail -20`
Expected: 编译通过，无错误

- [ ] **Step 7: 提交**

```bash
git add -A src-tauri/
git commit -m "refactor(backend): 删除链接/标签/内容相关模块，精简后端代码"
```

---

## Task 6: 前端类型和 API 更新

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/composables/useApi.ts`

- [ ] **Step 1: 精简 types/index.ts**

移除 Link、Content、AiResult、LinkDetail、Tag、TagWithCount 类型。为 Note 添加 source_url。

```typescript
// src/types/index.ts
export interface Note {
  id: number;
  title: string;
  file_name: string;
  source_url: string | null;
  created_at: string;
  updated_at: string;
  file_size: number;
  word_count: number;
}

export interface NoteDetail {
  note: Note;
  content: string;
}

export * from './persona';
```

- [ ] **Step 2: 精简 useApi.ts**

移除所有链接、标签、内容相关的 API 方法。保留 settings API 和 notesApi，添加 `createNoteFromLink`。

```typescript
// src/composables/useApi.ts
import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteDetail } from '../types/index';

export function useApi() {
  return {
    getSettings: () =>
      invoke<Record<string, string>>('get_settings'),

    getAiConfig: () =>
      invoke<{ provider: string; openai: { api_key: string; base_url: string; model: string }; ollama: { base_url: string; model: string } }>('get_ai_config'),

    updateAiConfig: (params: {
      provider: string;
      openaiApiKey?: string;
      openaiBaseUrl?: string;
      openaiModel?: string;
      ollamaBaseUrl?: string;
      ollamaModel?: string;
    }) =>
      invoke<void>('update_ai_config', {
        provider: params.provider,
        openaiApiKey: params.openaiApiKey ?? null,
        openaiBaseUrl: params.openaiBaseUrl ?? null,
        openaiModel: params.openaiModel ?? null,
        ollamaBaseUrl: params.ollamaBaseUrl ?? null,
        ollamaModel: params.ollamaModel ?? null,
      }),
  };
}

export const notesApi = {
  async createNote(title: string): Promise<Note> {
    return invoke('create_note', { title });
  },

  async getNote(id: number): Promise<NoteDetail> {
    return invoke('get_note', { id });
  },

  async listNotes(limit?: number, offset?: number): Promise<Note[]> {
    return invoke('list_notes', { limit, offset });
  },

  async updateNote(id: number, title: string, content: string): Promise<void> {
    return invoke('update_note', { id, title, content });
  },

  async deleteNote(id: number): Promise<void> {
    return invoke('delete_note', { id });
  },

  async createNoteFromLink(url: string): Promise<NoteDetail> {
    return invoke('create_note_from_link', { url });
  },
};
```

- [ ] **Step 3: 提交**

```bash
git add src/types/index.ts src/composables/useApi.ts
git commit -m "refactor(frontend): 精简类型定义和 API 封装，移除链接/标签相关代码"
```

---

## Task 7: 删除旧前端文件

**Files:**
- Delete: `src/views/LinksView.vue`
- Delete: `src/views/TagsView.vue`
- Delete: `src/views/ContentView.vue`
- Delete: `src/components/LinkCard.vue`
- Delete: `src/components/LinkCardSkeleton.vue`
- Delete: `src/components/AddLinkDialog.vue`
- Delete: `src/components/StickyHeader.vue`
- Delete: `src/components/TagInput.vue`
- Delete: `src/stores/links.ts`
- Delete: `src/stores/tags.ts`

- [ ] **Step 1: 删除文件**

```bash
rm src/views/LinksView.vue
rm src/views/TagsView.vue
rm src/views/ContentView.vue
rm src/components/LinkCard.vue
rm src/components/LinkCardSkeleton.vue
rm src/components/AddLinkDialog.vue
rm src/components/StickyHeader.vue
rm src/components/TagInput.vue
rm src/stores/links.ts
rm src/stores/tags.ts
```

- [ ] **Step 2: 更新 router/index.ts**

```typescript
// src/router/index.ts
import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/notes' },
    { path: '/notes', name: 'notes', component: () => import('../views/NotesView.vue') },
    { path: '/notes/:id', name: 'note-detail', component: () => import('../views/NotesView.vue'), props: true },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  ],
});

export default router;
```

- [ ] **Step 3: 更新 AppLayout.vue（临时最小版本）**

先写一个临时的最小 AppLayout，确保路由能正常工作。完整重写在 Task 9。

```vue
<!-- src/components/AppLayout.vue -->
<template>
  <router-view />
</template>
```

- [ ] **Step 4: 验证前端能启动**

Run: `npm run dev` 并在浏览器中访问
Expected: 应用能启动，笔记页面正常显示（虽然布局还不对）

- [ ] **Step 5: 提交**

```bash
git add -A src/
git commit -m "refactor(frontend): 删除知识库/图谱相关文件，精简路由"
```

---

## Task 8: 创建主题系统

**Files:**
- Create: `src/composables/useTheme.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 创建 useTheme.ts**

```typescript
// src/composables/useTheme.ts
import { ref, watch, onMounted } from 'vue';

type Theme = 'light' | 'dark';

const theme = ref<Theme>('light');

export function useTheme() {
  function applyTheme(t: Theme) {
    document.documentElement.setAttribute('data-theme', t);
    theme.value = t;
  }

  function toggleTheme() {
    const next = theme.value === 'light' ? 'dark' : 'light';
    applyTheme(next);
    localStorage.setItem('omnilink-theme', next);
  }

  function initTheme() {
    const saved = localStorage.getItem('omnilink-theme') as Theme | null;
    if (saved) {
      applyTheme(saved);
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      applyTheme('dark');
    } else {
      applyTheme('light');
    }
  }

  return {
    theme,
    toggleTheme,
    initTheme,
  };
}
```

- [ ] **Step 2: 在 App.vue 中初始化主题**

在 `App.vue` 的 `<script setup>` 中调用 `initTheme()`：

```vue
<!-- src/App.vue -->
<script setup lang="ts">
import { useTheme } from './composables/useTheme';

const { initTheme } = useTheme();
initTheme();
</script>
```

- [ ] **Step 3: 添加 CSS 变量基础样式**

在全局 CSS 文件中（如 `src/style.css` 或 `src/assets/main.css`）添加亮/暗主题变量。先检查项目中是否有全局 CSS 文件。

```css
:root,
[data-theme="light"] {
  --background: #ffffff;
  --foreground: #0a0a0a;
  --muted: #f5f5f5;
  --muted-foreground: #737373;
  --border: #e5e5e5;
  --primary: #6366f1;
  --primary-foreground: #ffffff;
  --accent: #f5f5f5;
  --accent-foreground: #0a0a0a;
}

[data-theme="dark"] {
  --background: #0a0a0a;
  --foreground: #fafafa;
  --muted: #262626;
  --muted-foreground: #a3a3a3;
  --border: #262626;
  --primary: #818cf8;
  --primary-foreground: #0a0a0a;
  --accent: #262626;
  --accent-foreground: #fafafa;
}
```

- [ ] **Step 4: 提交**

```bash
git add src/composables/useTheme.ts src/App.vue
git commit -m "feat(theme): 添加亮色/暗色主题切换系统"
```

---

## Task 9: 重写 AppLayout 和侧边栏

**Files:**
- Rewrite: `src/components/AppLayout.vue`

- [ ] **Step 1: 重写 AppLayout.vue**

```vue
<!-- src/components/AppLayout.vue -->
<template>
  <div class="app-layout">
    <aside v-show="sidebarOpen" class="sidebar">
      <div class="sidebar-search">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索笔记..."
          class="sidebar-search-input"
        />
      </div>

      <div class="sidebar-new-btn">
        <div class="new-note-wrapper">
          <button class="btn-new-note" @click="handleCreateBlank">
            + 新建笔记
          </button>
          <button class="btn-new-dropdown" @click="showNewMenu = !showNewMenu">
            ▾
          </button>
        </div>
        <div v-if="showNewMenu" class="new-menu">
          <button class="new-menu-item" @click="handleCreateBlank">空白笔记</button>
          <button class="new-menu-item" @click="handleCreateFromLink">从链接创建</button>
        </div>
      </div>

      <div class="sidebar-notes-list">
        <div v-if="notesStore.loading" class="sidebar-empty">加载中...</div>
        <div v-else-if="filteredNotes.length === 0" class="sidebar-empty">
          {{ searchQuery ? '未找到匹配的笔记' : '暂无笔记' }}
        </div>
        <div
          v-for="note in filteredNotes"
          :key="note.id"
          class="sidebar-note-item"
          :class="{ active: note.id === currentNoteId }"
          @click="goToNote(note.id)"
        >
          <div class="note-title">{{ note.title || '未命名笔记' }}</div>
          <div class="note-time">{{ formatTime(note.updated_at) }}</div>
        </div>
      </div>

      <div class="sidebar-footer">
        <button class="btn-icon" @click="goToSettings">⚙️</button>
      </div>
    </aside>

    <main class="main-area">
      <div class="main-topbar">
        <button class="btn-icon" @click="sidebarOpen = !sidebarOpen">≡</button>
        <span class="topbar-title">{{ currentTitle }}</span>
        <div class="topbar-actions">
          <slot name="topbar-actions" />
          <button class="btn-icon" @click="toggleTheme">
            {{ theme === 'light' ? '☀️' : '🌙' }}
          </button>
        </div>
      </div>

      <div class="main-content">
        <slot />
      </div>
    </main>

    <CreateNoteDialog
      v-if="showCreateFromLink"
      @close="showCreateFromLink = false"
      @created="handleLinkNoteCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useNotesStore } from '../stores/notes';
import { useTheme } from '../composables/useTheme';
import CreateNoteDialog from './CreateNoteDialog.vue';

const router = useRouter();
const route = useRoute();
const notesStore = useNotesStore();
const { theme, toggleTheme } = useTheme();

const sidebarOpen = ref(true);
const searchQuery = ref('');
const showNewMenu = ref(false);
const showCreateFromLink = ref(false);

const currentNoteId = computed(() => {
  const id = route.params.id;
  return id ? Number(id) : null;
});

const currentTitle = computed(() => {
  if (!currentNoteId.value) return 'OmniLink';
  const note = notesStore.notes.find(n => n.id === currentNoteId.value);
  return note?.title || '未命名笔记';
});

const filteredNotes = computed(() => {
  if (!searchQuery.value) return notesStore.notes;
  const q = searchQuery.value.toLowerCase();
  return notesStore.notes.filter(n => n.title.toLowerCase().includes(q));
});

onMounted(() => {
  notesStore.fetchNotes();
});

function goToNote(id: number) {
  router.push(`/notes/${id}`);
}

function goToSettings() {
  router.push('/settings');
}

async function handleCreateBlank() {
  showNewMenu.value = false;
  try {
    const note = await notesStore.createNote('未命名笔记');
    router.push(`/notes/${note.id}`);
  } catch (e) {
    console.error('Failed to create note:', e);
  }
}

function handleCreateFromLink() {
  showNewMenu.value = false;
  showCreateFromLink.value = true;
}

function handleLinkNoteCreated(noteId: number) {
  notesStore.fetchNotes();
  router.push(`/notes/${noteId}`);
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return '刚刚';
  if (diffMin < 60) return `${diffMin}分钟前`;
  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour}小时前`;
  const diffDay = Math.floor(diffHour / 24);
  if (diffDay < 7) return `${diffDay}天前`;
  return date.toLocaleDateString('zh-CN');
}
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  background: var(--background);
  color: var(--foreground);
}

.sidebar {
  width: 260px;
  min-width: 260px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--background);
}

.sidebar-search {
  padding: 12px;
}

.sidebar-search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--muted);
  color: var(--foreground);
  font-size: 13px;
  outline: none;
}

.sidebar-search-input:focus {
  border-color: var(--primary);
}

.sidebar-new-btn {
  padding: 0 12px 8px;
  position: relative;
}

.new-note-wrapper {
  display: flex;
}

.btn-new-note {
  flex: 1;
  padding: 8px 12px;
  background: var(--primary);
  color: var(--primary-foreground);
  border: none;
  border-radius: 6px 0 0 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-new-dropdown {
  padding: 8px 8px;
  background: var(--primary);
  color: var(--primary-foreground);
  border: none;
  border-left: 1px solid rgba(255,255,255,0.2);
  border-radius: 0 6px 6px 0;
  cursor: pointer;
  font-size: 13px;
}

.new-menu {
  position: absolute;
  top: 100%;
  left: 12px;
  right: 12px;
  background: var(--background);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  z-index: 10;
  overflow: hidden;
}

.new-menu-item {
  display: block;
  width: 100%;
  padding: 10px 12px;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  font-size: 13px;
  color: var(--foreground);
}

.new-menu-item:hover {
  background: var(--accent);
}

.sidebar-notes-list {
  flex: 1;
  overflow-y: auto;
}

.sidebar-empty {
  padding: 32px 12px;
  text-align: center;
  color: var(--muted-foreground);
  font-size: 13px;
}

.sidebar-note-item {
  padding: 12px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.15s;
}

.sidebar-note-item:hover {
  background: var(--accent);
}

.sidebar-note-item.active {
  border-left-color: var(--primary);
  background: var(--accent);
}

.note-title {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-time {
  font-size: 11px;
  color: var(--muted-foreground);
  margin-top: 4px;
}

.sidebar-footer {
  padding: 12px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: center;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-topbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  min-height: 44px;
}

.topbar-title {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 18px;
  padding: 4px 8px;
  border-radius: 4px;
  color: var(--foreground);
}

.btn-icon:hover {
  background: var(--accent);
}

.main-content {
  flex: 1;
  overflow: hidden;
}
</style>
```

- [ ] **Step 2: 更新 NotesView.vue 使用 AppLayout**

```vue
<!-- src/views/NotesView.vue -->
<template>
  <AppLayout>
    <template #topbar-actions>
      <button
        v-if="notesStore.currentNote"
        class="btn-persona"
        @click="showPersonaDialog = true"
      >
        👤
      </button>
    </template>

    <NoteDetail
      v-if="notesStore.currentNote"
      :note-detail="notesStore.currentNote"
      @save="handleSaveNote"
      @delete="handleDeleteNote"
    />
    <div v-else class="empty-state">
      <p>选择一个笔记或创建新笔记</p>
    </div>

    <PersonaDialog
      v-if="notesStore.currentNote"
      :open="showPersonaDialog"
      :note-id="notesStore.currentNote.note.id"
      @update:open="showPersonaDialog = $event"
      @confirm="handlePersonaConfirm"
    />
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useNotesStore } from '../stores/notes';
import AppLayout from '../components/AppLayout.vue';
import NoteDetail from '../components/notes/NoteDetail.vue';
import PersonaDialog from '../components/persona/PersonaDialog.vue';

const route = useRoute();
const router = useRouter();
const notesStore = useNotesStore();
const showPersonaDialog = ref(false);

watch(
  () => route.params.id,
  async (id) => {
    if (id) {
      await notesStore.loadNote(Number(id));
    } else {
      notesStore.currentNote = null;
    }
  },
  { immediate: true }
);

async function handleSaveNote(id: number, title: string, content: string) {
  try {
    await notesStore.saveNote(id, title, content);
  } catch (e) {
    console.error('Failed to save note:', e);
  }
}

async function handleDeleteNote(id: number) {
  try {
    await notesStore.deleteNote(id);
    router.push('/notes');
  } catch (e) {
    console.error('Failed to delete note:', e);
  }
}

function handlePersonaConfirm() {
  // Persona processing started, reload note after completion
  if (route.params.id) {
    notesStore.loadNote(Number(route.params.id));
  }
}
</script>

<style scoped>
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--muted-foreground);
}

.btn-persona {
  background: none;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 4px 10px;
  cursor: pointer;
  font-size: 16px;
}

.btn-persona:hover {
  background: var(--accent);
}
</style>
```

- [ ] **Step 3: 更新 NoteDetail.vue 移除重复的顶栏按钮**

NoteDetail 组件中现有的保存/删除/人格按钮需要移除（已移至 AppLayout 顶栏和 NotesView），只保留标题编辑器和 Markdown 编辑器。PersonaDialog 也移至 NotesView。

- [ ] **Step 4: 提交**

```bash
git add src/components/AppLayout.vue src/views/NotesView.vue src/components/notes/NoteDetail.vue
git commit -m "refactor(ui): 重写 AppLayout 侧边栏，集成笔记列表和主题切换"
```

---

## Task 10: 创建「从链接创建笔记」对话框

**Files:**
- Create: `src/components/CreateNoteDialog.vue`

- [ ] **Step 1: 创建 CreateNoteDialog.vue**

```vue
<!-- src/components/CreateNoteDialog.vue -->
<template>
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog-box">
      <h3 class="dialog-title">从链接创建笔记</h3>
      <input
        v-model="url"
        type="text"
        placeholder="粘贴 URL..."
        class="dialog-input"
        :disabled="loading"
        @keyup.enter="handleCreate"
      />
      <div v-if="error" class="dialog-error">{{ error }}</div>
      <div class="dialog-actions">
        <button class="btn-cancel" @click="$emit('close')" :disabled="loading">取消</button>
        <button class="btn-confirm" @click="handleCreate" :disabled="!url.trim() || loading">
          {{ loading ? '解析中...' : '解析创建' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { notesApi } from '../composables/useApi';

const emit = defineEmits<{
  close: [];
  created: [noteId: number];
}>();

const url = ref('');
const loading = ref(false);
const error = ref<string | null>(null);

async function handleCreate() {
  if (!url.value.trim() || loading.value) return;

  loading.value = true;
  error.value = null;

  try {
    const result = await notesApi.createNoteFromLink(url.value.trim());
    emit('created', result.note.id);
    emit('close');
  } catch (e) {
    error.value = e instanceof Error ? e.message : '解析失败，请检查 URL 或重试';
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog-box {
  background: var(--background);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
  width: 400px;
  max-width: 90vw;
}

.dialog-title {
  margin: 0 0 16px;
  font-size: 16px;
  font-weight: 600;
}

.dialog-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--muted);
  color: var(--foreground);
  font-size: 14px;
  outline: none;
  box-sizing: border-box;
}

.dialog-input:focus {
  border-color: var(--primary);
}

.dialog-error {
  margin-top: 8px;
  color: #ef4444;
  font-size: 13px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.btn-cancel {
  padding: 8px 16px;
  background: none;
  border: 1px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  color: var(--foreground);
  font-size: 13px;
}

.btn-confirm {
  padding: 8px 16px;
  background: var(--primary);
  color: var(--primary-foreground);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-confirm:disabled,
.btn-cancel:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/CreateNoteDialog.vue
git commit -m "feat(note): 添加从链接创建笔记对话框"
```

---

## Task 11: 更新 NoteDetail 组件

**Files:**
- Modify: `src/components/notes/NoteDetail.vue`

- [ ] **Step 1: 精简 NoteDetail.vue**

移除顶栏按钮（保存、删除、人格撰写、查看终端），这些功能已移至 AppLayout 顶栏和 NotesView。只保留标题输入和 Markdown 编辑器。

```vue
<!-- src/components/notes/NoteDetail.vue -->
<template>
  <div class="note-detail">
    <input
      v-model="localTitle"
      type="text"
      class="note-title-input"
      placeholder="笔记标题"
      @blur="handleSave"
    />
    <div class="note-meta">
      <span>创建于 {{ formatDate(noteDetail.note.created_at) }}</span>
      <span>更新于 {{ formatDate(noteDetail.note.updated_at) }}</span>
      <a
        v-if="noteDetail.note.source_url"
        :href="noteDetail.note.source_url"
        target="_blank"
        class="source-link"
      >
        来源链接
      </a>
    </div>
    <div class="note-editor">
      <MdEditor
        v-model="localContent"
        :language="'zh-CN'"
        :style="{ height: 'calc(100vh - 180px)' }"
        @update:model-value="handleContentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { NoteDetail } from '../../types/index';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

interface Props {
  noteDetail: NoteDetail;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [id: number, title: string, content: string];
  delete: [id: number];
}>();

const localTitle = ref(props.noteDetail.note.title);
const localContent = ref(props.noteDetail.content);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.noteDetail,
  (val) => {
    localTitle.value = val.note.title;
    localContent.value = val.content;
  }
);

function handleSave() {
  emit('save', props.noteDetail.note.id, localTitle.value, localContent.value);
}

function handleContentChange() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    handleSave();
  }, 1000);
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleString('zh-CN');
}
</script>

<style scoped>
.note-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px 24px;
}

.note-title-input {
  width: 100%;
  border: none;
  background: transparent;
  font-size: 24px;
  font-weight: 600;
  outline: none;
  color: var(--foreground);
}

.note-meta {
  display: flex;
  gap: 16px;
  padding: 8px 0;
  font-size: 12px;
  color: var(--muted-foreground);
  border-bottom: 1px solid var(--border);
  margin-bottom: 12px;
}

.source-link {
  color: var(--primary);
  text-decoration: none;
}

.source-link:hover {
  text-decoration: underline;
}

.note-editor {
  flex: 1;
  overflow: hidden;
}
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/notes/NoteDetail.vue
git commit -m "refactor(note): 精简 NoteDetail，移除重复按钮，添加来源链接显示"
```

---

## Task 12: 最终验证和清理

**Files:**
- 全项目检查

- [ ] **Step 1: 验证 Rust 后端编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无错误

- [ ] **Step 2: 验证前端类型检查**

Run: `npm run build`
Expected: TypeScript 类型检查通过，无错误

- [ ] **Step 3: 启动开发模式验证**

Run: `npm run dev:all`
Expected: 应用启动正常，在浏览器中验证：
- 侧边栏显示笔记列表
- 新建空白笔记正常
- 从链接创建笔记对话框正常打开
- 主题切换正常
- 设置页面正常访问
- 点击笔记正常加载编辑器

- [ ] **Step 4: 检查并清理残留引用**

在前端搜索是否有残留的旧类型引用：
```bash
grep -r "LinkCard\|LinkDetail\|TagWithCount\|useLinksStore\|useTagsStore" src/ --include="*.ts" --include="*.vue"
```
Expected: 无结果

- [ ] **Step 5: 更新 CLAUDE.md**

更新 CLAUDE.md 中的架构说明，反映重构后的新结构：
- 移除知识库/标签相关的描述
- 更新路由说明
- 更新数据库表说明

- [ ] **Step 6: 最终提交**

```bash
git add -A
git commit -m "refactor: 完成 OmniLink 整体重构，定位为极简 Markdown 笔记应用"
```
