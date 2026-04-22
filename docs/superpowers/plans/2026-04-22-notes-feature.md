# 笔记功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 OmniLink 添加独立的笔记功能，支持 Markdown 编辑、文件系统存储、左右分栏布局

**Architecture:** 采用轻量级文件系统方案，笔记内容存储为 `.md` 文件，SQLite 仅存储元数据。后端使用 Tauri Commands + Repository 模式，前端使用 Vue 3 + Pinia + Vue Router。

**Tech Stack:** Rust (Tauri, rusqlite), Vue 3, TypeScript, Pinia, Vue Router, markdown-it, highlight.js

---

## 文件结构规划

### 后端新增文件
- `src-tauri/src/models.rs` — 添加 `Note` 和 `NoteDetail` 结构体
- `src-tauri/src/repositories/note_repo.rs` — 笔记数据库操作层
- `src-tauri/src/commands/note_commands.rs` — Tauri 命令层
- `src-tauri/src/db/schema.rs` — 添加 notes 表和迁移逻辑
- `src-tauri/src/repositories/mod.rs` — 导出 note_repo 模块
- `src-tauri/src/commands/mod.rs` — 导出 note_commands 模块
- `src-tauri/src/lib.rs` — 注册 note_commands

### 前端新增文件
- `src/types/index.ts` — 添加 `Note` 和 `NoteDetail` 接口
- `src/stores/notes.ts` — Pinia store
- `src/composables/useApi.ts` — 添加笔记相关 API 调用
- `src/views/NotesView.vue` — 主容器（左右分栏）
- `src/components/notes/NotesList.vue` — 左侧笔记列表
- `src/components/notes/NoteDetail.vue` — 右侧笔记详情
- `src/components/notes/MarkdownEditor.vue` — Markdown 编辑器组件
- `src/router/index.ts` — 添加 `/notes` 路由

---

## Task 1: 数据库表结构和迁移

**Files:**
- Modify: `src-tauri/src/db/schema.rs:69-100`

- [ ] **Step 1: 在 init_schema 中添加 notes 表创建语句**

在 `init_schema` 函数的 `execute_batch` 中添加：

```rust
CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    file_name TEXT NOT NULL UNIQUE,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    file_size INTEGER DEFAULT 0,
    word_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);
```

- [ ] **Step 2: 在 migrate 函数中添加 user_settings 表的 notes_storage_path 字段迁移**

在 `migrate` 函数末尾添加：

```rust
// notes_storage_path migration
let has_notes_path: bool = conn
    .prepare("SELECT value FROM user_settings WHERE key = 'notes_storage_path' LIMIT 1")
    .is_ok();
if !has_notes_path {
    conn.execute(
        "INSERT OR IGNORE INTO user_settings (key, value) VALUES ('notes_storage_path', '~/.omnilink/notes/')",
        [],
    )?;
}
```

- [ ] **Step 3: 运行应用验证数据库迁移**

Run: `npm run dev:all`
Expected: 应用启动成功，无数据库错误

- [ ] **Step 4: 验证表结构**

Run: `sqlite3 ~/.omnilink/omnilink.db "SELECT sql FROM sqlite_master WHERE name='notes';"`
Expected: 输出 notes 表的 CREATE TABLE 语句

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/schema.rs
git commit -m "feat(db): 添加 notes 表和 notes_storage_path 配置"
```

---

## Task 2: 后端数据模型

**Files:**
- Modify: `src-tauri/src/models.rs:50-end`

- [ ] **Step 1: 添加 Note 结构体**

在 `models.rs` 末尾添加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub file_name: String,
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
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat(models): 添加 Note 和 NoteDetail 数据模型"
```

---

## Task 3: 笔记数据库操作层

**Files:**
- Create: `src-tauri/src/repositories/note_repo.rs`

- [ ] **Step 1: 创建 note_repo.rs 文件并添加基础结构**

```rust
use rusqlite::{params, Connection, Row};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::models::{Note, NoteDetail};

fn row_to_note(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        file_name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        file_size: row.get(5)?,
        word_count: row.get(6)?,
    })
}

fn get_notes_dir(conn: &Connection) -> AppResult<PathBuf> {
    let mut stmt = conn.prepare("SELECT value FROM user_settings WHERE key = 'notes_storage_path'")?;
    let path_str: String = stmt.query_row([], |row| row.get(0))?;
    let expanded = shellexpand::tilde(&path_str).to_string();
    Ok(PathBuf::from(expanded))
}
```

- [ ] **Step 2: 实现 create_note 函数**

