# 笔记功能设计文档

**日期**：2026-04-22  
**项目**：OmniLink  
**版本**：v0.1 MVP

## 概述

为 OmniLink 添加独立的笔记功能模块，采用轻量级文件系统存储方案。笔记以 Markdown 文件形式存储在用户可配置的目录中，SQLite 仅存储元数据索引。后续链接解析功能将作为笔记内容生成的一种方式整合进来。

## 核心需求

1. **完全独立**：笔记功能与现有链接系统独立，但共享 UI 框架和基础设施
2. **文件存储**：笔记内容存储为 `.md` 文件，可被外部工具（Obsidian、Typora）访问
3. **左右分栏布局**：左侧笔记列表，右侧笔记详情编辑
4. **多种创建方式**：空白新建、导入文件、后续支持链接解析生成
5. **扁平组织**：按时间排序，通过搜索筛选，暂不支持文件夹
6. **路径可配置**：默认 `~/.omnilink/notes/`，用户可自定义
7. **完整编辑能力**：富文本工具栏、代码高亮、图片粘贴、表格编辑
8. **AI 功能预留**：预留操作入口，暂不实现

## 技术方案：轻量级文件系统方案

### 优势
- 符合设计理念（"sqlite 只做元数据的存储"）
- 笔记文件可被外部工具直接访问
- 迁移简单，用户可直接复制 `.md` 文件
- 性能好，大文本不占用数据库空间

### 劣势与应对
- **文件系统权限问题**：启动时检查路径可写性，失败时提示用户
- **文件与数据库不同步**：启动时校验，优先以文件内容为准

## 数据层设计

### SQLite 表结构

```sql
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    file_name TEXT NOT NULL UNIQUE,  -- 实际文件名（如 "note_1713801234_1.md"）
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    file_size INTEGER DEFAULT 0,      -- 文件大小（字节）
    word_count INTEGER DEFAULT 0      -- 字数统计
);

CREATE INDEX idx_notes_updated_at ON notes(updated_at DESC);
CREATE INDEX idx_notes_created_at ON notes(created_at DESC);
```

### user_settings 表扩展

新增字段：
```sql
ALTER TABLE user_settings ADD COLUMN notes_storage_path TEXT DEFAULT '~/.omnilink/notes/';
```

### 文件存储规则

- **默认路径**：`~/.omnilink/notes/`
- **文件命名**：`note_{timestamp}_{id}.md`（如 `note_1713801234_1.md`）
- **图片存储**：`~/.omnilink/notes/images/{note_id}/`
- **Markdown 引用**：相对路径 `![](images/{note_id}/image_xxx.png)`

### 前端类型定义

```typescript
// src/types/index.ts
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
  content: string;  // Markdown 文本内容
}
```

## 后端实现

### 文件结构

```
src-tauri/src/
├── commands/
│   └── note_commands.rs          # 新增：Tauri 命令层
├── repositories/
│   └── note_repo.rs               # 新增：数据库操作层
├── models.rs                      # 扩展：添加 Note 和 NoteDetail
└── lib.rs                         # 注册 note_commands
```

### 核心 Tauri Commands

```rust
// 笔记 CRUD
#[tauri::command]
async fn create_note(title: String, state: State<'_, AppState>) -> Result<Note, String>

#[tauri::command]
async fn get_note(id: i64, state: State<'_, AppState>) -> Result<NoteDetail, String>

#[tauri::command]
async fn list_notes(offset: i64, limit: i64, state: State<'_, AppState>) -> Result<Vec<Note>, String>

#[tauri::command]
async fn update_note(id: i64, title: String, content: String, state: State<'_, AppState>) -> Result<(), String>

#[tauri::command]
async fn delete_note(id: i64, state: State<'_, AppState>) -> Result<(), String>

// 文件操作
#[tauri::command]
async fn read_note_content(id: i64, state: State<'_, AppState>) -> Result<String, String>

#[tauri::command]
async fn save_note_content(id: i64, content: String, state: State<'_, AppState>) -> Result<(), String>

// 设置管理
#[tauri::command]
async fn get_notes_storage_path(state: State<'_, AppState>) -> Result<String, String>

#[tauri::command]
async fn set_notes_storage_path(path: String, state: State<'_, AppState>) -> Result<(), String>

// 数据一致性
#[tauri::command]
async fn validate_notes_integrity(state: State<'_, AppState>) -> Result<Vec<String>, String>
```

