# P0 Bug 修复实施计划：列表加载与分类筛选

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复链接列表加载、添加/删除后刷新、分类筛选三个 P0 bug

**Architecture:** 从后端到前端统一数据流。后端 get_links 新增 category_id 参数；前端用单一 loadLinks() 替代双重加载机制；store 的 addLink/removeLink 改为纯 API 调用，由视图层统一刷新。

**Tech Stack:** Rust (rusqlite) / Tauri v2 / Vue 3 + Pinia / TypeScript

---

### Task 1: 后端 Repo 层 — get_links 和 get_links_count 新增 category_id

**Files:**
- Modify: `src-tauri/src/repositories/link_repo.rs:33-53` (get_links)
- Modify: `src-tauri/src/repositories/link_repo.rs:76-83` (get_links_count)

- [ ] **Step 1: 修改 get_links 函数签名和实现**

将第 33 行的函数签名：
```rust
pub fn get_links(conn: &Connection, limit: i64, offset: i64, status: Option<&str>) -> AppResult<Vec<Link>> {
```
改为：
```rust
pub fn get_links(conn: &Connection, limit: i64, offset: i64, status: Option<&str>, category_id: Option<i64>) -> AppResult<Vec<Link>> {
```

将第 34-53 行的函数体替换为：
```rust
    let mut links = Vec::new();
    match (status, category_id) {
        (Some(s), Some(c)) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links WHERE status = ? AND category_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![s, c, limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
        (Some(s), None) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![s, limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
        (None, Some(c)) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links WHERE category_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![c, limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
        (None, None) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM links ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )?;
            let rows = stmt.query_map(params![limit, offset], |row| row_to_link(row))?;
            for row in rows {
                links.push(row?);
            }
        }
    }
    Ok(links)
```

- [ ] **Step 2: 修改 get_links_count 函数签名和实现**

将第 76 行的函数签名：
```rust
pub fn get_links_count(conn: &Connection, status: Option<&str>) -> AppResult<i64> {
```
改为：
```rust
pub fn get_links_count(conn: &Connection, status: Option<&str>, category_id: Option<i64>) -> AppResult<i64> {
```

将第 77-83 行的函数体替换为：
```rust
    let count: i64 = match (status, category_id) {
        (Some(s), Some(c)) => {
            conn.query_row("SELECT COUNT(*) FROM links WHERE status = ? AND category_id = ?", params![s, c], |row| row.get(0))?
        }
        (Some(s), None) => {
            conn.query_row("SELECT COUNT(*) FROM links WHERE status = ?", params![s], |row| row.get(0))?
        }
        (None, Some(c)) => {
            conn.query_row("SELECT COUNT(*) FROM links WHERE category_id = ?", params![c], |row| row.get(0))?
        }
        (None, None) => {
            conn.query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))?
        }
    };
    Ok(count)
```

- [ ] **Step 3: 检查是否有其他调用 get_links_by_category 的地方需要更新**

Run: `grep -rn "get_links_by_category" src-tauri/src/`

如果只有 link_repo.rs 中的定义而没有调用方，该函数可以保留不动（不在本次修改范围内）。

- [ ] **Step 4: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | head -30`

Expected: 编译错误提示 link_commands.rs 中的调用参数不匹配（这是预期的，Task 2 修复）

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/repositories/link_repo.rs
git commit -m "refactor(repo): add category_id param to get_links and get_links_count"
```

---

### Task 2: 后端 Command 层 — get_links command 新增 category_id

**Files:**
- Modify: `src-tauri/src/commands/link_commands.rs:10-23`

- [ ] **Step 1: 修改 get_links command 签名和调用**

将第 10-23 行替换为：
```rust
#[tauri::command]
pub async fn get_links(
    state: State<'_, DbState>,
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<String>,
    category_id: Option<i64>,
) -> AppResult<LinksResponse> {
    let conn = state.0.lock().unwrap();
    let lim = limit.unwrap_or(20);
    let off = offset.unwrap_or(0);
    let links = link_repo::get_links(&conn, lim, off, status.as_deref(), category_id)?;
    let total = link_repo::get_links_count(&conn, status.as_deref(), category_id)?;
    Ok(LinksResponse { links, total })
}
```

- [ ] **Step 2: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | head -30`
Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/link_commands.rs
git commit -m "feat(cmd): add category_id param to get_links command"
```

---

### Task 3: 前端 API 层 — useApi.ts 新增 category_id 参数

**Files:**
- Modify: `src/composables/useApi.ts:6-11`

- [ ] **Step 1: 修改 getLinks 参数类型和 invoke 调用**

