# UI 优化实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完成 4 项 UI 优化——移除分类功能、重命名"链接"为"知识库"并改为瀑布流、禁用生产右键菜单、重命名"标签管理"为"智识图谱"

**Architecture:** 前端 Vue 3 + Rust 后端 Tauri v2，纯 CSS 瀑布流，按优先级逐项实施

**Tech Stack:** Vue 3, TypeScript, Rust, SQLite, Tauri v2

**设计文档：** `docs/superpowers/specs/2026-04-21-ui-optimization-design.md`

---

## 文件结构

### 删除的文件
- `src/components/CategoryTree.vue` — 分类树组件
- `src/components/CategoryNodeItem.vue` — 分类节点项组件
- `src/stores/categories.ts` — 分类 Pinia store
- `src-tauri/src/commands/category_commands.rs` — 分类 Tauri 命令
- `src-tauri/src/repositories/category_repo.rs` — 分类数据仓库

### 修改的文件
| 文件 | 改动 |
|------|------|
| `src/components/AppLayout.vue` | 移除 CategoryTree + 名称重命名 |
| `src/views/LinksView.vue` | 移除分类过滤 + 名称重命名 + 瀑布流 |
| `src/components/LinkCard.vue` | 瀑布流卡片样式 |
| `src/views/TagsView.vue` | 名称重命名 |
| `src/App.vue` | 添加右键菜单拦截 |
| `src/composables/useApi.ts` | 移除分类 API + category_id 参数 |
| `src/types/index.ts` | 移除分类类型 + category_id 字段 |
| `src-tauri/src/lib.rs` | 移除分类命令注册 |
| `src-tauri/src/commands/mod.rs` | 移除分类模块声明 |
| `src-tauri/src/repositories/mod.rs` | 移除分类模块声明 |
| `src-tauri/src/commands/link_commands.rs` | 移除 category_id 参数 |
| `src-tauri/src/repositories/link_repo.rs` | 移除 category_id 参数和相关函数 |
| `src-tauri/src/models.rs` | 移除分类相关结构体和字段 |
| `src-tauri/src/db/schema.rs` | 添加迁移删除分类表和列 |

---

## Task 1: 移除分类 — 前端清理

**Files:**
- Delete: `src/components/CategoryTree.vue`
- Delete: `src/components/CategoryNodeItem.vue`
- Delete: `src/stores/categories.ts`
- Modify: `src/components/AppLayout.vue`
- Modify: `src/views/LinksView.vue`
- Modify: `src/composables/useApi.ts`
- Modify: `src/types/index.ts`

- [ ] **Step 1: 删除分类相关文件**

```bash
rm src/components/CategoryTree.vue
rm src/components/CategoryNodeItem.vue
rm src/stores/categories.ts
```

- [ ] **Step 2: 修改 AppLayout.vue — 移除分类组件**

将整个文件替换为以下内容（移除 CategoryTree 引用、selectedCategoryId 逻辑、provide、category-section 区域）：

```vue
<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <img src="/logo-no-text.png" alt="OmniLink" class="logo" />
        <h1>OmniLink</h1>
      </div>
      <nav class="nav-section">
        <router-link to="/links" class="nav-item" :class="{ active: isLinksActive }" :aria-current="isLinksActive ? 'page' : undefined">
          <span class="icon">🔗</span> 链接
        </router-link>
        <router-link to="/tags" class="nav-item" active-class="active">
          <span class="icon">🏷️</span> 标签
        </router-link>
        <router-link to="/settings" class="nav-item" active-class="active">
          <span class="icon">⚙️</span> 设置
        </router-link>
      </nav>
    </aside>
    <main class="main-content">
      <router-view />
    </main>
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import Toast from './Toast.vue';

const route = useRoute();
const isLinksActive = computed(() => route.path.startsWith('/links'));
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 250px;
  background: #f8f9fa;
  border-right: 1px solid #e2e8f0;
  display: flex;
  flex-direction: column;
  padding: 16px 0;
  overflow-y: auto;
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px 16px;
  border-bottom: 1px solid #e2e8f0;
}

.logo {
  width: 28px;
  height: 28px;
}

.sidebar-header h1 {
  font-size: 18px;
  margin: 0;
}

.nav-section {
  padding: 8px;
  display: flex;
  flex-direction: column;
  row-gap: 5px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  color: #475569;
  text-decoration: none;
  font-size: 14px;
}

.nav-item:hover {
  background: #e2e8f0;
}

.nav-item.active {
  background: #6366f1;
  color: white;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}
</style>
```

- [ ] **Step 3: 修改 LinksView.vue — 移除分类过滤参数**