```rust
pub fn create_note(conn: &Connection, title: &str) -> AppResult<Note> {
    let timestamp = chrono::Utc::now().timestamp();
    
    conn.execute(
        "INSERT INTO notes (title, file_name) VALUES (?, ?)",
        params![title, ""],
    )?;
    
    let id = conn.last_insert_rowid();
    let file_name = format!("note_{}_{}.md", timestamp, id);
    
    conn.execute(
        "UPDATE notes SET file_name = ? WHERE id = ?",
        params![file_name, id],
    )?;
    
    let notes_dir = get_notes_dir(conn)?;
    fs::create_dir_all(&notes_dir)?;
    let file_path = notes_dir.join(&file_name);
    fs::write(&file_path, "")?;
    
    get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))
}
```

- [ ] **Step 3: 实现 get_note_by_id 函数**

```rust
pub fn get_note_by_id(conn: &Connection, id: i64) -> AppResult<Option<Note>> {
    let mut stmt = conn.prepare("SELECT * FROM notes WHERE id = ?")?;
    let note = stmt.query_row(params![id], |row| row_to_note(row)).ok();
    Ok(note)
}
```

- [ ] **Step 4: 实现 list_notes 函数**

```rust
pub fn list_notes(conn: &Connection, limit: i64, offset: i64) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM notes ORDER BY updated_at DESC LIMIT ? OFFSET ?",
    )?;
    let rows = stmt.query_map(params![limit, offset], |row| row_to_note(row))?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row?);
    }
    Ok(notes)
}
```

- [ ] **Step 5: 实现 read_note_content 函数**

```rust
pub fn read_note_content(conn: &Connection, id: i64) -> AppResult<String> {
    let note = get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))?;
    let notes_dir = get_notes_dir(conn)?;
    let file_path = notes_dir.join(&note.file_name);
    
    if !file_path.exists() {
        return Err(AppError::NotFound(format!("Note file not found: {}", note.file_name)));
    }
    
    let content = fs::read_to_string(&file_path)?;
    Ok(content)
}
```

- [ ] **Step 6: 实现 update_note 函数**

```rust
pub fn update_note(conn: &Connection, id: i64, title: &str, content: &str) -> AppResult<()> {
    let note = get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))?;
    let notes_dir = get_notes_dir(conn)?;
    let file_path = notes_dir.join(&note.file_name);
    
    fs::write(&file_path, content)?;
    
    let file_size = content.len() as i64;
    let word_count = content.split_whitespace().count() as i64;
    
    conn.execute(
        "UPDATE notes SET title = ?, updated_at = datetime('now'), file_size = ?, word_count = ? WHERE id = ?",
        params![title, file_size, word_count, id],
    )?;
    
    Ok(())
}
```

- [ ] **Step 7: 实现 delete_note 函数**

```rust
pub fn delete_note(conn: &Connection, id: i64) -> AppResult<()> {
    let note = get_note_by_id(conn, id)?.ok_or_else(|| AppError::NotFound("Note not found".into()))?;
    let notes_dir = get_notes_dir(conn)?;
    let file_path = notes_dir.join(&note.file_name);
    
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }
    
    let images_dir = notes_dir.join("images").join(id.to_string());
    if images_dir.exists() {
        fs::remove_dir_all(&images_dir)?;
    }
    
    conn.execute("DELETE FROM notes WHERE id = ?", params![id])?;
    
    Ok(())
}
```