### 文件操作逻辑

**创建笔记**：
1. 在数据库插入记录（生成 ID）
2. 生成文件名：`note_{timestamp}_{id}.md`
3. 在文件系统创建空 `.md` 文件
4. 返回 `Note` 对象

**读取笔记**：
1. 从数据库获取元数据
2. 从文件系统读取 Markdown 内容
3. 返回 `NoteDetail` 对象

**更新笔记**：
1. 更新数据库元数据（`title`、`updated_at`、`word_count`、`file_size`）
2. 写入文件内容到 `.md` 文件
3. 事务保证原子性

**删除笔记**：
1. 删除数据库记录
2. 删除文件系统文件
3. 删除关联的图片目录 `images/{note_id}/`

**路径变更**：
1. 检查新路径是否可写
2. 移动所有 `.md` 文件到新路径
3. 移动 `images/` 目录到新路径
4. 更新数据库配置
5. 失败时回滚，保持原路径

### 错误处理

| 错误场景 | 处理策略 |
|---------|---------|
| 文件不存在 | 返回错误，提示用户文件丢失，允许删除记录 |
| 路径无权限 | 提示用户选择其他路径或授予权限 |
| 文件与数据库不同步 | 记录日志，优先以文件内容为准 |
| 磁盘空间不足 | 保存失败，提示用户清理空间 |
| 文件名冲突 | 自动重命名（添加时间戳后缀） |

## 前端实现

### 路由新增

```typescript
// src/router/index.ts
{
  path: '/notes',
  name: 'notes',
  component: () => import('../views/NotesView.vue')
}
```

### 组件结构

```
src/views/NotesView.vue              # 主容器（左右分栏布局）
src/components/notes/
├── NotesList.vue                    # 左侧：笔记列表
│   ├── 搜索框
│   ├── 新建笔记按钮
│   └── 笔记列表项（标题、更新时间、字数）
├── NoteDetail.vue                   # 右侧：笔记详情
│   ├── 标题编辑
│   ├── Markdown 编辑器
│   ├── 工具栏（保存、删除、AI 入口占位）
│   └── 元数据显示
└── MarkdownEditor.vue               # 通用 Markdown 编辑器组件
    ├── 工具栏（加粗、斜体、链接、图片等）
    ├── 编辑区（CodeMirror 或类似）
    ├── 预览区（实时渲染）
    └── 图片粘贴处理
```

### 状态管理

```typescript
// src/stores/notes.ts
import { defineStore } from 'pinia'

export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([])
  const currentNote = ref<NoteDetail | null>(null)
  const isEditing = ref(false)
  const searchQuery = ref('')

  async function fetchNotes(offset = 0, limit = 50) {
    // 调用 list_notes
  }

  async function createNote(title: string) {
    // 调用 create_note
  }

  async function loadNote(id: number) {
    // 调用 get_note
  }

  async function saveNote(id: number, title: string, content: string) {
    // 调用 update_note
  }

  async function deleteNote(id: number) {
    // 调用 delete_note
  }

  return {
    notes,
    currentNote,
    isEditing,
    searchQuery,
    fetchNotes,
    createNote,
    loadNote,
    saveNote,
    deleteNote
  }
})
```

### UI 布局（NotesView.vue）

```vue
<template>
  <div class="notes-view">
    <!-- 左侧列表 -->
    <div class="notes-sidebar">
      <NotesList 
        :notes="notesStore.notes"
        :selected-id="notesStore.currentNote?.note.id"
        @select="handleSelectNote"
        @create="handleCreateNote"
      />
    </div>

    <!-- 右侧详情 -->
    <div class="notes-content">
      <NoteDetail 
        v-if="notesStore.currentNote"
        :note-detail="notesStore.currentNote"
        @save="handleSaveNote"
        @delete="handleDeleteNote"
      />
      <div v-else class="empty-state">
        选择一个笔记或创建新笔记
      </div>
    </div>
  </div>
</template>

<style scoped>
.notes-view {
  display: flex;
  height: 100vh;
}

.notes-sidebar {
  width: 300px;
  border-right: 1px solid #e5e7eb;
  overflow-y: auto;
}

.notes-content {
  flex: 1;
  overflow-y: auto;
}
</style>
```