在 `loadLinks` 函数中，移除 `category_id` 相关逻辑：

将 `loadLinks` 函数改为：

```typescript
async function loadLinks() {
  store.loading = true;
  try {
    const params: { limit: number; offset: number; status?: string } = { limit: 20, offset: 0 };
    if (activeFilter.value) {
      params.status = activeFilter.value;
    }
    const res = await api.getLinks(params);
    store.links = res.links;
    store.total = res.total;
  } finally {
    store.loading = false;
  }
}
```

同时简化 watch，移除 `route.query.category`：

```typescript
watch(
  activeFilter,
  () => {
    loadLinks();
  },
  { immediate: true },
);
```

并移除不再使用的 `useRoute` 导入和 `route` 变量：

```typescript
// 删除这行
import { useRouter, useRoute } from 'vue-router';
// 改为
import { useRouter } from 'vue-router';

// 删除这行
const route = useRoute();
```

- [ ] **Step 4: 修改 useApi.ts — 移除分类 API 和 category_id 参数**

1. 移除顶部 import 中的 `CategoryNode`：

```typescript
import type { Link, LinkDetail, TagWithCount } from '../types/index';
```

2. `getLinks` 方法移除 `category_id` 参数：

```typescript
getLinks: (params?: { limit?: number; offset?: number; status?: string }) =>
  invoke<{ links: Link[]; total: number }>('get_links', {
    limit: params?.limit ?? null,
    offset: params?.offset ?? null,
    status: params?.status ?? null,
  }),
```

3. 删除整个 `// Categories` 注释块及其下 4 个方法（getCategories、upsertCategory、deleteCategory、updateLinkCategory）

- [ ] **Step 5: 修改 types/index.ts — 移除分类类型和 category_id 字段**

1. 从 `Link` 接口移除 `category_id` 字段
2. 删除 `Category` 接口
3. 删除 `CategoryNode` 接口

最终 `Link` 接口：

```typescript
export interface Link {
  id: number;
  url: string;
  title: string | null;
  platform: string | null;
  source: string;
  status: 'pending' | 'parsing' | 'parsed' | 'failed';
  created_at: string;
  updated_at: string;
}
```

- [ ] **Step 6: 验证前端编译**

Run: `npm run build`
Expected: 编译通过，无 TypeScript 错误

- [ ] **Step 7: 提交**

```bash
git add -A
git commit -m "refactor: remove categories feature from frontend"
```

---

## Task 2: 移除分类 — 后端清理

**Files:**
- Delete: `src-tauri/src/commands/category_commands.rs`
- Delete: `src-tauri/src/repositories/category_repo.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/repositories/mod.rs`
- Modify: `src-tauri/src/commands/link_commands.rs`
- Modify: `src-tauri/src/repositories/link_repo.rs`
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: 删除分类相关文件**

```bash
rm src-tauri/src/commands/category_commands.rs
rm src-tauri/src/repositories/category_repo.rs
```

- [ ] **Step 2: 修改 commands/mod.rs — 移除分类模块声明**

将文件改为：

```rust
pub mod link_commands;
pub mod content_commands;
pub mod settings_commands;
pub mod tag_commands;
```

- [ ] **Step 3: 修改 repositories/mod.rs — 移除分类模块声明**

将文件改为：

```rust
pub mod link_repo;
pub mod content_repo;
pub mod settings_repo;
pub mod ai_result_repo;
pub mod tag_repo;
```

- [ ] **Step 4: 修改 lib.rs — 移除分类命令注册**

将 `.invoke_handler(tauri::generate_handler![...])` 改为：

```rust
.invoke_handler(tauri::generate_handler![
    commands::link_commands::get_links,
    commands::link_commands::create_link,
    commands::link_commands::get_link,
    commands::link_commands::delete_link,
    commands::link_commands::parse_link_cmd,
    commands::content_commands::get_link_detail,
    commands::content_commands::analyze_content_cmd,
    commands::content_commands::update_content_cmd,
    commands::settings_commands::get_settings,
    commands::settings_commands::get_ai_config,
    commands::settings_commands::update_ai_config,
    commands::tag_commands::get_tags,
    commands::tag_commands::create_tag,
    commands::tag_commands::delete_tag,
    commands::tag_commands::update_content_tags,
])
```

- [ ] **Step 5: 修改 models.rs — 移除分类结构体和 category_id 字段**

1. 从 `Link` 结构体移除 `category_id` 字段
2. 删除 `Category` 结构体
3. 删除 `CategoryNode` 结构体
4. 删除 `UpsertCategoryInput` 结构体

`Link` 结构体改为：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub platform: Option<String>,
    pub source: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 6: 修改 link_commands.rs — 移除 category_id 参数**

