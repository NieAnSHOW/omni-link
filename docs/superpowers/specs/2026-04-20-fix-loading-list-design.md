# P0 Bug 修复设计：列表加载与分类筛选

日期：2026-04-20

## 问题概述

3 个 P0 bug 根因均为数据流不统一：

1. 应用启动时链接列表不加载（需跳转后才显示）
2. 添加/删除链接后列表不更新
3. 点击分类时没有正确筛选

## 根因分析

- `LinksView.vue` 同时使用 `onMounted` 和 `route.query.category` watcher（`immediate: true`）双重加载机制，产生冲突
- `getLinks` API 不支持 `category_id` 参数，后端无法按分类筛选
- `ContentView.vue` 使用 `categoriesStore` 计算分类名，但从未调用 `fetchCategories()`
- 添加/删除后的刷新逻辑与筛选条件脱节

## 设计

### 第 1 节：统一加载逻辑（LinksView.vue）

**改动：** 删除 `onMounted` 中的 `store.fetchLinks()`，改为单一 `loadLinks()` 方法。

```ts
const loadLinks = async () => {
  const params: any = { limit: 20, offset: 0 };
  if (activeFilter.value) params.status = activeFilter.value;
  const catId = route.query.category;
  if (catId) {
    params.category_id = Number(catId);
    activeFilter.value = '';
  }
  const res = await api.getLinks(params);
  store.links = res.links;
  store.total = res.total;
};
```

- 一个 watcher 同时监听 `route.query.category` 和 `activeFilter`，触发 `loadLinks()`，`immediate: true` 首次加载
- `setFilter()` 只设置 `activeFilter` 值，不再直接调 API
- 分类点击切换：再次点击同一分类时清除 query（导航到 `/links`）
- 所有场景（首次、切分类、切状态、添加/删除后）都走 `loadLinks()`

### 第 2 节：后端分类筛选支持

**改动 4 层：**

1. **Rust Repo 层** (`link_repo.rs`)：`get_links` 新增 `category_id: Option<i64>` 参数，SQL 加 `WHERE category_id = ?` 条件
2. **Rust Command 层** (`commands/link.rs`)：`get_links` command 新增 `category_id` 入参，透传给 repo
3. **前端类型** (`useApi.ts`)：`getLinks` params 新增 `category_id?: number`
4. **前端调用**：`loadLinks()` 中传 `category_id`

### 第 3 节：添加/删除后刷新列表

**改动：** 统一用 `loadLinks()` 刷新。

- `store.addLink(url)` / `store.removeLink(id)` 只负责调用 API
- `LinksView.vue` 中调用方在操作成功后统一调 `loadLinks()`
- 筛选条件通过 `loadLinks()` 内部逻辑自动保持

### 第 4 节：ContentView 分类名显示

**改动：** 在 `ContentView.vue` 的数据加载中加一行 `categoriesStore.fetchCategories()`。

- store 内部已有防重判断（`if (categories.length > 0) return`），不会重复请求

## 涉及文件

- `src/views/LinksView.vue` — 加载逻辑重构
- `src/stores/links.ts` — addLink/removeLink 改为纯 API 调用
- `src/composables/useApi.ts` — getLinks 新增 category_id 参数
- `src-tauri/src/repositories/link_repo.rs` — SQL 加 category_id 条件
- `src-tauri/src/commands/link.rs` — command 新增 category_id 入参
- `src/views/ContentView.vue` — 加载分类数据