### 交互流程

1. **新建笔记**：
   - 用户点击"新建笔记"按钮
   - 调用 `create_note("未命名笔记")`
   - 自动选中新笔记，右侧打开编辑器
   - 标题可编辑，焦点自动定位到标题输入框

2. **选择笔记**：
   - 用户点击左侧列表项
   - 调用 `get_note(id)` 加载详情
   - 右侧显示笔记内容
   - 编辑器默认为预览模式，点击"编辑"按钮进入编辑模式

3. **编辑笔记**：
   - 用户修改标题或内容
   - 内容变更后 2 秒防抖自动保存
   - 或手动点击"保存"按钮
   - 保存成功后更新左侧列表（标题、更新时间、字数）

4. **删除笔记**：
   - 用户点击"删除"按钮
   - 弹出确认对话框
   - 确认后调用 `delete_note(id)`
   - 删除成功后清空右侧详情，左侧列表移除该项

5. **搜索笔记**：
   - 用户在左侧搜索框输入关键词
   - 实时过滤列表（按标题模糊匹配）
   - 暂不支持全文搜索

### Markdown 编辑器复用

从 `ContentView.vue` 中提取编辑器逻辑为独立组件 `MarkdownEditor.vue`：

**功能清单**：
- 工具栏：加粗、斜体、标题、列表、链接、图片、代码块、表格
- 实时预览：左右分栏或上下分栏
- 代码高亮：支持多种语言语法高亮
- 图片粘贴：
  - 监听 `paste` 事件
  - 提取图片数据
  - 保存到 `images/{note_id}/` 目录
  - 插入 Markdown 引用
- 表格编辑：工具栏插入表格模板
- 目录生成：自动解析标题生成目录（可选）

**技术选型**：
- 编辑器核心：CodeMirror 6 或 Monaco Editor
- Markdown 渲染：markdown-it + highlight.js
- 图片上传：Tauri 文件系统 API

## 功能细节与边界情况

### 1. 图片处理

**粘贴图片**：
- 监听编辑器 `paste` 事件
- 提取 `clipboardData` 中的图片
- 生成文件名：`image_{timestamp}.png`
- 调用 Tauri 命令保存到 `images/{note_id}/`
- 插入 Markdown：`![](images/{note_id}/image_xxx.png)`

**删除笔记时**：
- 同时删除 `images/{note_id}/` 目录及其所有文件

**路径变更时**：
- 移动 `images/` 目录到新路径

### 2. 自动保存机制

**触发条件**：
- 内容变更后 2 秒防抖
- 用户手动点击"保存"按钮
- 切换到其他笔记前自动保存当前笔记

**保存流程**：
1. 计算字数（去除 Markdown 语法后的纯文本字数）
2. 计算文件大小（字节）
3. 调用 `update_note(id, title, content)`
4. 更新 `updated_at`、`word_count`、`file_size`
5. 显示保存状态（保存中 → 已保存 / 保存失败）

**失败处理**：
- 显示错误提示
- 不丢失编辑内容（保留在内存中）
- 允许用户重试或手动保存

### 3. 搜索功能

**实现方式**：
- 左侧搜索框输入关键词
- 前端过滤 `notes` 数组（按 `title` 模糊匹配）
- 或调用后端 SQL：`SELECT * FROM notes WHERE title LIKE '%keyword%'`

**暂不支持**：
- 全文搜索（搜索笔记内容）
- 后续可升级为方案 C（添加全文索引）

### 4. 路径变更处理

**设置页面新增配置项**：
- "笔记存储路径"输入框 + "浏览"按钮
- 显示当前路径和占用空间

**变更流程**：
1. 用户选择新路径
2. 检查新路径是否可写（调用 Tauri 文件系统 API）
3. 弹出确认对话框："将移动 X 个笔记文件到新路径"
4. 后台执行移动操作：
   - 移动所有 `.md` 文件
   - 移动 `images/` 目录
   - 更新数据库配置