将 `get_links` 函数签名和调用改为：

```rust
#[tauri::command]
pub async fn get_links(
    state: State<'_, DbState>,
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<String>,
) -> AppResult<LinksResponse> {
    let conn = state.0.lock().unwrap();
    let lim = limit.unwrap_or(20);
    let off = offset.unwrap_or(0);
    let links = link_repo::get_links(&conn, lim, off, status.as_deref())?;
    let total = link_repo::get_links_count(&conn, status.as_deref())?;
    Ok(LinksResponse { links, total })
}
```

- [ ] **Step 7: 修改 link_repo.rs — 移除 category_id 参数和相关函数**

1. `get_links` 函数简化（移除 category_id 参数，只处理 status 过滤）：

```rust
pub fn get_links(conn: &Connection, limit: i64, offset: i64, status: Option<&str>) -> AppResult<Vec<Link>> {
    let mut links = Vec::new();
    if let Some(s) = status {
        let mut stmt = conn.prepare(
            "SELECT * FROM links WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )?;
        let rows = stmt.query_map(params![s, limit, offset], |row| row_to_link(row))?;
        for row in rows {
            links.push(row?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT * FROM links ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )?;
        let rows = stmt.query_map(params![limit, offset], |row| row_to_link(row))?;
        for row in rows {
            links.push(row?);
        }
    }
    Ok(links)
}
```

2. `get_links_count` 函数简化：

```rust
pub fn get_links_count(conn: &Connection, status: Option<&str>) -> AppResult<i64> {
    let count: i64 = if let Some(s) = status {
        conn.query_row(
            "SELECT COUNT(*) FROM links WHERE status = ?",
            params![s],
            |row| row.get(0),
        )?
    } else {
        conn.query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))?
    };
    Ok(count)
}
```

3. `row_to_link` 函数移除 `category_id` 字段读取：

```rust
fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<Link> {
    Ok(Link {
        id: row.get("id")?,
        url: row.get("url")?,
        title: row.get("title")?,
        platform: row.get("platform")?,
        source: row.get("source")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}
```

4. 删除 `update_link_category` 函数
5. 删除 `get_links_by_category` 函数

- [ ] **Step 8: 验证 Rust 编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无错误

- [ ] **Step 9: 提交**

```bash
git add -A
git commit -m "refactor: remove categories feature from backend"
```

---

## Task 3: 移除分类 — 数据库迁移

**Files:**
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: 修改 schema.rs — 添加迁移删除分类表和列**

在 `init_schema` 函数的 `CREATE TABLE` 语句中：
1. 删除整个 `CREATE TABLE IF NOT EXISTS categories` 语句
2. 从 `links` 表定义中移除 `category_id INTEGER` 行
3. 删除 `CREATE INDEX IF NOT EXISTS idx_links_category` 行

同时修改 `migrate` 函数，移除旧的 `category_id` 迁移逻辑，添加新的迁移来清理已有数据库：

将 `migrate` 函数替换为：

```rust
fn migrate(conn: &Connection) -> AppResult<()> {
    // content_status migration
    let has_content_status: bool = conn
        .prepare("SELECT content_status FROM contents LIMIT 0")
        .is_ok();
    if !has_content_status {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN content_status TEXT DEFAULT 'success';")?;
    }

    // updated_at for contents
    let has_updated_at: bool = conn
        .prepare("SELECT updated_at FROM contents LIMIT 0")
        .is_ok();
    if !has_updated_at {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN updated_at TEXT;")?;
        conn.execute_batch("UPDATE contents SET updated_at = datetime('now') WHERE updated_at IS NULL;")?;
    }

    // Drop categories table and category_id column from links
    drop_categories(conn)?;

    Ok(())
}

fn drop_categories(conn: &Connection) -> AppResult<()> {
    // Drop categories table if exists
    conn.execute_batch("DROP TABLE IF EXISTS categories;")?;

    // Rebuild links table without category_id (SQLite < 3.35.0 safe)
    let has_category_id: bool = conn
        .prepare("SELECT category_id FROM links LIMIT 0")
        .is_ok();
    if has_category_id {
        conn.execute_batch(
            "CREATE TABLE links_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                platform TEXT,
                source TEXT DEFAULT 'manual',
                status TEXT DEFAULT 'pending' CHECK(status IN ('pending','parsing','parsed','failed')),
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );
            INSERT INTO links_new (id, url, title, platform, source, status, created_at, updated_at)
                SELECT id, url, title, platform, source, status, created_at, updated_at FROM links;
            DROP TABLE links;
            ALTER TABLE links_new RENAME TO links;
            CREATE INDEX IF NOT EXISTS idx_links_status ON links(status);
            CREATE INDEX IF NOT EXISTS idx_links_platform ON links(platform);
            CREATE INDEX IF NOT EXISTS idx_links_created ON links(created_at);",
        )?;
    }

    Ok(())
}
```

