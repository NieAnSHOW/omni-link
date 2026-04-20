# OmniLink P0 — 标签与分类管理设计

**日期**: 2026-04-20
**状态**: 已确认
**优先级**: P0（MVP 核心缺失功能）

## Context

MVP v0.1 已完成链接采集、内容解析、AI 摘要和基础 UI，但标签和分类功能尚未激活。数据库中 `tags`、`content_tags`、`categories` 表已定义但未使用，AI 分析返回的标签未入库。本设计补齐这部分能力。

## 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 标签 vs 分类 | 分离：标签扁平关键词，分类树状层级 | 职责清晰，标签灵活，分类结构化 |
| 标签管理 UI | 侧边栏独立标签页 + 内容详情页内嵌编辑 | 全局管理和局部编辑兼顾 |
| 分类树 UI | 侧边栏内嵌，点击筛选链接 | 类似文件管理器，操作直觉 |
| 分类归属 | AI 自动推断 + 手动调整 | 减少手动工作，保留灵活控制 |
| 标签创建 | AI 返回标签全自动入库，同名去重 | 零干预，不中断分析流程 |

## 后端数据层

### Schema 变更

**tags 表**（已有，激活使用）：
- `id` INTEGER PK
- `name` TEXT UNIQUE — 标签名
- `color` TEXT — 可选颜色值（如 `#50fa7b`），AI 创建时自动分配
- `type` TEXT — `auto`（AI 创建）或 `manual`（手动创建）
- `created_at` TEXT

**content_tags 表**（已有，激活使用）：
- `content_id` INTEGER → `contents.id`
- `tag_id` INTEGER → `tags.id`
- 联合主键去重

**categories 表**（已有，激活使用）：
- `id` INTEGER PK
- `name` TEXT — 分类名
- `parent_id` INTEGER → `categories.id`（NULL = 根节点）
- `sort_order` INTEGER — 同级排序
- `created_at` TEXT

**links 表**（新增字段）：
- `category_id` INTEGER → `categories.id`（可为 NULL）

### 新增 Tauri Commands（8 个）

**标签相关：**
1. `get_tags` — 返回所有标签及关联内容数量
2. `create_tag(name, color)` — 手动创建标签
3. `delete_tag(tag_id)` — 删除标签（级联清理 content_tags）
4. `update_content_tags(content_id, tag_ids: Vec<i64>)` — 更新内容的标签关联（全量替换）

**分类相关：**
5. `get_categories` — 返回完整分类树
6. `upsert_category(id, name, parent_id, sort_order)` — 创建或更新分类节点
7. `delete_category(category_id)` — 删除分类（子节点的 category_id 置 NULL）
8. `update_link_category(link_id, category_id)` — 手动调整链接的分类

### AI 分析流程改造

现有 `analyze_content_cmd` 扩展：

**标签自动入库：**
```
AI Provider 返回 { summary, tags: ["Rust", "异步", "Tokio"] }
  ↓
遍历 tags：
  INSERT OR IGNORE INTO tags (name, type='auto', color=预设色板轮取)
  获取 tag_id
  INSERT OR IGNORE INTO content_tags (content_id, tag_id)
```

**分类自动推断：**
```
AI Provider 返回增加 classification 字段
  { summary, tags, category: "技术/后端" }
  ↓
解析路径 "技术/后端" → 逐级查找/创建 categories 记录
  ↓
更新 links.category_id = 末级分类 ID
```

**AI Prompt 扩展：**
返回格式从 `{ summary, tags }` 扩展为：
```json
{
  "summary": "...",
  "tags": ["tag1", "tag2"],
  "category": "技术/后端"
}
```

**Fallback Provider 扩展：**
用关键词匹配简单分类规则（如含 "Rust/Go/Python" → "技术/后端"），无匹配时 category 返回 null。

## 前端 UI

### 侧边栏改造

现有侧边栏（Links / Settings）改造为：
- **导航区**：链接、标签、设置三个入口
- **分类树区**：折叠式分类树，点击分类筛选链接列表，右上角「+」新建分类

### 标签管理页（/tags 路由）

- 标签列表：显示颜色圆点、名称、关联内容数量、来源类型（auto/manual）
- 操作：编辑（改颜色/名称）、删除
- 顶部：搜索框 + 新建标签按钮

### 内容详情页内嵌标签编辑

- 标签区：显示已关联标签（pill 样式），每个带「×」移除按钮
- 添加标签：输入框，输入时模糊匹配已有标签，回车确认
- 分类显示：标题下方显示当前分类路径，点击可切换

### 路由变更

新增 `/tags` 路由 → TagsView 组件

## 文件变更清单

### Rust 后端（新增/修改）
- `src-tauri/src/db/schema.rs` — 激活 tags/content_tags/categories 表，links 加 category_id
- `src-tauri/src/repositories/tag_repo.rs` — 新增，标签 CRUD + 内容关联
- `src-tauri/src/repositories/category_repo.rs` — 新增，分类树 CRUD
- `src-tauri/src/repositories/mod.rs` — 注册新 repo
- `src-tauri/src/commands/tag_commands.rs` — 新增，4 个标签 commands
- `src-tauri/src/commands/category_commands.rs` — 新增，4 个分类 commands
- `src-tauri/src/commands/mod.rs` — 注册新 commands
- `src-tauri/src/lib.rs` — 注册新 commands 到 Tauri app
- `src-tauri/src/ai/ai_service.rs` — 扩展 AI 分析流程（标签入库 + 分类推断）
- `src-tauri/src/ai/openai_provider.rs` — prompt 扩展 category 字段
- `src-tauri/src/ai/ollama_provider.rs` — prompt 扩展 category 字段
- `src-tauri/src/ai/fallback_provider.rs` — 扩展关键词分类规则
- `src-tauri/src/models.rs` — 扩展 AiAnalysisResult 结构体

### Vue 前端（新增/修改）
- `src/views/TagsView.vue` — 新增，标签管理页
- `src/views/ContentView.vue` — 修改，内嵌标签编辑 + 分类显示
- `src/views/LinksView.vue` — 修改，支持分类筛选
- `src/components/AppLayout.vue` — 修改，侧边栏加分类树和标签导航
- `src/components/CategoryTree.vue` — 新增，分类树组件
- `src/components/TagInput.vue` — 新增，标签输入/选择组件
- `src/composables/useApi.ts` — 新增标签和分类 API 调用
- `src/stores/tags.ts` — 新增，标签状态管理
- `src/stores/categories.ts` — 新增，分类状态管理
- `src/types/index.ts` — 新增 Tag、Category 类型定义
- `src/router/index.ts` — 新增 /tags 路由

## 实现策略

自底向上，分层实现：

1. **数据层**：schema 激活、新增 repo、新增 commands
2. **AI 层**：扩展 prompt 和分析流程
3. **前端层**：新增组件、修改布局、串联交互
4. **集成测试**：端到端验证标签自动创建、分类推断、UI 操作
