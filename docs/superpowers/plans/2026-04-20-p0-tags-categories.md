# OmniLink P0 — 标签与分类管理 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 OmniLink 补齐标签自动创建、标签管理、分类树和 AI 分类推断功能

**Architecture:** 自底向上实现——先后端数据层（Rust repo + commands），再 AI 流程改造，最后前端 UI（Vue 组件 + 路由 + 状态管理）。SQLite 已有 tags/content_tags/categories 表定义，需要激活使用并新增 links.category_id 字段。

**Tech Stack:** Rust (rusqlite, serde) + Vue 3 (Composition API, Pinia) + TypeScript + Tauri v2 IPC

---

## 文件变更清单

### Rust 后端
| 操作 | 文件 | 职责 |
|------|------|------|
| 修改 | `src-tauri/src/db/schema.rs` | links 表加 category_id 列 |
| 修改 | `src-tauri/src/models.rs` | 新增 Tag, Category, TagWithCount, CategoryNode 类型 |
| 新增 | `src-tauri/src/repositories/tag_repo.rs` | 标签 CRUD + 内容关联 |
| 新增 | `src-tauri/src/repositories/category_repo.rs` | 分类树 CRUD |
| 修改 | `src-tauri/src/repositories/mod.rs` | 注册新 repo 模块 |
| 修改 | `src-tauri/src/repositories/link_repo.rs` | 查询支持 category_id |
| 新增 | `src-tauri/src/commands/tag_commands.rs` | 4 个标签 Tauri commands |
| 新增 | `src-tauri/src/commands/category_commands.rs` | 4 个分类 Tauri commands |
| 修改 | `src-tauri/src/commands/mod.rs` | 注册新 command 模块 |
| 修改 | `src-tauri/src/commands/content_commands.rs` | analyze_content_cmd 扩展标签入库 + 分类推断 |
| 修改 | `src-tauri/src/lib.rs` | 注册新 commands |
| 修改 | `src-tauri/src/ai/openai_provider.rs` | prompt 增加 classification 字段 |
| 修改 | `src-tauri/src/ai/ollama_provider.rs` | prompt 增加 classification 字段 |
| 修改 | `src-tauri/src/ai/fallback_provider.rs` | 关键词分类规则 |

### Vue 前端
| 操作 | 文件 | 职责 |
|------|------|------|
| 修改 | `src/types/index.ts` | 新增 Tag, Category, TagWithCount 类型 |
| 修改 | `src/composables/useApi.ts` | 新增标签和分类 API 调用 |
| 新增 | `src/stores/tags.ts` | 标签状态管理 |
| 新增 | `src/stores/categories.ts` | 分类状态管理 |
| 新增 | `src/components/CategoryTree.vue` | 分类树组件 |
| 新增 | `src/components/TagInput.vue` | 标签输入/选择组件 |
| 新增 | `src/views/TagsView.vue` | 标签管理页 |
| 修改 | `src/components/AppLayout.vue` | 侧边栏加分类树和标签导航 |
| 修改 | `src/views/ContentView.vue` | 内嵌标签编辑 + 分类显示 |
| 修改 | `src/views/LinksView.vue` | 分类筛选支持 |
| 修改 | `src/router/index.ts` | 新增 /tags 路由 |

---

## Task 1: Schema 迁移 — links 表加 category_id

**Files:**
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: 在 links 建表语句中加 category_id 列**

在 `src-tauri/src/db/schema.rs` 的 links 表定义中，`updated_at` 行后加：

```sql
category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
```

同时在建表语句末尾（`CREATE INDEX` 区域）加一个分类索引：

```sql
CREATE INDEX IF NOT EXISTS idx_links_category ON links(category_id);
```

最终 links 建表应为：

```sql
CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    platform TEXT,
    source TEXT DEFAULT 'manual',
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending','parsing','parsed','failed')),
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);
```

注意：因为 `categories` 表在 links 之后定义，需要将 `categories` 建表语句移到 `links` 之前，或者去掉外键约束的 `REFERENCES`（SQLite 外键在 CREATE TABLE 时不验证引用顺序）。推荐方案：去掉 `REFERENCES categories(id) ON DELETE SET NULL`，只在代码层维护关联，简化建表顺序依赖。

最终 links 表加列为：

```sql
category_id INTEGER,
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无报错

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/db/schema.rs
git commit -m "feat(schema): add category_id to links table"
```

---

## Task 2: 新增数据模型

**Files:**
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: 在 models.rs 末尾添加 Tag、Category 相关结构体**

在 `src-tauri/src/models.rs` 文件末尾追加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub tag_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub tag_type: String,
    pub content_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryNode {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub children: Vec<CategoryNode>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertCategoryInput {
    pub id: Option<i64>,
    pub name: String,
    pub parent_id: Option<i64>,
}
```

- [ ] **Step 2: 在 Link 结构体中加 category_id**

在 `src-tauri/src/models.rs` 的 `Link` 结构体中，`updated_at` 字段后加：

```rust
pub category_id: Option<i64>,
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 可能有编译错误（link_repo 的 `row_to_link` 需要同步更新），先忽略，下个 task 一起修

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/models.rs
git commit -m "feat(models): add Tag, Category models and category_id to Link"
```

---

## Task 3: link_repo 适配 category_id

**Files:**
- Modify: `src-tauri/src/repositories/link_repo.rs`

- [ ] **Step 1: 更新 row_to_link 函数**

在 `src-tauri/src/repositories/link_repo.rs` 的 `row_to_link` 函数中加：

```rust
category_id: row.get("category_id")?,
```

同时新增一个函数，用于按分类筛选链接：

```rust
pub fn update_link_category(conn: &Connection, id: i64, category_id: Option<i64>) -> AppResult<()> {
    conn.execute(
        "UPDATE links SET category_id = ?, updated_at = datetime('now') WHERE id = ?",
        params![category_id, id],
    )?;
    Ok(())
}

pub fn get_links_by_category(conn: &Connection, category_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Link>> {
    let mut links = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT * FROM links WHERE category_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )?;
    let rows = stmt.query_map(params![category_id, limit, offset], |row| row_to_link(row))?;
    for row in rows {
        links.push(row?);
    }
    Ok(links)
}
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/repositories/link_repo.rs
git commit -m "feat(link_repo): support category_id and category filter"
```

---

## Task 4: tag_repo.rs — 标签数据访问

**Files:**
- Create: `src-tauri/src/repositories/tag_repo.rs`

- [ ] **Step 1: 创建 tag_repo.rs**

创建文件 `src-tauri/src/repositories/tag_repo.rs`：

```rust
use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::{Tag, TagWithCount};

pub fn create_tag(conn: &Connection, name: &str, color: &str, tag_type: &str) -> AppResult<Tag> {
    conn.execute(
        "INSERT INTO tags (name, color, type) VALUES (?, ?, ?)",
        params![name, color, tag_type],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: name.to_string(),
        color: color.to_string(),
        tag_type: tag_type.to_string(),
        created_at: String::new(),
    })
}