- [ ] **Step 2: 验证 Rust 编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add -A
git commit -m "refactor: add DB migration to drop categories table and column"
```

---

## Task 4: "链接" → "知识库" + 瀑布流布局

**Files:**
- Modify: `src/components/AppLayout.vue`
- Modify: `src/views/LinksView.vue`
- Modify: `src/components/LinkCard.vue`

- [ ] **Step 1: 修改 AppLayout.vue — 重命名"链接"为"知识库"**

将 sidebar 导航中链接项的文字和图标改为：

```html
<router-link to="/links" class="nav-item" :class="{ active: isLinksActive }" :aria-current="isLinksActive ? 'page' : undefined">
  <span class="icon">📚</span> 知识库
</router-link>
```

- [ ] **Step 2: 修改 LinksView.vue — 重命名文案**

在模板中修改以下文案：
- `<h2>我的链接</h2>` → `<h2>我的知识库</h2>`
- `+ 添加链接` → `+ 添加知识`
- `还没有链接，点击上方按钮添加` → `还没有知识，点击上方按钮添加`

在脚本中修改 toast 文案：
- `` 成功添加 ${urls.length} 个链接 `` → `` 成功添加 ${urls.length} 条知识 ``
- `链接已删除` → `知识已删除`
- `添加失败` 不变
- `删除失败` 不变

- [ ] **Step 3: 修改 LinksView.vue — 添加瀑布流 CSS**

将 `.link-list` 的样式从纵向列表改为瀑布流：

```css
.link-list {
  columns: 3;
  column-gap: 12px;
}

@media (max-width: 899px) {
  .link-list { columns: 2; }
}

@media (max-width: 599px) {
  .link-list { columns: 1; }
}
```

- [ ] **Step 4: 修改 LinkCard.vue — 瀑布流卡片样式**

将 `link-card` 添加瀑布流所需的样式，修改标题截断方式：

```css
.link-card {
  border: 1px solid #e2e8f0; border-radius: 10px;
  padding: 16px; cursor: pointer;
  transition: box-shadow 0.15s;
  break-inside: avoid;
  margin-bottom: 12px;
}
```

将 `.link-title` 从单行截断改为最多 3 行：

```css
.link-title {
  font-size: 15px; font-weight: 500; color: #1e293b;
  margin-bottom: 4px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
```

- [ ] **Step 5: 验证前端编译**

Run: `npm run build`
Expected: 编译通过

- [ ] **Step 6: 手动验证瀑布流布局**

Run: `npm run tauri dev`
Expected: 知识库页面显示 Pinterest 式 3 列瀑布流，卡片高度随内容自适应，标题最多显示 3 行

- [ ] **Step 7: 提交**

```bash
git add -A
git commit -m "feat: rename links to knowledge base and add waterfall layout"
```

---

## Task 5: 禁用生产右键菜单

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 修改 App.vue — 添加生产环境右键菜单拦截**

将整个文件替换为：

```vue
<template>
  <AppLayout />
</template>

<script setup lang="ts">
import AppLayout from './components/AppLayout.vue';

if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}
</script>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
</style>
```

注意：`import.meta.env.PROD` 在 `tauri build` 时为 `true`，`tauri dev` 时为 `false`，开发环境保留右键菜单。

- [ ] **Step 2: 验证**

Run: `npm run build`
Expected: 编译通过

Run: `npm run tauri dev`
Expected: 开发模式下右键菜单正常可用（因为 `import.meta.env.PROD` 为 `false`）

- [ ] **Step 3: 提交**

```bash
git add -A
git commit -m "feat: disable context menu in production builds"
```

---

## Task 6: "标签管理" → "智识图谱" 改名

**Files:**
- Modify: `src/components/AppLayout.vue`
- Modify: `src/views/TagsView.vue`

- [ ] **Step 1: 修改 AppLayout.vue — 重命名 sidebar 导航**

将标签导航项改为：

```html
<router-link to="/tags" class="nav-item" active-class="active">
  <span class="icon">🧠</span> 智识图谱
</router-link>
```

- [ ] **Step 2: 修改 TagsView.vue — 重命名页面标题**

将 `<h2>标签管理</h2>` 改为 `<h2>智识图谱</h2>`

- [ ] **Step 3: 验证**

Run: `npm run build`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add -A
git commit -m "feat: rename tag management to knowledge graph"
```
