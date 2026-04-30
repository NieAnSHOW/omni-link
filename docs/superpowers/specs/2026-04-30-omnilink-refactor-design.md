# OmniLink 整体重构设计

> 日期：2026-04-30
> 状态：待审阅
> 方案：渐进式重构（方案 A）

## 1. 产品定位

将 OmniLink 从「跨平台链接采集 + 知识管理」重新定位为**极简 Markdown 笔记应用**，参考 Typora 风格。

核心保留功能：
- **笔记**：Markdown 编辑/渲染，文件存储
- **链接解析**：作为笔记创建的一种来源方式
- **人格系统**：完整保留，通过编辑器工具栏触发

移除功能：
- 知识库卡片瀑布流视图
- 智识图谱（标签管理页面）
- 独立的链接列表和内容详情页

## 2. 导航与路由

路由表从 5 条精简为 3 条：

| 路径 | 组件 | 说明 |
|------|------|------|
| `/` | redirect → `/notes` | 默认页 |
| `/notes` | NotesView | 笔记主页（集成侧边栏 + 编辑器） |
| `/notes/:id` | NotesView | 选中笔记的 URL 状态 |
| `/settings` | SettingsView | 设置页 |

删除的路由：`/links`、`/tags`、`/links/:id`。

## 3. 侧边栏设计

宽度 260px，支持收起/展开（快捷键 `Cmd+B`）。

结构从上到下：

```
┌─────────────────────┐
│  搜索笔记...         │  搜索框，实时过滤
├─────────────────────┤
│  + 新建笔记 ▾        │  按钮，展开：空白笔记 / 从链接创建
├─────────────────────┤
│  标题A    2分钟前     │
│  标题B    1小时前     │  笔记列表，按更新时间倒序
│  标题C    昨天        │  每项：标题 + 相对时间
│  ...                 │  选中项高亮
├─────────────────────┤
│         ⚙️           │  底部设置图标
└─────────────────────┘
```

交互：
- 点击笔记 → 主区域切换到该笔记编辑器
- 搜索框 → 实时按标题过滤
- 新建「空白笔记」→ 直接创建并打开
- 新建「从链接创建」→ 弹出 URL 输入对话框

## 4. 笔记编辑器主区域

收起侧边栏后，编辑器占满全屏宽度。

```
┌──────────────────────────────────────────────┐
│  [≡]  笔记标题                    [👤] [☀️]  │
│       ─────────────────────────────────────  │
│                                              │
│       # 正文内容以 Markdown 渲染...           │
│                                              │
│       支持所见即所得编辑（复用现有              │
│       MarkdownEditor 组件）                   │
│                                              │
└──────────────────────────────────────────────┘
```

顶栏元素：
- `[≡]` — 侧边栏展开/收起按钮
- 笔记标题 — 可直接点击编辑
- `[👤]` — 人格工具栏按钮（弹出 PersonaDialog，选择人格处理当前笔记）
- `[☀️/🌙]` — 主题切换按钮

编辑器复用现有 `MarkdownEditor.vue` 组件。

人格处理流程：点击 `[👤]` → 弹出 PersonaDialog → 选择人格 → 处理当前笔记内容 → 结果覆盖或追加。

## 5. 从链接创建笔记

用户流程：

1. 点击侧边栏「新建笔记 ▾」→ 选择「从链接创建」
2. 弹出对话框，输入/粘贴 URL
3. 点击「解析创建」→ 后端调用解析管道（identifier → fetcher → extractor）
4. 解析完成 → 自动创建新笔记，内容为解析出的 Markdown
5. 打开新笔记进入编辑器

对话框：

```
┌─── 从链接创建笔记 ───────────────┐
│                                  │
│  URL: [https://example.com...  ] │
│                                  │
│  [取消]              [解析创建]   │
└──────────────────────────────────┘
```

状态反馈：解析中显示 loading，解析失败显示错误信息可重试。

## 6. 主题系统

- CSS 变量方式（`--background`、`--foreground`、`--primary` 等）
- `<html data-theme="light|dark">` 属性切换
- 持久化到 `user_settings` 表
- 支持跟随系统偏好 + 手动切换

## 7. 后端变更

### 7.1 删除

| 文件 | 操作 |
|------|------|
| `commands/tag_commands.rs` | 删除 |
| `repositories/tag_repo.rs` | 删除 |
| `db/schema.rs` tags/content_tags 表 | 删除建表语句 |
| `lib.rs` tag 命令注册 | 移除 |
| `models.rs` Tag/TagWithCount/CreateTagInput | 移除 |
| `commands/link_commands.rs` | 删除 |
| `repositories/link_repo.rs` | 删除 |
| `commands/content_commands.rs` | 删除 |
| `repositories/content_repo.rs` | 删除（保留解析能力） |

### 7.2 保留并复用