pub fn get_all_tags(conn: &Connection) -> AppResult<Vec<Tag>> {
    let mut tags = Vec::new();
    let mut stmt = conn.prepare("SELECT * FROM tags ORDER BY created_at")?;
    let rows = stmt.query_map([], |row| Ok(Tag {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        tag_type: row.get("type")?,
        created_at: row.get("created_at")?,
    }))?;
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn get_tags_with_count(conn: &Connection) -> AppResult<Vec<TagWithCount>> {
    let mut tags = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT t.*, COUNT(ct.content_id) as content_count
         FROM tags t
         LEFT JOIN content_tags ct ON t.id = ct.tag_id
         GROUP BY t.id
         ORDER BY t.created_at"
    )?;
    let rows = stmt.query_map([], |row| Ok(TagWithCount {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        tag_type: row.get("type")?,
        content_count: row.get("content_count")?,
        created_at: row.get("created_at")?,
    }))?;
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn get_tag_by_name(conn: &Connection, name: &str) -> AppResult<Option<Tag>> {
    let mut stmt = conn.prepare("SELECT * FROM tags WHERE name = ?")?;
    let tag = stmt.query_row(params![name], |row| Ok(Tag {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        tag_type: row.get("type")?,
        created_at: row.get("created_at")?,
    })).ok();
    Ok(tag)
}

pub fn delete_tag(conn: &Connection, id: i64) -> AppResult<bool> {
    conn.execute("DELETE FROM tags WHERE id = ?", params![id])?;
    Ok(conn.changes() > 0)
}

pub fn ensure_tag(conn: &Connection, name: &str, color: &str) -> AppResult<i64> {
    if let Some(tag) = get_tag_by_name(conn, name)? {
        return Ok(tag.id);
    }
    create_tag(conn, name, color, "auto")?;
    Ok(conn.last_insert_rowid())
}

pub fn set_content_tags(conn: &Connection, content_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    conn.execute("DELETE FROM content_tags WHERE content_id = ?", params![content_id])?;
    for &tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO content_tags (content_id, tag_id) VALUES (?, ?)",
            params![content_id, tag_id],
        )?;
    }
    Ok(())
}
```

- [ ] **Step 2: 注册 tag_repo 模块**

在 `src-tauri/src/repositories/mod.rs` 中加：

```rust
pub mod tag_repo;
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/repositories/tag_repo.rs src-tauri/src/repositories/mod.rs
git commit -m "feat: add tag_repo for tag CRUD and content association"
```

---

## Task 5: category_repo.rs — 分类树数据访问

**Files:**
- Create: `src-tauri/src/repositories/category_repo.rs`

- [ ] **Step 1: 创建 category_repo.rs**

创建文件 `src-tauri/src/repositories/category_repo.rs`：

```rust
use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::{Category, CategoryNode};

pub fn create_category(conn: &Connection, name: &str, parent_id: Option<i64>) -> AppResult<Category> {
    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?, ?)",
        params![name, parent_id],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Category {
        id,
        name: name.to_string(),
        parent_id,
        created_at: String::new(),
    })
}

pub fn update_category(conn: &Connection, id: i64, name: &str, parent_id: Option<i64>) -> AppResult<()> {
    conn.execute(
        "UPDATE categories SET name = ?, parent_id = ? WHERE id = ?",
        params![name, parent_id, id],
    )?;
    Ok(())
}

pub fn get_all_categories(conn: &Connection) -> AppResult<Vec<Category>> {
    let mut categories = Vec::new();
    let mut stmt = conn.prepare("SELECT * FROM categories ORDER BY id")?;
    let rows = stmt.query_map([], |row| Ok(Category {
        id: row.get("id")?,
        name: row.get("name")?,
        parent_id: row.get("parent_id")?,
        created_at: row.get("created_at")?,
    }))?;
    for row in rows {
        categories.push(row?);
    }
    Ok(categories)
}

pub fn get_category_tree(conn: &Connection) -> AppResult<Vec<CategoryNode>> {
    let all = get_all_categories(conn)?;
    Ok(build_tree(&all, None))
}

pub fn delete_category(conn: &Connection, id: i64) -> AppResult<bool> {
    conn.execute("DELETE FROM categories WHERE id = ?", params![id])?;
    Ok(conn.changes() > 0)
}

pub fn find_or_create_by_path(conn: &Connection, path: &str) -> AppResult<Option<i64>> {
    let segments: Vec<&str> = path.split('/').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Ok(None);
    }

    let mut parent_id: Option<i64> = None;
    for segment in &segments {
        let existing: Option<Category> = if let Some(pid) = parent_id {
            let mut stmt = conn.prepare("SELECT * FROM categories WHERE name = ? AND parent_id = ?")?;
            stmt.query_row(params![segment, pid], |row| Ok(Category {
                id: row.get("id")?,
                name: row.get("name")?,
                parent_id: row.get("parent_id")?,
                created_at: row.get("created_at")?,
            })).ok()
        } else {
            let mut stmt = conn.prepare("SELECT * FROM categories WHERE name = ? AND parent_id IS NULL")?;
            stmt.query_row(params![segment], |row| Ok(Category {
                id: row.get("id")?,
                name: row.get("name")?,
                parent_id: row.get("parent_id")?,
                created_at: row.get("created_at")?,
            })).ok()
        };

        match existing {
            Some(cat) => parent_id = Some(cat.id),
            None => {
                let cat = create_category(conn, segment, parent_id)?;
                parent_id = Some(cat.id);
            }
        }
    }

    Ok(parent_id)
}