将第 6-11 行替换为：
```ts
    getLinks: (params?: { limit?: number; offset?: number; status?: string; category_id?: number }) =>
      invoke<{ links: Link[]; total: number }>('get_links', {
        limit: params?.limit ?? null,
        offset: params?.offset ?? null,
        status: params?.status ?? null,
        category_id: params?.category_id ?? null,
      }),
```

- [ ] **Step 2: Commit**

```bash
git add src/composables/useApi.ts
git commit -m "feat(api): add category_id param to getLinks"
```

---

### Task 4: 前端 Store 层 — 简化 addLink/removeLink

**Files:**
- Modify: `src/stores/links.ts`

- [ ] **Step 1: 重写 store，移除内部 fetch 调用**

将整个文件内容替换为：
```ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { Link } from '../types/index';

export const useLinksStore = defineStore('links', () => {
  const api = useApi();
  const links = ref<Link[]>([]);
  const total = ref(0);
  const loading = ref(false);

  async function addLink(url: string) {
    await api.addLink(url);
  }

  async function addLinks(urls: string[]) {
    await api.addLinks(urls);
  }

  async function removeLink(id: number) {
    await api.deleteLink(id);
  }

  return { links, total, loading, addLink, addLinks, removeLink };
});
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/links.ts
git commit -m "refactor(store): simplify links store to pure API calls"
```

---

### Task 5: 前端视图层 — 重构 LinksView.vue 加载逻辑（核心）

**Files:**
- Modify: `src/views/LinksView.vue`

- [ ] **Step 1: 替换整个 script setup 部分**

将第 36-93 行的 `<script setup lang="ts">` 块替换为：
```ts
<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useLinksStore } from '../stores/links';
import { useApi } from '../composables/useApi';
import LinkCard from '../components/LinkCard.vue';
import AddLinkDialog from '../components/AddLinkDialog.vue';

const router = useRouter();
const store = useLinksStore();
const api = useApi();
const showDialog = ref(false);
const activeFilter = ref<string>('');
const route = useRoute();

const { links, loading } = store;

const filters = [
  { label: '全部', value: '' },
  { label: '待解析', value: 'pending' },
  { label: '已解析', value: 'parsed' },
  { label: '失败', value: 'failed' },
];

async function loadLinks() {
  store.loading = true;
  try {
    const params: { limit: number; offset: number; status?: string; category_id?: number } = { limit: 20, offset: 0 };
    if (activeFilter.value) {
      params.status = activeFilter.value;
    }
    const catId = route.query.category;
    if (catId) {
      params.category_id = Number(catId);
    }
    const res = await api.getLinks(params);
    store.links = res.links;
    store.total = res.total;
  } finally {
    store.loading = false;
  }
}

watch(
  [() => route.query.category, activeFilter],
  () => {
    const catId = route.query.category;
    if (catId) {
      activeFilter.value = '';
    }
    loadLinks();
  },
  { immediate: true },
);

function setFilter(status: string) {
  activeFilter.value = status;
}

function goToDetail(id: number) {
  router.push({ name: 'content', params: { id } });
}

async function handleAddLinks(urls: string[]) {
  await api.addLinks(urls);
  await loadLinks();
}

async function handleDelete(id: number) {
  await api.deleteLink(id);
  await loadLinks();
}
</script>
```

- [ ] **Step 2: 验证页面加载**

Run: `npm run dev`
在浏览器打开 http://localhost:1420，确认：
- 进入链接页面时列表自动加载
- 切换状态筛选（全部/待解析/已解析/失败）正常
- 点击侧边栏分类，列表按分类筛选
- 再次点击同一分类或点击"全部"，取消筛选
- 添加链接后列表刷新
- 删除链接后列表刷新

- [ ] **Step 3: Commit**

```bash
git add src/views/LinksView.vue
git commit -m "fix(LinksView): unify loading logic with single loadLinks entry point"
```

---

### Task 6: 前端视图层 — 修复 ContentView 分类名显示

**Files:**
- Modify: `src/views/ContentView.vue:73-76`

- [ ] **Step 1: 在 onMounted 中添加 fetchCategories 调用**

将第 73-76 行：
```ts
onMounted(async () => {
  await fetchDetail();
  await tagsStore.fetchTags();
});
```
改为：
```ts
onMounted(async () => {
  await fetchDetail();
  await tagsStore.fetchTags();
  await categoriesStore.fetchCategories();
});
```

- [ ] **Step 2: 验证分类名显示**

进入一个已设置分类的链接详情页，确认分类名 badge 正确显示。

- [ ] **Step 3: Commit**

```bash
git add src/views/ContentView.vue
git commit -m "fix(ContentView): load categories to display category name badge"
```