5. 显示进度条
6. 完成后提示用户

**失败处理**：
- 移动失败时回滚（恢复原路径）
- 记录错误日志
- 提示用户检查权限或磁盘空间

### 5. 数据一致性校验

**启动时校验**：
1. 读取数据库中的所有笔记记录
2. 扫描笔记目录中的所有 `.md` 文件
3. 对比差异：
   - **文件存在但数据库无记录**：提示用户是否导入
   - **数据库有记录但文件不存在**：标记为"文件丢失"，允许删除记录
4. 显示校验报告（如有问题）

**手动校验**：
- 设置页面新增"检查笔记完整性"按钮
- 执行上述校验流程

### 6. AI 功能预留

**UI 预留**：
- 右侧工具栏添加"AI 整理"按钮（禁用状态 + Tooltip："功能开发中"）
- 预留下拉菜单位置（整理内容、扩展内容、人格系统）

**后端预留**：
```rust
#[tauri::command]
async fn ai_process_note(
    id: i64, 
    action: String,  // "organize" | "expand" | "persona"
    state: State<'_, AppState>
) -> Result<String, String> {
    // 暂时返回 "功能开发中"
    Err("AI 功能尚未实现".to_string())
}
```

## 实现优先级

### P0（MVP 核心功能）
1. 数据库表结构和迁移
2. 后端 Tauri Commands（CRUD + 文件操作）
3. 前端路由和基础布局
4. 笔记列表和详情展示
5. Markdown 编辑器（基础编辑 + 预览）
6. 自动保存机制

### P1（完善体验）
1. 搜索功能
2. 图片粘贴上传
3. 工具栏（加粗、斜体等）
4. 路径配置功能
5. 数据一致性校验

### P2（后续迭代）
1. 导入本地 Markdown 文件
2. 全文搜索
3. AI 功能实现
4. 笔记导出（PDF、HTML）
5. 笔记模板

## 测试要点

### 功能测试
- [ ] 创建、读取、更新、删除笔记
- [ ] 自动保存和手动保存
- [ ] 搜索笔记（按标题）
- [ ] 图片粘贴和显示
- [ ] 路径变更和文件迁移
- [ ] 数据一致性校验

### 边界测试
- [ ] 文件不存在时的错误处理
- [ ] 路径无权限时的提示
- [ ] 磁盘空间不足时的处理
- [ ] 大文件（>10MB）的性能
- [ ] 特殊字符文件名的处理
- [ ] 并发编辑同一笔记（多窗口）

### 兼容性测试
- [ ] macOS、Windows、Linux 路径处理
- [ ] 外部工具（Obsidian）访问笔记文件
- [ ] 中文文件名和内容

## 风险与应对

| 风险 | 影响 | 应对措施 |
|-----|------|---------|
| 文件系统权限问题 | 无法读写笔记 | 启动时检查权限，提示用户授权 |
| 文件与数据库不同步 | 数据丢失或重复 | 启动时校验，优先以文件为准 |
| 大文件性能问题 | 编辑卡顿 | 限制单个笔记大小（如 10MB），超出时提示 |
| 图片占用空间过大 | 磁盘空间不足 | 设置页面显示占用空间，提供清理功能 |
| 路径变更失败 | 文件丢失 | 事务回滚，保持原路径 |

## 后续扩展方向

1. **全文搜索**：添加 SQLite FTS5 全文索引或集成 Tantivy
2. **标签系统**：复用现有标签功能，支持笔记打标签
3. **链接解析集成**：将链接解析结果直接生成为笔记
4. **AI 功能**：整理、扩展、人格系统集成
5. **笔记模板**：预设模板（日记、读书笔记、技术文档）
6. **导出功能**：导出为 PDF、HTML、Markdown 压缩包
7. **版本历史**：Git 集成或自定义版本管理
8. **协作功能**：多设备同步（WebDAV、iCloud）

## 参考资料

- Tauri 文件系统 API：https://tauri.app/v1/api/js/fs
- markdown-it：https://github.com/markdown-it/markdown-it
- CodeMirror 6：https://codemirror.net/
- SQLite FTS5：https://www.sqlite.org/fts5.html