fn build_tree(categories: &[Category], parent_id: Option<i64>) -> Vec<CategoryNode> {
    categories
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .map(|c| CategoryNode {
            id: c.id,
            name: c.name.clone(),
            parent_id: c.parent_id,
            children: build_tree(categories, Some(c.id)),
        })
        .collect()
}
```

- [ ] **Step 2: 注册 category_repo 模块**

在 `src-tauri/src/repositories/mod.rs` 中加：

```rust
pub mod category_repo;
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/repositories/category_repo.rs src-tauri/src/repositories/mod.rs
git commit -m "feat: add category_repo for category tree CRUD"
```

---

## Task 6: tag_commands.rs — 标签 Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/tag_commands.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 创建 tag_commands.rs**

创建文件 `src-tauri/src/commands/tag_commands.rs`：

```rust
use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::{CreateTagInput, TagWithCount};
use crate::repositories::tag_repo;

const TAG_COLORS: &[&str] = &[
    "#50fa7b", "#8be9fd", "#ff79c6", "#f1fa8c",
    "#ffb86c", "#bd93f9", "#ff5555", "#6272a4",
];

fn auto_color(index: usize) -> String {
    TAG_COLORS[index % TAG_COLORS.len()].to_string()
}

#[tauri::command]
pub async fn get_tags(state: State<'_, DbState>) -> AppResult<Vec<TagWithCount>> {
    let conn = state.0.lock().unwrap();
    tag_repo::get_tags_with_count(&conn)
}

#[tauri::command]
pub async fn create_tag(state: State<'_, DbState>, input: CreateTagInput) -> AppResult<TagWithCount> {
    let conn = state.0.lock().unwrap();
    let color = input.color.unwrap_or_else(|| auto_color(
        tag_repo::get_all_tags(&conn).map(|t| t.len()).unwrap_or(0)
    ));
    let tag = tag_repo::create_tag(&conn, &input.name, &color, "manual")?;
    Ok(TagWithCount {
        id: tag.id,
        name: tag.name,
        color: tag.color,
        tag_type: tag.tag_type,
        content_count: 0,
        created_at: tag.created_at,
    })
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, DbState>, id: i64) -> AppResult<bool> {
    let conn = state.0.lock().unwrap();
    tag_repo::delete_tag(&conn, id)
}

#[tauri::command]
pub async fn update_content_tags(
    state: State<'_, DbState>,
    content_id: i64,
    tag_ids: Vec<i64>,
) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    tag_repo::set_content_tags(&conn, content_id, &tag_ids)
}
```

- [ ] **Step 2: 注册 tag_commands 模块**

在 `src-tauri/src/commands/mod.rs` 中加：

```rust
pub mod tag_commands;
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/tag_commands.rs src-tauri/src/commands/mod.rs
git commit -m "feat: add tag Tauri commands (get, create, delete, update_content_tags)"
```

---

## Task 7: category_commands.rs — 分类 Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/category_commands.rs`

- [ ] **Step 1: 创建 category_commands.rs**

创建文件 `src-tauri/src/commands/category_commands.rs`：

```rust
use tauri::State;

use crate::db::DbState;
use crate::error::AppResult;
use crate::models::{CategoryNode, UpsertCategoryInput};
use crate::repositories::{category_repo, link_repo};

#[tauri::command]
pub async fn get_categories(state: State<'_, DbState>) -> AppResult<Vec<CategoryNode>> {
    let conn = state.0.lock().unwrap();
    category_repo::get_category_tree(&conn)
}

#[tauri::command]
pub async fn upsert_category(
    state: State<'_, DbState>,
    input: UpsertCategoryInput,
) -> AppResult<CategoryNode> {
    let conn = state.0.lock().unwrap();
    if let Some(id) = input.id {
        category_repo::update_category(&conn, id, &input.name, input.parent_id)?;
        let tree = category_repo::get_category_tree(&conn)?;
        find_node(&tree, id).cloned()
            .ok_or_else(|| crate::error::AppError::NotFound("Category not found".into()))
    } else {
        let cat = category_repo::create_category(&conn, &input.name, input.parent_id)?;
        Ok(CategoryNode {
            id: cat.id,
            name: cat.name,
            parent_id: cat.parent_id,
            children: vec![],
        })
    }
}

#[tauri::command]
pub async fn delete_category(state: State<'_, DbState>, id: i64) -> AppResult<bool> {
    let conn = state.0.lock().unwrap();
    category_repo::delete_category(&conn, id)
}

#[tauri::command]
pub async fn update_link_category(
    state: State<'_, DbState>,
    link_id: i64,
    category_id: Option<i64>,
) -> AppResult<()> {
    let conn = state.0.lock().unwrap();
    link_repo::update_link_category(&conn, link_id, category_id)
}

fn find_node<'a>(nodes: &'a [CategoryNode], id: i64) -> Option<&'a CategoryNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let found @ Some(_) = find_node(&node.children, id) {
            return found;
        }
    }
    None
}
```

- [ ] **Step 2: 注册 category_commands 模块**

在 `src-tauri/src/commands/mod.rs` 中加：

```rust
pub mod category_commands;
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/category_commands.rs src-tauri/src/commands/mod.rs
git commit -m "feat: add category Tauri commands (get, upsert, delete, update_link_category)"
```

---

## Task 8: 注册所有新 Commands 到 Tauri App

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 invoke_handler 中注册新 commands**

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 宏中，`settings_commands::update_ai_config,` 之后加：

```rust
commands::tag_commands::get_tags,
commands::tag_commands::create_tag,
commands::tag_commands::delete_tag,
commands::tag_commands::update_content_tags,
commands::category_commands::get_categories,
commands::category_commands::upsert_category,
commands::category_commands::delete_category,
commands::category_commands::update_link_category,
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register tag and category commands in Tauri app"
```

---

## Task 9: AI 分析流程改造 — 标签自动入库 + 分类推断

**Files:**
- Modify: `src-tauri/src/ai/openai_provider.rs`
- Modify: `src-tauri/src/ai/ollama_provider.rs`
- Modify: `src-tauri/src/ai/fallback_provider.rs`
- Modify: `src-tauri/src/commands/content_commands.rs`

- [ ] **Step 1: 更新 OpenAI provider prompt**

在 `src-tauri/src/ai/openai_provider.rs` 的 `generate_summary` 方法中，替换 prompt 格式化字符串为：

```rust
let prompt = format!(
    "请对以下内容进行分析，返回JSON格式：\n\
     {{\"summary\": \"200字以内摘要\", \"tags\": [\"标签1\",\"标签2\",\"标签3\"], \"category\": \"一级分类/二级分类\"}}\n\
     分类规则：如果有现成分类树就选最匹配的路径，否则根据内容推断一个分类路径（最多两级）。不确定时category设为null。\n\n\
     标题：{}\n内容：{}",
    title.unwrap_or("未知"),
    truncated,
);
```