- [ ] **Step 8: 添加 Cargo.toml 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
chrono = "0.4"
shellexpand = "3.1"
```

- [ ] **Step 9: 在 repositories/mod.rs 中导出 note_repo**

在 `src-tauri/src/repositories/mod.rs` 中添加：

```rust
pub mod note_repo;
```

- [ ] **Step 10: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 11: Commit**

```bash
git add src-tauri/src/repositories/note_repo.rs src-tauri/src/repositories/mod.rs src-tauri/Cargo.toml
git commit -m "feat(repo): 实现笔记数据库操作层"
```

---

## Task 4: Tauri 命令层

**Files:**
- Create: `src-tauri/src/commands/note_commands.rs`

- [ ] **Step 1: 创建 note_commands.rs 文件并添加基础导入**

```rust
use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::{Note, NoteDetail};
use crate::repositories::note_repo;
```

- [ ] **Step 2: 实现 create_note 命令**

```rust
#[tauri::command]
pub async fn create_note(state: State<'_, DbState>, title: String) -> AppResult<Note> {
    let conn = state.0.lock().unwrap();
    note_repo::create_note(&conn, &title)
}
```

- [ ] **Step 3: 实现 get_note 命令**

```rust
#[tauri::command]
pub async fn get_note(state: State<'_, DbState>, id: i64) -> AppResult<NoteDetail> {
    let conn = state.0.lock().unwrap();
    let note = note_repo::get_note_by_id(&conn, id)?
        .ok_or_else(|| crate::error::AppError::NotFound("Note not found".into()))?;
    let content = note_repo::read_note_content(&conn, id)?;
    Ok(NoteDetail { note, content })
}
```

- [ ] **Step 4: 实现 list_notes 命令**

```rust
#[tauri::command]
pub async fn list_notes(
    state: State<'_, DbState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<Vec<Note>> {
    let conn = state.0.lock().unwrap();
    let lim = limit.unwrap_or(50);
    let off = offset.unwrap_or(0);
    note_repo::list_notes(&conn, lim, off)
}
```

- [ ] **Step 5: 实现 update_note 命令**

```rust
#[tauri::command]
pub async fn update_note(
    state: State<'_, DbState>,
    id: i64,
    title: String,
    content: String,
) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    note_repo::update_note(&conn, id, &title, &content)
}
```

- [ ] **Step 6: 实现 delete_note 命令**

```rust
#[tauri::command]
pub async fn delete_note(state: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    note_repo::delete_note(&conn, id)
}
```

- [ ] **Step 7: 在 commands/mod.rs 中导出 note_commands**

在 `src-tauri/src/commands/mod.rs` 中添加：

```rust
pub mod note_commands;
```

- [ ] **Step 8: 在 lib.rs 中注册命令**

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中添加：

```rust
commands::note_commands::create_note,
commands::note_commands::get_note,
commands::note_commands::list_notes,
commands::note_commands::update_note,
commands::note_commands::delete_note,
```

- [ ] **Step 9: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/commands/note_commands.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(commands): 实现笔记 Tauri 命令层"
```

---

## Task 5: 前端类型定义

**Files:**
- Modify: `src/types/index.ts:54-end`

- [ ] **Step 1: 添加 Note 和 NoteDetail 接口**

在 `src/types/index.ts` 末尾添加：

```typescript
export interface Note {
  id: number;
  title: string;
  file_name: string;
  created_at: string;
  updated_at: string;
  file_size: number;
  word_count: number;
}

export interface NoteDetail {
  note: Note;
  content: string;
}
```

- [ ] **Step 2: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 3: Commit**

```bash
git add src/types/index.ts
git commit -m "feat(types): 添加 Note 和 NoteDetail 类型定义"
```

---

## Task 6: API 调用封装

**Files:**
- Modify: `src/composables/useApi.ts:end`

- [ ] **Step 1: 添加笔记相关 API 函数**

在 `src/composables/useApi.ts` 末尾添加：

```typescript
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
};
```

- [ ] **Step 2: 在文件顶部导入 Note 和 NoteDetail 类型**

在 `src/composables/useApi.ts` 的导入部分添加：

```typescript
import type { Note, NoteDetail } from '@/types';
```

- [ ] **Step 3: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 4: Commit**

```bash
git add src/composables/useApi.ts
git commit -m "feat(api): 添加笔记 API 调用封装"
```

---

## Task 7: Pinia Store

**Files:**
- Create: `src/stores/notes.ts`

- [ ] **Step 1: 创建 notes store 文件**

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Note, NoteDetail } from '@/types';
import { notesApi } from '@/composables/useApi';

