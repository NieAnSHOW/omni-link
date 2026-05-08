# Workspace 功能设计

**日期：** 2026-05-08
**分支：** feat/v0.0.2
**状态：** 待实施

## 概述

将笔记管理从"数据库驱动"切换到"文件系统驱动"，用户选择任意文件夹作为工作区，直接编辑其中的 Markdown 文件。类似 Obsidian 的 Vault 模式。

## 设计决策

| 决策项 | 选择 |
|--------|------|
| 元数据 | 完全舍弃，文件名即标题，文件修改时间即更新时间 |
| 工作区初始化 | 欢迎页选择（类 Obsidian Vault 选择页） |
| 文件树 | 只读，仅浏览和点击打开 |
| "从链接创建" | 保留解析流水线，.md 直接写入工作区 |
| 工作区持久化 | 路径存入 ~/.omnilink/config.json |

## Rust 后端变更

### 新增 Tauri Commands

```
set_workspace(path: String) -> ()
  保存工作区路径到 config.json

get_workspace() -> Option<String>
  读取当前工作区路径

list_files(dir: String) -> Vec<FileEntry>
  递归扫描目录，返回文件树（仅 .md 文件）

read_file(path: String) -> String
  读取文本文件内容（path 为绝对路径，Rust 侧校验路径在工作区内）

write_file(path: String, content: String) -> ()
  写入文本文件内容（path 为绝对路径，Rust 侧校验路径在工作区内，自动创建父目录）
```

### FileEntry 数据结构

```rust
struct FileEntry {
    name: String,       // 文件/文件夹名
    path: String,       // 相对于工作区的完整路径
    is_dir: bool,       // 是否为目录
    children: Vec<FileEntry>,  // 子节点（仅目录有）
}
```

### 修改现有 Command

- `create_note_from_link`：去掉 SQLite 写入环节，生成的 .md 文件直接写入工作区根目录（以标题为文件名）

### 废弃不用的 Commands

以下 commands 不再需要，代码保留但不调用：
- `create_note` / `get_note` / `list_notes` / `update_note` / `delete_note`

### Config 扩展

`~/.omnilink/config.json` 新增字段：
```json
{
  "workspace_path": "/Users/xxx/Documents/my-notes"
}
```

## 前端变更

### 新增文件

- `src/views/WorkspaceView.vue` — 欢迎页，提供"打开文件夹"按钮，调用 Tauri dialog 选择文件夹
- `src/stores/workspace.ts` — 工作区状态管理（路径、文件树、当前文件内容）
- `src/composables/useWorkspaceApi.ts` — 封装 workspace 相关的 invoke 调用

### 修改文件

- `src/App.vue` — onMounted 检查工作区，有则进 AppLayout，无则显示 WorkspaceView
- `src/components/AppLayout.vue` — 侧边栏从扁平笔记列表改为递归文件树
- `src/components/notes/NoteDetail.vue` — 移除标题输入框和元数据行，只保留 VditorEditor
- `src/router/index.ts` — 新增 `/workspace` 路由，移除 `/notes/:id`（文件选择不再走路由参数，改为 store 状态驱动）
- `src/types/index.ts` — 新增 FileEntry 类型

### 可删除/不再使用

- `src/components/notes/NotesList.vue` — 不再使用（侧边栏内联到 AppLayout）
- `src/stores/notes.ts` — 被 workspace store 替代
- `src/composables/useApi.ts` 中的 notesApi — 被 workspaceApi 替代

### WorkspaceView 欢迎页

```
┌─────────────────────────────────┐
│                                 │
│        Omni-Link Logo           │
│                                 │
│   选择或创建一个工作区来开始     │
│                                 │
│   ┌───────────────────────┐     │
│   │    📂 打开文件夹       │     │
│   └───────────────────────┘     │
│                                 │
│   最近：                       │
│   /Users/xxx/Documents/notes   │
│                                 │
└─────────────────────────────────┘
```

### 文件树侧边栏

```
┌─────────────────┐
│ 📁 我的工作区 ▾  │
│─────────────────│
│ 📂 项目笔记/    │  ← 可展开/折叠
│   📄 规划.md    │  ← 点击打开编辑
│   📄 日报.md    │
│ 📂 技术参考/    │
│   📄 Rust.md   │
│   📄 Vue3.md   │
│ 📄 README.md   │
│ 📄 随手记.md   │
│─────────────────│
│ ⚙ 设置          │
└─────────────────┘
```

### NoteDetail 简化

- **移除：** 标题 `<input>`、元数据行（时间/来源链接）、保存状态 Badge
- **保留：** VditorEditor markdown 编辑器
- **保存逻辑：** 自动保存（内容变化 2 秒防抖 → 直接 `write_file`）

## 数据流

```
选择工作区
  → set_workspace(path)
  → list_files(path)
  → 渲染文件树

点击文件
  → read_file(path)
  → VditorEditor 展示内容

编辑内容
  → 防抖 2 秒
  → write_file(path, content)

从链接创建
  → 解析 URL → 生成 Markdown
  → write_file(workspace_path/title.md, content)
```

## 路径安全

- `read_file` / `write_file` 接收绝对路径，Rust 侧校验路径必须以 `workspace_path` 开头，拒绝路径遍历攻击
- `list_files` 仅返回 `.md` 文件，跳过隐藏文件（`.` 开头）

## 待清理

- SQLite `notes` 表不再使用，可后续清理 schema
- `note_commands.rs` 中废弃的 commands 可在后续版本移除

## 范围外

- 非 .md 文件在文件树中的展示
- 文件拖拽、重命名、删除等文件管理操作（在系统文件管理器中完成）
- Git 集成
- Frontmatter 解析