- [ ] **Step 2: 更新 Ollama provider prompt**

在 `src-tauri/src/ai/ollama_provider.rs` 的 `generate_summary` 方法中，替换 prompt 格式化字符串为：

```rust
let prompt = format!(
    "请对以下内容进行分析，严格返回JSON格式：\n\
     {{\"summary\": \"200字以内摘要\", \"tags\": [\"标签1\",\"标签2\",\"标签3\"], \"category\": \"一级分类/二级分类\"}}\n\
     分类规则：根据内容推断一个分类路径（最多两级）。不确定时category设为null。\n\n\
     标题：{}\n内容：{}",
    title.unwrap_or("未知"),
    truncated,
);
```

- [ ] **Step 3: 更新 Fallback provider — 增加分类推断**

在 `src-tauri/src/ai/fallback_provider.rs` 中：

在 `STOP_WORDS` 常量之后加分类关键词规则：

```rust
const CATEGORY_RULES: &[(&str, &str)] = &[
    ("rust|golang|python|java|typescript|javascript|编程|开发|api|框架", "技术/后端"),
    ("vue|react|angular|css|html|前端|组件|界面", "技术/前端"),
    ("ai|gpt|llm|模型|机器学习|深度学习|神经网络|transformer", "技术/AI"),
    ("设计|ui|ux|交互|体验|原型|figma", "设计"),
    ("产品|需求|用户|功能|迭代|mvp", "产品"),
    ("阅读|书籍|读书|文章|书评", "阅读"),
];
```

在 `generate_summary` 方法中，构建返回值之前加分类推断：

```rust
let category = classify_by_keywords(clean_text);
```

返回值改为：

```rust
Ok(serde_json::json!({
    "summary": summary,
    "tags": tags,
    "category": category,
}))
```

在文件末尾加：

```rust
fn classify_by_keywords(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for (pattern, category) in CATEGORY_RULES {
        let re = regex::Regex::new(pattern).ok()?;
        if re.is_match(&lower) {
            return Some(category.to_string());
        }
    }
    None
}
```

- [ ] **Step 4: 改造 analyze_content_cmd — 标签自动入库 + 分类推断**

在 `src-tauri/src/commands/content_commands.rs` 中：

顶部 imports 加：

```rust
use crate::models::TagWithCount;
use crate::repositories::{tag_repo, category_repo};
```

在 `analyze_content_cmd` 函数中，`let conn = state.0.lock().unwrap();` 这行（第二次获取锁的位置）之后、`ai_result_repo::create_ai_result` 之前，插入标签入库逻辑。

将整个 `analyze_content_cmd` 函数替换为：

```rust
#[tauri::command]
pub async fn analyze_content_cmd(
    state: State<'_, DbState>,
    config_state: State<'_, ConfigState>,
    content_id: i64,
) -> AppResult<serde_json::Value> {
    let (body_text, title, link_id) = {
        let conn = state.0.lock().unwrap();
        let content = content_repo::get_content_by_id(&conn, content_id)?
            .ok_or_else(|| AppError::NotFound("Content not found".into()))?;
        (content.body_text.unwrap_or_default(), content.title, content.link_id)
    };

    let config = config_state.0.lock().unwrap().clone();
    let result = ai_service::analyze_content(&config.ai, &body_text, title.as_deref()).await?;

    let summary = result["summary"].as_str().unwrap_or("").to_string();
    let tags: Vec<String> = result["tags"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let classification = result["category"].as_str().map(String::from);

    let conn = state.0.lock().unwrap();

    // AI 结果入库
    ai_result_repo::create_ai_result(
        &conn,
        content_id,
        Some(&summary),
        &tags,
        classification.as_deref(),
        Some(&config.ai.provider),
    )?;

    // 标签自动入库 + 关联
    let tag_colors = [
        "#50fa7b", "#8be9fd", "#ff79c6", "#f1fa8c",
        "#ffb86c", "#bd93f9", "#ff5555", "#6272a4",
    ];
    let mut tag_ids = Vec::new();
    for (i, tag_name) in tags.iter().enumerate() {
        let color = tag_colors[i % tag_colors.len()];
        let tag_id = tag_repo::ensure_tag(&conn, tag_name, color)?;
        tag_ids.push(tag_id);
    }
    tag_repo::set_content_tags(&conn, content_id, &tag_ids)?;

    // 分类自动推断
    if let Some(ref cat_path) = classification {
        if let Some(cat_id) = category_repo::find_or_create_by_path(&conn, cat_path)? {
            let _ = link_repo::update_link_category(&conn, link_id, Some(cat_id));
        }
    }

    Ok(result)
}
```

同时在文件顶部移除未使用的 `use crate::models::Link;`（如果有的话），确保 `link_id` 变量在闭包中被正确解构。

- [ ] **Step 5: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/ai/openai_provider.rs src-tauri/src/ai/ollama_provider.rs src-tauri/src/ai/fallback_provider.rs src-tauri/src/commands/content_commands.rs
git commit -m "feat: AI analysis auto-creates tags and infers category"
```

---

## Task 10: 前端类型和 API 扩展

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/composables/useApi.ts`

- [ ] **Step 1: 在 types/index.ts 中添加新类型**

在 `src/types/index.ts` 文件末尾追加：

```typescript
export interface Tag {
  id: number;
  name: string;
  color: string;
  tag_type: 'auto' | 'manual';
  created_at: string;
}

export interface TagWithCount {
  id: number;
  name: string;
  color: string;
  tag_type: 'auto' | 'manual';
  content_count: number;
  created_at: string;
}

export interface Category {
  id: number;
  name: string;
  parent_id: number | null;
  created_at: string;
}

export interface CategoryNode {
  id: number;
  name: string;
  parent_id: number | null;
  children: CategoryNode[];
}
```

同时在 `Link` 接口中加 `category_id`：

```typescript
category_id: number | null;
```

在 `LinkDetail` 接口中加 `tags`：

```typescript
export interface LinkDetail {
  link: Link;
  content?: Content;
  ai?: AiResult;
  tags?: TagWithCount[];
}
```

- [ ] **Step 2: 在 useApi.ts 中添加标签和分类 API**

在 `src/composables/useApi.ts` 中，return 对象末尾（`updateAiConfig` 之后）加：