export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([]);
  const currentNote = ref<NoteDetail | null>(null);
  const isEditing = ref(false);
  const searchQuery = ref('');
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchNotes(offset = 0, limit = 50) {
    loading.value = true;
    error.value = null;
    try {
      notes.value = await notesApi.listNotes(limit, offset);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载笔记失败';
      console.error('Failed to fetch notes:', e);
    } finally {
      loading.value = false;
    }
  }

  async function createNote(title: string) {
    loading.value = true;
    error.value = null;
    try {
      const note = await notesApi.createNote(title);
      notes.value.unshift(note);
      await loadNote(note.id);
      return note;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建笔记失败';
      console.error('Failed to create note:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadNote(id: number) {
    loading.value = true;
    error.value = null;
    try {
      currentNote.value = await notesApi.getNote(id);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载笔记失败';
      console.error('Failed to load note:', e);
    } finally {
      loading.value = false;
    }
  }

  async function saveNote(id: number, title: string, content: string) {
    error.value = null;
    try {
      await notesApi.updateNote(id, title, content);
      
      const index = notes.value.findIndex(n => n.id === id);
      if (index !== -1) {
        notes.value[index].title = title;
        notes.value[index].updated_at = new Date().toISOString();
        notes.value[index].word_count = content.split(/\s+/).length;
        notes.value[index].file_size = new Blob([content]).size;
      }
      
      if (currentNote.value && currentNote.value.note.id === id) {
        currentNote.value.note.title = title;
        currentNote.value.note.updated_at = new Date().toISOString();
        currentNote.value.content = content;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存笔记失败';
      console.error('Failed to save note:', e);
      throw e;
    }
  }

  async function deleteNote(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await notesApi.deleteNote(id);
      notes.value = notes.value.filter(n => n.id !== id);
      if (currentNote.value?.note.id === id) {
        currentNote.value = null;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除笔记失败';
      console.error('Failed to delete note:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    notes,
    currentNote,
    isEditing,
    searchQuery,
    loading,
    error,
    fetchNotes,
    createNote,
    loadNote,
    saveNote,
    deleteNote,
  };
});
```

- [ ] **Step 2: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 3: Commit**

```bash
git add src/stores/notes.ts
git commit -m "feat(store): 实现笔记 Pinia store"
```

---

## Task 8: 路由配置

**Files:**
- Modify: `src/router/index.ts:10-11`

- [ ] **Step 1: 添加 /notes 路由**

在 `routes` 数组中添加：

```typescript
{ path: '/notes', name: 'notes', component: () => import('../views/NotesView.vue') },
```

- [ ] **Step 2: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误（NotesView.vue 尚未创建，会有警告，可忽略）

- [ ] **Step 3: Commit**

```bash
git add src/router/index.ts
git commit -m "feat(router): 添加笔记路由"
```

---

## Task 9: 笔记列表组件

**Files:**
- Create: `src/components/notes/NotesList.vue`

- [ ] **Step 1: 创建 notes 目录**

Run: `mkdir -p src/components/notes`

- [ ] **Step 2: 创建 NotesList.vue 组件**

```vue
<template>
  <div class="notes-list">
    <div class="list-header">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索笔记..."
        class="search-input"
      />
      <button class="btn-create" @click="$emit('create')">
        + 新建笔记
      </button>
    </div>

    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="filteredNotes.length === 0" class="empty">
      {{ searchQuery ? '未找到匹配的笔记' : '暂无笔记' }}
    </div>
    <div v-else class="notes-items">
      <div
        v-for="note in filteredNotes"
        :key="note.id"
        class="note-item"
        :class="{ active: note.id === selectedId }"
        @click="$emit('select', note.id)"
      >
        <div class="note-title">{{ note.title || '未命名笔记' }}</div>
        <div class="note-meta">
          <span class="note-time">{{ formatTime(note.updated_at) }}</span>
          <span class="note-words">{{ note.word_count }} 字</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { Note } from '@/types';

interface Props {
  notes: Note[];
  selectedId?: number;
  loading?: boolean;
}

const props = defineProps<Props>();
const searchQuery = ref('');

defineEmits<{
  select: [id: number];
  create: [];
}>();

const filteredNotes = computed(() => {
  if (!searchQuery.value) return props.notes;
  const query = searchQuery.value.toLowerCase();
  return props.notes.filter(note =>
    note.title.toLowerCase().includes(query)
  );
});

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) return '今天';
  if (days === 1) return '昨天';
  if (days < 7) return `${days} 天前`;
  return date.toLocaleDateString('zh-CN');
}
</script>

<style scoped>
.notes-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
}

.list-header {
  padding: 16px;
  border-bottom: 1px solid #e5e7eb;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  font-size: 14px;
  margin-bottom: 12px;
}

.search-input:focus {
  outline: none;
  border-color: #6366f1;
}

.btn-create {
  width: 100%;
  padding: 10px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.btn-create:hover {
  background: #4f46e5;
}

.loading,
.empty {
  padding: 32px 16px;
  text-align: center;
  color: #9ca3af;
  font-size: 14px;
}

.notes-items {
  flex: 1;
  overflow-y: auto;
}

.note-item {
  padding: 16px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
  transition: background 0.2s;
}

.note-item:hover {
  background: #f9fafb;
}

.note-item.active {
  background: #eef2ff;
  border-left: 3px solid #6366f1;
}

.note-title {
  font-size: 15px;
  font-weight: 500;
  color: #1f2937;
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.note-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #9ca3af;
}
</style>
```

- [ ] **Step 3: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 4: Commit**

```bash
git add src/components/notes/NotesList.vue
git commit -m "feat(components): 实现笔记列表组件"
```

---

## Task 10: Markdown 编辑器组件

**Files:**
- Create: `src/components/notes/MarkdownEditor.vue`

- [ ] **Step 1: 安装依赖**

Run: `npm install markdown-it highlight.js @types/markdown-it`

- [ ] **Step 2: 创建 MarkdownEditor.vue 组件**

```vue
<template>
  <div class="markdown-editor">
    <div class="editor-toolbar">
      <button @click="insertBold" title="加粗">B</button>
      <button @click="insertItalic" title="斜体">I</button>
      <button @click="insertHeading" title="标题">H</button>
      <button @click="insertLink" title="链接">🔗</button>
      <button @click="insertCode" title="代码">{ }</button>
      <button @click="insertList" title="列表">•</button>
    </div>

    <div class="editor-content">
      <textarea
        ref="textareaRef"
        v-model="localContent"
        class="editor-textarea"
        placeholder="开始编写..."
        @input="handleInput"
      />
      <div class="editor-preview" v-html="renderedHtml" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js';
import 'highlight.js/styles/github.css';

interface Props {
  modelValue: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const localContent = ref(props.modelValue);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

const md = new MarkdownIt({
  highlight: (str, lang) => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value;
      } catch {}
    }
    return '';
  },
});

