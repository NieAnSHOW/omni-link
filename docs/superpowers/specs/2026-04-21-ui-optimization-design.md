# UI 优化设计方案

**日期：** 2026-04-21
**分支：** feat/optimization001
**优先级：** P0 × 3 + P1 × 1

## 概述

4 项 UI 优化，按优先级逐项实施，每项独立提交验证。

---

## 优化 1（P0）：移除分类功能

前后端全部移除分类相关代码和数据结构。

### 前端清理

| 操作 | 文件 |
|------|------|
| 删除 | `src/components/CategoryTree.vue` |
| 删除 | `src/components/CategoryNodeItem.vue` |
| 删除 | `src/stores/categories.ts` |
| 修改 | `src/components/AppLayout.vue` — 移除 CategoryTree 引用、category-section 区域、selectedCategoryId 逻辑和 provide |
| 修改 | `src/views/LinksView.vue` — 移除 `route.query.category` 和 `category_id` 过滤参数 |
| 修改 | `src/composables/useApi.ts` — 移除分类相关 API 方法（getCategories、createCategory、deleteCategory、updateLinkCategory） |
| 修改 | `src/types/index.ts` — 移除 CategoryNode 类型 |

### 后端清理

| 操作 | 文件 |
|------|------|
| 删除 | `src-tauri/src/commands/category_commands.rs` |
| 删除 | `src-tauri/src/repositories/category_repo.rs` |
| 修改 | `src-tauri/src/lib.rs` — 从 generate_handler! 移除 4 个分类命令 |
| 修改 | `src-tauri/src/commands/mod.rs` — 移除 category_commands 模块声明 |
| 修改 | `src-tauri/src/repositories/mod.rs` — 移除 category_repo 模块声明 |

### 数据库迁移

在 `src-tauri/src/db/schema.rs` 的 `migrate()` 函数中添加：

- 删除 `categories` 表（`DROP TABLE IF EXISTS categories`）
- 从 `links` 表移除 `category_id` 列
  - SQLite < 3.35.0 不支持 `DROP COLUMN`，采用重建表策略：
    1. 创建 `links_new`（不含 `category_id`）
    2. 复制数据
    3. 删除旧表
    4. 重命名新表
  - 同时删除 `idx_links_category` 索引

---

## 优化 2（P0）："链接" → "知识库" + 瀑布流布局

### 名称修改

| 位置 | 原文 | 新文案 |
|------|------|--------|
| AppLayout sidebar 导航 | `🔗 链接` | `📚 知识库` |
| LinksView 页面标题 | `我的链接` | `我的知识库` |
| LinksView 添加按钮 | `+ 添加链接` | `+ 添加知识` |
| LinksView 空状态 | `还没有链接，点击上方按钮添加` | `还没有知识，点击上方按钮添加` |
| Toast 成功提示 | `成功添加 X 个链接` | `成功添加 X 条知识` |
| Toast 删除提示 | `链接已删除` | `知识已删除` |

路由 `/links` 保持不变。

### 瀑布流布局

**实现方式：** CSS `columns` 属性

- `LinksView.vue` 的 `.link-list` 改为：
  ```css
  .link-list {
    columns: 3;
    column-gap: 12px;
  }
  ```
- `LinkCard.vue` 卡片加 `break-inside: avoid`，去掉固定高度约束
- 标题允许换行显示（移除 `white-space: nowrap`），让卡片高度随内容自适应
- 响应式断点：
  - `>= 900px`：3 列
  - `600px ~ 899px`：2 列
  - `< 600px`：1 列

**LinkCard 调整：**
- 移除标题的单行截断（`overflow: hidden; text-overflow: ellipsis; white-space: nowrap`）
- 标题最多显示 3 行（`-webkit-line-clamp: 3`）
- URL 仍单行截断
- 卡片之间通过 `column-gap` 间距，卡片内加 `margin-bottom: 12px`（columns 布局的垂直间距）

---

## 优化 3（P0）：生产构建禁用右键菜单

在 `src/App.vue` 的 `onMounted` 中：

```typescript
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}
```

- `import.meta.env.PROD` 在 `tauri build` 时为 `true`，`tauri dev` 时为 `false`
- 开发环境保留右键菜单，便于调试

---

## 优化 4（P1）："标签管理" → "智识图谱"

仅改 UI 文案，不改逻辑和路由。

| 位置 | 原文 | 新文案 |
|------|------|--------|
| AppLayout sidebar 导航 | `🏷️ 标签` | `🧠 智识图谱` |
| TagsView 页面标题 | `标签管理` | `智识图谱` |

路由 `/tags` 保持不变。标签内部的创建、删除、搜索等功能不变。

---

## 实施顺序

1. 优化 1 — 移除分类功能（前后端 + 数据库）
2. 优化 2 — "知识库" 重命名 + 瀑布流布局
3. 优化 3 — 禁用生产右键菜单
4. 优化 4 — "智识图谱" 改名

每步独立提交，独立验证。