```typescript
    // Tags
    getTags: () =>
      invoke<TagWithCount[]>('get_tags'),

    createTag: (name: string, color?: string) =>
      invoke<TagWithCount>('create_tag', { input: { name, color: color ?? null } }),

    deleteTag: (id: number) =>
      invoke<boolean>('delete_tag', { id }),

    updateContentTags: (contentId: number, tagIds: number[]) =>
      invoke<void>('update_content_tags', { contentId, tagIds }),

    // Categories
    getCategories: () =>
      invoke<CategoryNode[]>('get_categories'),

    upsertCategory: (params: { id?: number; name: string; parentId?: number | null }) =>
      invoke<CategoryNode>('upsert_category', { input: { id: params.id ?? null, name: params.name, parent_id: params.parentId ?? null } }),

    deleteCategory: (id: number) =>
      invoke<boolean>('delete_category', { id }),

    updateLinkCategory: (linkId: number, categoryId: number | null) =>
      invoke<void>('update_link_category', { linkId, categoryId }),
```

在顶部 import 中加入新类型：

```typescript
import type { Link, LinkDetail, TagWithCount, CategoryNode } from '../types/index';
```

- [ ] **Step 3: 验证 TypeScript 编译**

Run: `cd /Users/niean/Documents/project/omni-link && npx vue-tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 4: 提交**

```bash
git add src/types/index.ts src/composables/useApi.ts
git commit -m "feat: add Tag/Category types and API calls to frontend"
```

---

## Task 11: Pinia Stores — tags 和 categories

**Files:**
- Create: `src/stores/tags.ts`
- Create: `src/stores/categories.ts`

- [ ] **Step 1: 创建 tags store**

创建文件 `src/stores/tags.ts`：

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { TagWithCount } from '../types/index';

export const useTagsStore = defineStore('tags', () => {
  const api = useApi();
  const tags = ref<TagWithCount[]>([]);
  const loading = ref(false);

  async function fetchTags() {
    loading.value = true;
    try {
      tags.value = await api.getTags();
    } finally {
      loading.value = false;
    }
  }

  async function createTag(name: string, color?: string) {
    const tag = await api.createTag(name, color);
    tags.value.push(tag);
    return tag;
  }

  async function deleteTag(id: number) {
    await api.deleteTag(id);
    tags.value = tags.value.filter(t => t.id !== id);
  }

  return { tags, loading, fetchTags, createTag, deleteTag };
});
```

- [ ] **Step 2: 创建 categories store**

创建文件 `src/stores/categories.ts`：

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { CategoryNode } from '../types/index';

export const useCategoriesStore = defineStore('categories', () => {
  const api = useApi();
  const categories = ref<CategoryNode[]>([]);
  const loading = ref(false);

  async function fetchCategories() {
    loading.value = true;
    try {
      categories.value = await api.getCategories();
    } finally {
      loading.value = false;
    }
  }

  async function createCategory(name: string, parentId?: number | null) {
    const node = await api.upsertCategory({ name, parentId });
    await fetchCategories();
    return node;
  }

  async function deleteCategory(id: number) {
    await api.deleteCategory(id);
    await fetchCategories();
  }

  return { categories, loading, fetchCategories, createCategory, deleteCategory };
});
```

- [ ] **Step 3: 提交**

```bash
git add src/stores/tags.ts src/stores/categories.ts
git commit -m "feat: add tags and categories Pinia stores"
```

---

## Task 12: CategoryTree 组件

**Files:**
- Create: `src/components/CategoryTree.vue`

- [ ] **Step 1: 创建 CategoryTree.vue**

创建文件 `src/components/CategoryTree.vue`：

```vue
<template>
  <div class="category-tree">
    <div class="tree-header">
      <span class="label">分类</span>
      <button class="btn-add" @click="showAdd = !showAdd" title="新建分类">＋</button>
    </div>
    <div v-if="showAdd" class="add-form">
      <input
        v-model="newName"
        class="add-input"
        placeholder="分类名称"
        @keyup.enter="handleAdd"
      />
      <button class="btn-confirm" @click="handleAdd">确定</button>
    </div>
    <div class="tree-content">
      <div
        class="tree-item all"
        :class="{ active: selectedId === null }"
        @click="$emit('select', null)"
      >
        📁 全部
      </div>
      <CategoryNodeItem
        v-for="node in categories"
        :key="node.id"
        :node="node"
        :depth="0"
        :selected-id="selectedId"
        @select="$emit('select', $event)"
        @delete="handleDelete"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { CategoryNode } from '../types/index';
import CategoryNodeItem from './CategoryNodeItem.vue';

const props = defineProps<{
  categories: CategoryNode[];
  selectedId: number | null;
}>();

const emit = defineEmits<{
  select: [id: number | null];
  refresh: [];
}>();

const showAdd = ref(false);
const newName = ref('');

async function handleAdd() {
  if (!newName.value.trim()) return;
  const { useCategoriesStore } = await import('../stores/categories');
  const store = useCategoriesStore();
  await store.createCategory(newName.value.trim());
  newName.value = '';
  showAdd.value = false;
  emit('refresh');
}

async function handleDelete(id: number) {
  const { useCategoriesStore } = await import('../stores/categories');
  const store = useCategoriesStore();
  await store.deleteCategory(id);
  emit('refresh');
}
</script>