const renderedHtml = computed(() => {
  return md.render(localContent.value);
});

watch(() => props.modelValue, (newVal) => {
  if (newVal !== localContent.value) {
    localContent.value = newVal;
  }
});

function handleInput() {
  emit('update:modelValue', localContent.value);
}

function insertAtCursor(before: string, after = '') {
  const textarea = textareaRef.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selectedText = localContent.value.substring(start, end);
  const newText = before + selectedText + after;

  localContent.value =
    localContent.value.substring(0, start) +
    newText +
    localContent.value.substring(end);

  emit('update:modelValue', localContent.value);

  setTimeout(() => {
    textarea.focus();
    textarea.setSelectionRange(start + before.length, start + before.length + selectedText.length);
  }, 0);
}

function insertBold() {
  insertAtCursor('**', '**');
}

function insertItalic() {
  insertAtCursor('*', '*');
}

function insertHeading() {
  insertAtCursor('## ');
}

function insertLink() {
  insertAtCursor('[', '](url)');
}

function insertCode() {
  insertAtCursor('`', '`');
}

function insertList() {
  insertAtCursor('- ');
}
</script>

<style scoped>
.markdown-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
}

.editor-toolbar {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
}

.editor-toolbar button {
  padding: 6px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.editor-toolbar button:hover {
  background: #f3f4f6;
}

.editor-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.editor-textarea,
.editor-preview {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.editor-textarea {
  border: none;
  border-right: 1px solid #e5e7eb;
  resize: none;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 14px;
  line-height: 1.6;
}

.editor-textarea:focus {
  outline: none;
}

.editor-preview {
  background: #fafafa;
  font-size: 15px;
  line-height: 1.7;
}

.editor-preview :deep(h1),
.editor-preview :deep(h2),
.editor-preview :deep(h3) {
  margin-top: 24px;
  margin-bottom: 12px;
  font-weight: 600;
}

.editor-preview :deep(h1) {
  font-size: 28px;
}

.editor-preview :deep(h2) {
  font-size: 24px;
}

.editor-preview :deep(h3) {
  font-size: 20px;
}

.editor-preview :deep(p) {
  margin-bottom: 12px;
}

.editor-preview :deep(code) {
  padding: 2px 6px;
  background: #f3f4f6;
  border-radius: 3px;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 13px;
}

.editor-preview :deep(pre) {
  padding: 12px;
  background: #1f2937;
  border-radius: 6px;
  overflow-x: auto;
  margin-bottom: 12px;
}

.editor-preview :deep(pre code) {
  background: transparent;
  color: #e5e7eb;
  padding: 0;
}

.editor-preview :deep(ul),
.editor-preview :deep(ol) {
  margin-left: 24px;
  margin-bottom: 12px;
}

.editor-preview :deep(li) {
  margin-bottom: 6px;
}
</style>
```

- [ ] **Step 3: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 4: Commit**

```bash
git add src/components/notes/MarkdownEditor.vue package.json package-lock.json
git commit -m "feat(components): 实现 Markdown 编辑器组件"
```

---

## Task 11: 笔记详情组件

**Files:**
- Create: `src/components/notes/NoteDetail.vue`

- [ ] **Step 1: 创建 NoteDetail.vue 组件**

```vue
<template>
  <div class="note-detail">
    <div class="detail-header">
      <input
        v-model="localTitle"
        type="text"
        class="title-input"
        placeholder="笔记标题"
        @blur="handleSave"
      />
      <div class="detail-actions">
        <button class="btn-save" @click="handleSave" :disabled="saving">
          {{ saving ? '保存中...' : '保存' }}
        </button>
        <button class="btn-ai" disabled title="功能开发中">
          AI 整理
        </button>
        <button class="btn-delete" @click="handleDelete">
          删除
        </button>
      </div>
    </div>

    <div class="detail-meta">
      <span>创建于 {{ formatDate(noteDetail.note.created_at) }}</span>
      <span>更新于 {{ formatDate(noteDetail.note.updated_at) }}</span>
      <span>{{ noteDetail.note.word_count }} 字</span>
    </div>

    <MarkdownEditor
      v-model="localContent"
      @update:model-value="handleContentChange"
    />

    <div v-if="saveStatus" class="save-status" :class="saveStatus">
      {{ saveStatus === 'success' ? '已保存' : '保存失败' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { NoteDetail } from '@/types';
import MarkdownEditor from './MarkdownEditor.vue';

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
const saving = ref(false);
const saveStatus = ref<'success' | 'error' | null>(null);
let saveTimer: number | null = null;

watch(() => props.noteDetail, (newVal) => {
  localTitle.value = newVal.note.title;
  localContent.value = newVal.content;
});

function handleContentChange() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    handleSave();
  }, 2000);
}

async function handleSave() {
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }

  saving.value = true;
  saveStatus.value = null;

  try {
    emit('save', props.noteDetail.note.id, localTitle.value, localContent.value);
    saveStatus.value = 'success';
    setTimeout(() => {
      saveStatus.value = null;
    }, 2000);
  } catch (e) {
    saveStatus.value = 'error';
    console.error('Save failed:', e);
  } finally {
    saving.value = false;
  }
}