| 模块 | 说明 |
|------|------|
| `parser/` | identifier → fetcher → extractor 管道，用于链接解析 |
| `ai/` | AI 服务和 providers |
| `persona/` | 人格扫描器和内置人格 |
| `terminal/` | PTY 管理和会话 |
| `commands/note_commands.rs` | 笔记 CRUD |
| `repositories/note_repo.rs` | 笔记仓库 |
| `commands/settings_commands.rs` | 设置命令 |
| `commands/persona_commands.rs` | 人格命令 |
| `commands/terminal_commands.rs` | 终端命令 |

### 7.3 新增

| 内容 | 说明 |
|------|------|
| Note 模型 `source_url` 字段 | `Option<String>`，记录笔记来源 URL |
| `create_note_from_link` Command | 解析链接 → 创建笔记的 Tauri 命令 |
| 数据库迁移 | 添加 `source_url` 列到 notes 表，删除标签相关表 |

### 7.4 解析管道适配

现有 `parser/pipeline.rs` 编排了解析流程并将结果写入 `contents` 表。重构后需要：

- 修改 pipeline 的输出目标：不再写入 contents 表，而是返回解析结果（标题 + Markdown 正文）
- `create_note_from_link` Command 调用 pipeline 获取结果 → 创建 notes 记录 → 写入 .md 文件
- `parser/identifier.rs`、`fetcher.rs`、`extractor.rs` 保持不变
- `parser/llm_extractor.rs`、`content_judge.rs`、`webview_extractor.rs` 保留（解析能力不受影响）

### 7.5 数据迁移

现有数据库中可能有 links/contents 数据。迁移策略：

1. **可选迁移**：在 schema 迁移中，将已解析的 links + contents 转换为 notes 记录
   - 从 links 表读取 URL 和 title
   - 从 contents 表读取 body_text
   - 创建对应的 notes 记录（source_url 设为原 URL）
   - 将 body_text 写入 .md 文件
2. **清理旧表**：迁移完成后 DROP 旧表
3. **降级方案**：如果数据量小或用户不在意，可直接删除旧表不做迁移

迁移代码放在 `db/schema.rs` 的迁移逻辑中，版本号递增。

### 7.6 清理

- 删除不再被前端调用的旧 API 封装（useApi.ts 中的 tag/link/content 相关方法）
- 删除不再使用的类型定义（types/index.ts 中的 Tag、Link 等）

## 8. 前端变更

### 8.1 删除的文件

| 文件 | 原因 |
|------|------|
| `views/LinksView.vue` | 知识库页面移除 |
| `views/TagsView.vue` | 智识图谱移除 |
| `views/ContentView.vue` | 链接内容详情移除 |
| `components/LinkCard.vue` | 知识库卡片移除 |
| `components/LinkCardSkeleton.vue` | 骨架屏移除 |
| `components/AddLinkDialog.vue` | 替换为 CreateNoteDialog |
| `components/StickyHeader.vue` | 知识库专用 |
| `components/TagInput.vue` | 标签输入移除 |
| `stores/links.ts` | 链接 store 移除 |
| `stores/tags.ts` | 标签 store 移除 |

### 8.2 重写的文件

| 文件 | 说明 |
|------|------|
| `components/AppLayout.vue` | 新侧边栏 + 主题切换 + 收起/展开 |
| `router/index.ts` | 新路由表 |

### 8.3 新增的文件

| 文件 | 说明 |
|------|------|
| `components/CreateNoteDialog.vue` | 新建笔记对话框（空白/从链接） |
| `composables/useTheme.ts` | 主题切换 composable |

### 8.4 保留的文件

| 文件 | 说明 |
|------|------|
| `views/NotesView.vue` | 主笔记视图，需小幅调整 |
| `views/SettingsView.vue` | 设置页保留 |
| `components/notes/*` | NotesList、NoteDetail、MarkdownEditor |
| `components/persona/*` | PersonaDialog、TerminalPanel（TerminalPanel 保留代码，但不再默认嵌入笔记详情页，仅在人格需要终端时按需加载） |
| `stores/notes.ts` | 笔记 store |
| `composables/useApi.ts` | API 封装，需更新 |
| `types/index.ts` | 类型定义，需更新 |
| `types/persona.ts` | 人格类型保留 |

## 9. 数据模型变更

### Notes 表扩展

```sql
ALTER TABLE notes ADD COLUMN source_url TEXT;
```

新增字段：
- `source_url: Option<String>` — 笔记来源 URL，手动创建的笔记为 NULL

### 删除的表

- `tags` — 标签表
- `content_tags` — 内容-标签关联表
- `links` — 链接表
- `contents` — 链接内容表
- `ai_results` — AI 结果表（关联 contents，随 contents 一起删除）
- `import_history` — 导入历史表

### 保留的表

- `notes` — 笔记表（扩展）
- `user_settings` — 用户设置表
- `personas` — 人格表
- `terminal_sessions` — 终端会话表