<style scoped>
.category-tree { padding: 8px 0; }
.tree-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 0 12px 8px; border-bottom: 1px solid #e2e8f0; margin-bottom: 4px;
}
.label { font-size: 11px; color: #94a3b8; text-transform: uppercase; letter-spacing: 1px; }
.btn-add {
  background: none; border: none; font-size: 16px; color: #94a3b8;
  cursor: pointer; padding: 0 4px;
}
.btn-add:hover { color: #6366f1; }
.add-form {
  display: flex; gap: 4px; padding: 4px 12px 8px;
}
.add-input {
  flex: 1; border: 1px solid #e2e8f0; border-radius: 4px;
  padding: 4px 8px; font-size: 12px; outline: none;
}
.add-input:focus { border-color: #6366f1; }
.btn-confirm {
  padding: 4px 8px; font-size: 12px; background: #6366f1; color: white;
  border: none; border-radius: 4px; cursor: pointer;
}
.tree-content { font-size: 13px; }
.tree-item {
  padding: 6px 12px; cursor: pointer; border-radius: 6px; margin: 1px 4px;
  display: flex; align-items: center; justify-content: space-between;
}
.tree-item:hover { background: #f1f5f9; }
.tree-item.active { background: #6366f1; color: white; }
</style>
```

- [ ] **Step 2: 创建 CategoryNodeItem.vue**

创建文件 `src/components/CategoryNodeItem.vue`：

```vue
<template>
  <div>
    <div
      class="node-item"
      :style="{ paddingLeft: `${depth * 16 + 12}px` }"
      :class="{ active: node.id === selectedId }"
      @click="$emit('select', node.id)"
    >
      <span class="node-name">
        <span class="icon">{{ node.children.length ? '📂' : '📄' }}</span>
        {{ node.name }}
      </span>
      <span class="node-actions">
        <button class="btn-del" @click.stop="$emit('delete', node.id)" title="删除">×</button>
      </span>
    </div>
    <CategoryNodeItem
      v-for="child in node.children"
      :key="child.id"
      :node="child"
      :depth="depth + 1"
      :selected-id="selectedId"
      @select="$emit('select', $event)"
      @delete="$emit('delete', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import type { CategoryNode } from '../types/index';

defineProps<{
  node: CategoryNode;
  depth: number;
  selectedId: number | null;
}>();

defineEmits<{
  select: [id: number];
  delete: [id: number];
}>();
</script>

<style scoped>
.node-item {
  padding: 6px 12px 6px 12px; cursor: pointer; border-radius: 6px;
  margin: 1px 4px; display: flex; align-items: center; justify-content: space-between;
}
.node-item:hover { background: #f1f5f9; }
.node-item.active { background: #6366f1; color: white; }
.node-name { display: flex; align-items: center; gap: 4px; }
.icon { font-size: 12px; }
.btn-del {
  background: none; border: none; color: #cbd5e1; cursor: pointer;
  font-size: 14px; padding: 0 2px; opacity: 0;
}
.node-item:hover .btn-del { opacity: 1; }
.btn-del:hover { color: #ef4444; }
.node-item.active .btn-del { color: rgba(255,255,255,0.6); }
.node-item.active .btn-del:hover { color: #fff; }
</style>
```

- [ ] **Step 3: 提交**

```bash
git add src/components/CategoryTree.vue src/components/CategoryNodeItem.vue
git commit -m "feat: add CategoryTree and CategoryNodeItem components"
```

---

## Task 13: TagInput 组件

**Files:**
- Create: `src/components/TagInput.vue`

- [ ] **Step 1: 创建 TagInput.vue**

创建文件 `src/components/TagInput.vue`：

```vue
<template>
  <div class="tag-input">
    <div class="tags-area">
      <span
        v-for="tag in modelValue"
        :key="tag.id"
        class="tag-pill"
        :style="{ background: tag.color + '22', color: tag.color, borderColor: tag.color + '44' }"
      >
        {{ tag.name }}
        <button class="tag-remove" @click="removeTag(tag.id)">×</button>
      </span>
      <input
        ref="inputRef"
        v-model="query"
        class="tag-text-input"
        :placeholder="modelValue.length ? '' : '+ 添加标签'"
        @input="onInput"
        @keydown.enter.prevent="handleEnter"
        @keydown.backspace="handleBackspace"
        @blur="showSuggestions = false"
        @focus="onInput"
      />
    </div>
    <div v-if="showSuggestions && suggestions.length" class="suggestions">
      <div
        v-for="tag in suggestions"
        :key="tag.id"
        class="suggestion-item"
        @mousedown.prevent="addTag(tag)"
      >
        <span class="suggestion-dot" :style="{ background: tag.color }"></span>
        {{ tag.name }}
        <span class="suggestion-count">{{ tag.content_count }} 篇</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { TagWithCount } from '../types/index';

const props = defineProps<{
  modelValue: TagWithCount[];
  allTags: TagWithCount[];
}>();

const emit = defineEmits<{
  'update:modelValue': [tags: TagWithCount[]];
}>();

const query = ref('');
const showSuggestions = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

const suggestions = computed(() => {
  const q = query.value.toLowerCase().trim();
  const selectedIds = new Set(props.modelValue.map(t => t.id));
  return props.allTags.filter(t => {
    if (selectedIds.has(t.id)) return false;
    if (!q) return true;
    return t.name.toLowerCase().includes(q);
  }).slice(0, 8);
});

function addTag(tag: TagWithCount) {
  emit('update:modelValue', [...props.modelValue, tag]);
  query.value = '';
  showSuggestions.value = false;
}

function removeTag(id: number) {
  emit('update:modelValue', props.modelValue.filter(t => t.id !== id));
}

function handleEnter() {
  if (suggestions.value.length === 1) {
    addTag(suggestions.value[0]);
  } else if (query.value.trim()) {
    // If no match, emit a custom event for creating new tag
    const name = query.value.trim();
    const existing = props.allTags.find(t => t.name.toLowerCase() === name.toLowerCase());
    if (existing) {
      addTag(existing);
    } else {
      // Create temporary tag for parent to handle
      emit('update:modelValue', [...props.modelValue, {
        id: -Date.now(),
        name,
        color: '#6366f1',
        tag_type: 'manual' as const,
        content_count: 0,
        created_at: new Date().toISOString(),
      }]);
      query.value = '';
      showSuggestions.value = false;
    }
  }
}

function handleBackspace() {
  if (!query.value && props.modelValue.length) {
    removeTag(props.modelValue[props.modelValue.length - 1].id);
  }
}

function onInput() {
  showSuggestions.value = true;
}
</script>

<style scoped>
.tag-input { position: relative; }
.tags-area {
  display: flex; flex-wrap: wrap; gap: 6px; align-items: center;
  padding: 6px 8px; border: 1px solid #e2e8f0; border-radius: 8px;
  min-height: 38px; background: white;
}
.tags-area:focus-within { border-color: #6366f1; }
.tag-pill {
  display: flex; align-items: center; gap: 4px;
  padding: 2px 10px; border-radius: 12px; font-size: 12px;
  border: 1px solid; white-space: nowrap;
}
.tag-remove {
  background: none; border: none; cursor: pointer;
  font-size: 14px; padding: 0; line-height: 1; opacity: 0.6;
}
.tag-remove:hover { opacity: 1; }
.tag-text-input {
  border: none; outline: none; font-size: 13px;
  flex: 1; min-width: 80px; padding: 2px 0;
}
.suggestions {
  position: absolute; top: 100%; left: 0; right: 0;
  background: white; border: 1px solid #e2e8f0; border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1); z-index: 50;
  max-height: 200px; overflow-y: auto; margin-top: 4px;
}
.suggestion-item {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; cursor: pointer; font-size: 13px;
}
.suggestion-item:hover { background: #f1f5f9; }
.suggestion-dot { width: 8px; height: 8px; border-radius: 50%; }
.suggestion-count { margin-left: auto; font-size: 11px; color: #94a3b8; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/TagInput.vue
git commit -m "feat: add TagInput component with autocomplete and inline creation"
```

---

## Task 14: TagsView — 标签管理页

**Files:**
- Create: `src/views/TagsView.vue`

- [ ] **Step 1: 创建 TagsView.vue**

创建文件 `src/views/TagsView.vue`：

```vue
<template>
  <div class="tags-view">
    <div class="page-header">
      <h2>标签管理</h2>
      <button class="btn-primary" @click="showAdd = true">＋ 新建标签</button>
    </div>

    <div v-if="showAdd" class="add-form">
      <input v-model="newName" class="input" placeholder="标签名称" />
      <input v-model="newColor" class="input color-input" type="color" />
      <button class="btn-primary btn-sm" @click="handleCreate" :disabled="!newName.trim()">确定</button>
      <button class="btn-cancel" @click="showAdd = false">取消</button>
    </div>

    <input v-model="search" class="input search-input" placeholder="搜索标签..." />

    <div v-if="loading" class="empty-state">加载中...</div>
    <div v-else-if="filteredTags.length === 0" class="empty-state">暂无标签</div>
    <div v-else class="tag-list">
      <div v-for="tag in filteredTags" :key="tag.id" class="tag-row">
        <div class="tag-info">
          <span class="color-dot" :style="{ background: tag.color }"></span>
          <span class="tag-name">{{ tag.name }}</span>
          <span class="tag-count">{{ tag.content_count }} 篇内容</span>
          <span v-if="tag.tag_type === 'auto'" class="tag-badge auto">auto</span>
          <span v-else class="tag-badge manual">manual</span>
        </div>
        <div class="tag-actions">
          <button class="btn-text" @click="handleDelete(tag.id, tag.name)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useTagsStore } from '../stores/tags';

const store = useTagsStore();
const { tags, loading } = store;

const showAdd = ref(false);
const newName = ref('');
const newColor = ref('#6366f1');
const search = ref('');

const filteredTags = computed(() => {
  const q = search.value.toLowerCase().trim();
  if (!q) return tags;
  return tags.filter(t => t.name.toLowerCase().includes(q));
});

onMounted(() => store.fetchTags());

async function handleCreate() {
  if (!newName.value.trim()) return;
  await store.createTag(newName.value.trim(), newColor.value);
  newName.value = '';
  newColor.value = '#6366f1';
  showAdd.value = false;
}

async function handleDelete(id: number, name: string) {
  if (confirm(`确定删除标签「${name}」吗？`)) {
    await store.deleteTag(id);
  }
}
</script>

<style scoped>
.tags-view { width: 100%; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-header h2 { font-size: 22px; }
.btn-primary {
  padding: 8px 16px; background: #6366f1; color: white;
  border: none; border-radius: 8px; font-size: 14px; cursor: pointer;
}
.btn-sm { padding: 6px 12px; font-size: 13px; }
.btn-cancel {
  padding: 6px 12px; background: none; border: 1px solid #e2e8f0;
  border-radius: 8px; font-size: 13px; cursor: pointer; color: #64748b;
}
.add-form {
  display: flex; gap: 8px; align-items: center;
  margin-bottom: 16px; padding: 12px; background: #f8fafc; border-radius: 8px;
}
.input {
  border: 1px solid #e2e8f0; border-radius: 6px; padding: 8px 12px;
  font-size: 14px; outline: none;
}
.input:focus { border-color: #6366f1; }
.color-input { width: 40px; height: 36px; padding: 2px; cursor: pointer; }
.search-input { width: 100%; margin-bottom: 16px; box-sizing: border-box; }
.tag-list { display: flex; flex-direction: column; gap: 6px; }
.tag-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px; background: white; border: 1px solid #f1f5f9;
  border-radius: 8px;
}
.tag-row:hover { border-color: #e2e8f0; }
.tag-info { display: flex; align-items: center; gap: 8px; }
.color-dot { width: 10px; height: 10px; border-radius: 50%; }
.tag-name { font-size: 14px; font-weight: 500; }
.tag-count { font-size: 12px; color: #94a3b8; }
.tag-badge {
  font-size: 10px; padding: 1px 6px; border-radius: 4px;
  text-transform: uppercase;
}
.tag-badge.auto { background: #f0f0ff; color: #6366f1; }
.tag-badge.manual { background: #f0fdf4; color: #16a34a; }
.tag-actions { display: flex; gap: 8px; }
.btn-text {
  background: none; border: none; color: #94a3b8; font-size: 12px;
  cursor: pointer;
}
.btn-text:hover { color: #ef4444; }
.empty-state { text-align: center; color: #94a3b8; padding: 40px; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/TagsView.vue
git commit -m "feat: add TagsView page for tag management"
```

---

## Task 15: AppLayout 改造 — 侧边栏加分类树和标签导航

**Files:**
- Modify: `src/components/AppLayout.vue`

- [ ] **Step 1: 替换 AppLayout.vue 为新布局**

将 `src/components/AppLayout.vue` 替换为：

```vue
<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <img src="/logo-no-text.png" alt="OmniLink" class="logo" />
        <h1>OmniLink</h1>
      </div>
      <nav class="nav-section">
        <router-link to="/links" class="nav-item" active-class="active">
          <span class="icon">🔗</span> 链接
        </router-link>
        <router-link to="/tags" class="nav-item" active-class="active">
          <span class="icon">🏷️</span> 标签
        </router-link>
        <router-link to="/settings" class="nav-item" active-class="active">
          <span class="icon">⚙️</span> 设置
        </router-link>
      </nav>
      <div class="category-section">
        <CategoryTree
          :categories="categories"
          :selected-id="selectedCategoryId"
          @select="handleCategorySelect"
          @refresh="fetchCategories"
        />
      </div>
    </aside>
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, provide } from 'vue';
import { useRouter } from 'vue-router';
import CategoryTree from './CategoryTree.vue';
import { useCategoriesStore } from '../stores/categories';

const router = useRouter();
const categoriesStore = useCategoriesStore();
const categories = ref(categoriesStore.categories);
const selectedCategoryId = ref<number | null>(null);

provide('selectedCategoryId', selectedCategoryId);

onMounted(async () => {
  await categoriesStore.fetchCategories();
  categories.value = categoriesStore.categories;
});

async function fetchCategories() {
  await categoriesStore.fetchCategories();
  categories.value = categoriesStore.categories;
}

function handleCategorySelect(id: number | null) {
  selectedCategoryId.value = id;
  // Navigate to links with category filter
  router.push({ path: '/links', query: id ? { category: String(id) } : {} });
}
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
}
.sidebar {
  width: 220px;
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
.logo { width: 28px; height: 28px; }
.sidebar-header h1 { font-size: 18px; margin: 0; }
.nav-section { padding: 8px; }
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
.nav-item:hover { background: #e2e8f0; }
.nav-item.active { background: #6366f1; color: white; }
.category-section {
  flex: 1;
  border-top: 1px solid #e2e8f0;
  padding-top: 8px;
}
.main-content { flex: 1; overflow-y: auto; padding: 24px; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/AppLayout.vue
git commit -m "feat: add category tree and tag nav to sidebar layout"
```

---

## Task 16: ContentView 改造 — 内嵌标签编辑 + 分类显示

**Files:**
- Modify: `src/views/ContentView.vue`

- [ ] **Step 1: 在 ContentView.vue 中加标签编辑和分类显示**

在 `<script setup>` 中的 imports 区域追加：

```typescript
import { useTagsStore } from '../stores/tags';
import { useApi } from '../composables/useApi';
import TagInput from '../components/TagInput.vue';
import type { TagWithCount } from '../types/index';
```

在 `const analyzing = ref(false);` 之后加：

```typescript
const tagsStore = useTagsStore();
const contentTags = ref<TagWithCount[]>([]);

onMounted(async () => {
  await fetchDetail();
  await tagsStore.fetchTags();
});
```

在 `fetchDetail` 函数的 `detail.value = await api.getLinkDetail(...)` 之后加：

```typescript
// Load current tags for this content
if (detail.value?.content && detail.value?.ai) {
  contentTags.value = detail.value.ai.tags.map(name => {
    const existing = tagsStore.tags.find(t => t.name === name);
    return existing || {
      id: -Date.now(),
      name,
      color: '#6366f1',
      tag_type: 'auto' as const,
      content_count: 0,
      created_at: '',
    };
  });
}
```

在 `analyzeContent` 函数中，`await fetchDetail()` 之后加：

```typescript
await tagsStore.fetchTags();
```

新增函数：

```typescript
async function handleTagsUpdate(newTags: TagWithCount[]) {
  if (!detail.value?.content) return;
  contentTags.value = newTags;
  const tagIds = newTags.map(t => t.id).filter(id => id > 0);
  await api.updateContentTags(detail.value.content.id, tagIds);
}
```

在 `<template>` 中，将原来 `detail.ai` 区域的 `.tags` div 替换为 TagInput 组件：

把原来：
```html
<div class="tags" v-if="detail.ai.tags.length">
  <span v-for="tag in detail.ai.tags" :key="tag" class="tag">{{ tag }}</span>
</div>
```

替换为：
```html
<TagInput
  v-if="detail.content"
  :model-value="contentTags"
  :all-tags="tagsStore.tags"
  @update:model-value="handleTagsUpdate"
/>
```

在 `.meta` 区域的分类显示中，加一个分类 badge（如果 link 有 category_id）：

在 `<a ...>查看原文 →</a>` 之后加：

```html
<span v-if="categoryName" class="category-badge">{{ categoryName }}</span>
```

在 `<script setup>` 中加 computed：

```typescript
import { computed } from 'vue';
import { useCategoriesStore } from '../stores/categories';

const categoriesStore = useCategoriesStore();

const categoryName = computed(() => {
  if (!detail.value?.link.category_id) return null;
  const find = (nodes: any[]): string | null => {
    for (const n of nodes) {
      if (n.id === detail.value!.link.category_id) return n.name;
      const found = find(n.children);
      if (found) return found;
    }
    return null;
  };
  return find(categoriesStore.categories);
});
```

在 `<style scoped>` 中加：

```css
.category-badge {
  font-size: 11px; background: #f0f0ff; padding: 2px 8px;
  border-radius: 4px; color: #6366f1;
}
```

- [ ] **Step 2: 提交**

```bash
git add src/views/ContentView.vue
git commit -m "feat: inline tag editing and category display in content view"
```

---

## Task 17: LinksView 改造 — 分类筛选 + 路由注册

**Files:**
- Modify: `src/views/LinksView.vue`
- Modify: `src/router/index.ts`

- [ ] **Step 1: 在 LinksView.vue 中加分类筛选支持**

在 `<script setup>` 的 imports 中加：

```typescript
import { useRoute } from 'vue-router';
import { watch } from 'vue';
```

在 `const activeFilter = ref<string>('');` 之后加：

```typescript
const route = useRoute();

// Handle category filter from sidebar
watch(() => route.query.category, (catId) => {
  if (catId) {
    activeFilter.value = '';
    api.getLinks({ status: null }).then(res => {
      // Client-side filter by category — server-side filter comes later
      store.links = res.links;
      store.total = res.total;
    });
  }
}, { immediate: true });
```

- [ ] **Step 2: 在 router/index.ts 中加 /tags 路由**

在 `src/router/index.ts` 的 routes 数组中，`settings` 路由之前加：

```typescript
{ path: '/tags', name: 'tags', component: () => import('../views/TagsView.vue') },
```

- [ ] **Step 3: 提交**

```bash
git add src/views/LinksView.vue src/router/index.ts
git commit -m "feat: add category filter to links view and /tags route"
```

---

## Task 18: 编译验证 + 手动冒烟测试

- [ ] **Step 1: Rust 编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无错误

- [ ] **Step 2: 前端 TypeScript 检查**

Run: `cd /Users/niean/Documents/project/omni-link && npx vue-tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 3: 启动开发服务器**

Run: `npm run tauri dev`
Expected: 应用正常启动，窗口打开

- [ ] **Step 4: 冒烟测试清单**

手动验证以下功能：
1. 应用启动后侧边栏显示分类树和标签导航
2. 点击「标签」进入标签管理页
3. 创建一个新标签（输入名称 + 选颜色）
4. 添加一个链接并解析
5. 对解析后的内容执行 AI 分析
6. 验证 AI 分析后标签自动出现在标签管理页
7. 在内容详情页内嵌编辑标签（添加/移除）
8. 在侧边栏分类树中新建分类
9. 点击分类树节点筛选链接列表

- [ ] **Step 5: 最终提交**

```bash
git add -A
git commit -m "feat: complete P0 tags and categories management"
```

---

---

---

---