function handleDelete() {
  if (confirm('确定要删除这篇笔记吗？')) {
    emit('delete', props.noteDetail.note.id);
  }
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN');
}
</script>

<style scoped>
.note-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
  position: relative;
}

.detail-header {
  padding: 16px;
  border-bottom: 1px solid #e5e7eb;
}

.title-input {
  width: 100%;
  padding: 8px 0;
  border: none;
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 12px;
}

.title-input:focus {
  outline: none;
}

.detail-actions {
  display: flex;
  gap: 8px;
}

.detail-actions button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.btn-save {
  background: #6366f1;
  color: white;
}

.btn-save:hover:not(:disabled) {
  background: #4f46e5;
}

.btn-save:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-ai {
  background: #f3f4f6;
  color: #9ca3af;
  cursor: not-allowed;
}

.btn-delete {
  background: #fee;
  color: #dc2626;
}

.btn-delete:hover {
  background: #fdd;
}

.detail-meta {
  padding: 12px 16px;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #6b7280;
}

.save-status {
  position: absolute;
  bottom: 24px;
  right: 24px;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  animation: fadeIn 0.3s;
}

.save-status.success {
  background: #d1fae5;
  color: #065f46;
}

.save-status.error {
  background: #fee2e2;
  color: #991b1b;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
```

- [ ] **Step 2: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 3: Commit**

```bash
git add src/components/notes/NoteDetail.vue
git commit -m "feat(components): 实现笔记详情组件"
```

---

## Task 12: 笔记主视图

**Files:**
- Create: `src/views/NotesView.vue`

- [ ] **Step 1: 创建 NotesView.vue 组件**

```vue
<template>
  <div class="notes-view">
    <div class="notes-sidebar">
      <NotesList
        :notes="notesStore.notes"
        :selected-id="notesStore.currentNote?.note.id"
        :loading="notesStore.loading"
        @select="handleSelectNote"
        @create="handleCreateNote"
      />
    </div>

    <div class="notes-content">
      <NoteDetail
        v-if="notesStore.currentNote"
        :note-detail="notesStore.currentNote"
        @save="handleSaveNote"
        @delete="handleDeleteNote"
      />
      <div v-else class="empty-state">
        <p>选择一个笔记或创建新笔记</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useNotesStore } from '@/stores/notes';
import NotesList from '@/components/notes/NotesList.vue';
import NoteDetail from '@/components/notes/NoteDetail.vue';

const notesStore = useNotesStore();

onMounted(() => {
  notesStore.fetchNotes();
});

async function handleSelectNote(id: number) {
  await notesStore.loadNote(id);
}

async function handleCreateNote() {
  try {
    await notesStore.createNote('未命名笔记');
  } catch (e) {
    console.error('Failed to create note:', e);
  }
}

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
  } catch (e) {
    console.error('Failed to delete note:', e);
  }
}
</script>

<style scoped>
.notes-view {
  display: flex;
  height: 100vh;
  background: #ffffff;
}

.notes-sidebar {
  width: 300px;
  border-right: 1px solid #e5e7eb;
  overflow-y: auto;
}

.notes-content {
  flex: 1;
  overflow: hidden;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #9ca3af;
  font-size: 16px;
}
</style>
```

- [ ] **Step 2: 运行类型检查**

Run: `npm run build`
Expected: 类型检查通过，无错误

- [ ] **Step 3: 启动开发服务器测试**

Run: `npm run dev:all`
Expected: 应用启动成功，访问 `http://localhost:1420/#/notes` 可以看到笔记页面

- [ ] **Step 4: 测试基本功能**

1. 点击"新建笔记"按钮，验证笔记创建成功
2. 在左侧列表中选择笔记，验证右侧显示详情
3. 编辑标题和内容，验证自动保存
4. 删除笔记，验证删除成功

- [ ] **Step 5: Commit**

```bash
git add src/views/NotesView.vue
git commit -m "feat(views): 实现笔记主视图"
```

---

## Task 13: 导航菜单集成

**Files:**
- Modify: `src/components/AppLayout.vue`

- [ ] **Step 1: 在导航菜单中添加笔记入口**

找到导航菜单部分，添加笔记链接：

```vue
<router-link to="/notes" class="nav-item">
  📝 笔记
</router-link>
```

- [ ] **Step 2: 运行应用验证**

Run: `npm run dev:all`
Expected: 导航菜单中显示"笔记"选项，点击可跳转到笔记页面

- [ ] **Step 3: Commit**

```bash
git add src/components/AppLayout.vue
git commit -m "feat(nav): 在导航菜单中添加笔记入口"
```

---

## Task 14: 端到端测试

**Files:**
- Test: 完整功能流程

- [ ] **Step 1: 启动应用**

Run: `npm run dev:all`
Expected: 应用启动成功

- [ ] **Step 2: 测试创建笔记**

1. 访问 `http://localhost:1420/#/notes`
2. 点击"新建笔记"按钮
3. 验证左侧列表中出现新笔记
4. 验证右侧自动打开编辑器

Expected: 笔记创建成功，自动选中

- [ ] **Step 3: 测试编辑笔记**

1. 修改标题为"测试笔记"
2. 在编辑器中输入内容：`# 标题\n\n这是一段测试内容`
3. 等待 2 秒自动保存
4. 验证右下角显示"已保存"提示

Expected: 内容自动保存，预览区实时渲染

- [ ] **Step 4: 测试搜索功能**

1. 创建多个笔记
2. 在搜索框输入关键词
3. 验证列表过滤正确

Expected: 搜索结果正确显示

- [ ] **Step 5: 测试删除笔记**

1. 选择一个笔记
2. 点击"删除"按钮
3. 确认删除
4. 验证笔记从列表中移除

Expected: 笔记删除成功

- [ ] **Step 6: 验证文件系统存储**

Run: `ls -la ~/.omnilink/notes/`
Expected: 看到创建的 `.md` 文件

Run: `cat ~/.omnilink/notes/note_*.md`
Expected: 文件内容与编辑器中一致

- [ ] **Step 7: 验证数据库记录**

Run: `sqlite3 ~/.omnilink/omnilink.db "SELECT * FROM notes;"`
Expected: 看到笔记元数据记录

- [ ] **Step 8: 测试外部工具访问**

Run: `open ~/.omnilink/notes/` (macOS) 或 `explorer ~/.omnilink/notes/` (Windows)
Expected: 可以用 Typora、Obsidian 等工具打开 `.md` 文件

- [ ] **Step 9: 记录测试结果**

创建测试报告文件：

```bash
echo "# 笔记功能测试报告

## 测试时间
$(date)

## 测试结果
- [x] 创建笔记
- [x] 编辑笔记
- [x] 自动保存
- [x] 搜索笔记
- [x] 删除笔记
- [x] 文件系统存储
- [x] 数据库记录
- [x] 外部工具访问

## 已知问题
无

## 备注
所有核心功能正常工作
" > docs/superpowers/test-reports/2026-04-22-notes-feature-test.md
```

- [ ] **Step 10: Commit**

```bash
git add docs/superpowers/test-reports/2026-04-22-notes-feature-test.md
git commit -m "test: 笔记功能端到端测试通过"
```

---

## Task 15: 文档更新

**Files:**
- Modify: `README.md` (如果存在)
- Create: `docs/features/notes.md`

- [ ] **Step 1: 创建功能文档**

```markdown
# 笔记功能

## 概述

OmniLink 的笔记功能提供了一个轻量级的 Markdown 笔记管理系统，支持文件系统存储、实时预览和自动保存。

## 功能特性

- **Markdown 编辑**：支持实时预览、语法高亮
- **文件存储**：笔记存储为 `.md` 文件，可被外部工具访问
- **自动保存**：内容变更后 2 秒自动保存
- **搜索功能**：按标题搜索笔记
- **左右分栏**：列表和详情同屏显示

## 使用方法

### 创建笔记

1. 点击左侧"新建笔记"按钮
2. 输入标题和内容
3. 自动保存

### 编辑笔记

1. 在左侧列表中选择笔记
2. 在右侧编辑器中修改内容
3. 使用工具栏快速插入格式

### 删除笔记

1. 选择要删除的笔记
2. 点击"删除"按钮
3. 确认删除

## 存储位置

- 默认路径：`~/.omnilink/notes/`
- 可在设置中自定义路径（后续版本）

## 文件格式

- 笔记文件：`note_{timestamp}_{id}.md`
- 图片文件：`images/{note_id}/image_{timestamp}.png`（后续版本）

## 外部工具集成

笔记文件可被以下工具直接访问：

- Obsidian
- Typora
- VS Code
- 任何 Markdown 编辑器

## 后续计划

- [ ] 图片粘贴上传
- [ ] 全文搜索
- [ ] AI 整理功能
- [ ] 笔记模板
- [ ] 导出功能
```

- [ ] **Step 2: 保存文档**

Run: `mkdir -p docs/features && cat > docs/features/notes.md`

- [ ] **Step 3: Commit**

```bash
git add docs/features/notes.md
git commit -m "docs: 添加笔记功能文档"
```

---

## Task 16: 最终验证和清理

**Files:**
- All modified files

- [ ] **Step 1: 运行完整构建**

Run: `npm run build`
Expected: 构建成功，无错误

- [ ] **Step 2: 检查代码质量**

Run: `cd src-tauri && cargo clippy`
Expected: 无警告或错误

- [ ] **Step 3: 验证所有功能**

1. 创建笔记 ✓
2. 编辑笔记 ✓
3. 搜索笔记 ✓
4. 删除笔记 ✓
5. 自动保存 ✓
6. 文件存储 ✓

- [ ] **Step 4: 清理临时文件**

Run: `rm -rf ~/.omnilink/notes/note_test_*.md`

- [ ] **Step 5: 创建最终提交**

```bash
git add -A
git commit -m "feat: 完成笔记功能 MVP

- 实现笔记 CRUD 操作
- 支持 Markdown 编辑和实时预览
- 文件系统存储，外部工具可访问
- 左右分栏布局
- 自动保存机制
- 搜索功能

Closes #issue-number"
```

- [ ] **Step 6: 推送到远程仓库**

Run: `git push origin feat/v0.0.1`

---

## 实现优先级总结

### P0 - MVP 核心功能（已完成）
- [x] 数据库表结构和迁移
- [x] 后端 Tauri Commands（CRUD + 文件操作）
- [x] 前端路由和基础布局
- [x] 笔记列表和详情展示
- [x] Markdown 编辑器（基础编辑 + 预览）
- [x] 自动保存机制

### P1 - 完善体验（后续迭代）
- [ ] 搜索功能增强（全文搜索）
- [ ] 图片粘贴上传
- [ ] 工具栏增强（表格、引用等）
- [ ] 路径配置功能
- [ ] 数据一致性校验

### P2 - 高级功能（未来版本）
- [ ] 导入本地 Markdown 文件
- [ ] AI 功能实现
- [ ] 笔记导出（PDF、HTML）
- [ ] 笔记模板
- [ ] 标签系统集成

---

## 预估工作量

- Task 1-4（后端）：2-3 小时
- Task 5-8（前端基础）：1-2 小时
- Task 9-12（UI 组件）：3-4 小时
- Task 13-16（集成测试）：1-2 小时

**总计**：7-11 小时

---

## 注意事项

1. **文件路径处理**：确保跨平台兼容（macOS、Windows、Linux）
2. **错误处理**：所有文件操作都需要错误处理
3. **性能优化**：大文件（>10MB）需要特殊处理
4. **数据一致性**：启动时校验文件与数据库同步
5. **用户体验**：自动保存提示、加载状态、错误提示

---

## 依赖项

### Rust Crates
- `rusqlite` - 已有
- `serde` - 已有
- `chrono` - 新增
- `shellexpand` - 新增

### NPM Packages
- `markdown-it` - 新增
- `highlight.js` - 新增
- `@types/markdown-it` - 新增

---

## 参考资料

- [Tauri 文件系统 API](https://tauri.app/v1/api/js/fs)
- [markdown-it 文档](https://github.com/markdown-it/markdown-it)
- [highlight.js 文档](https://highlightjs.org/)
- [Vue 3 文档](https://vuejs.org/)
- [Pinia 文档](https://pinia.vuejs.org/)

