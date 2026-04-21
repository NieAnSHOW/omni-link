# P0 优化功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 4 个 P0 优化任务：AI 整理确认流程、回到顶部按钮、吸顶功能、标签颜色统一

**Architecture:** 数据库扩展 Link 模型添加 ai_processing_status 字段，前端创建 ScrollToTop 和 StickyHeader 通用组件，后端新增 start_ai_process 异步命令，前端通过事件监听实现状态同步

**Tech Stack:** Rust (Tauri, rusqlite), Vue 3 (Composition API), TypeScript, SQLite

---

## 文件结构

### 新增文件
- `src/components/ScrollToTop.vue` - 全局回到顶部按钮组件
- `src/components/StickyHeader.vue` - 吸顶容器组件

### 修改文件
- `src-tauri/src/models.rs` - Link 模型添加 ai_processing_status 字段
- `src-tauri/src/db/schema.rs` - 数据库迁移添加新字段
- `src-tauri/src/repositories/link_repo.rs` - 添加更新 ai_processing_status 方法
- `src-tauri/src/commands/link.rs` - 添加 start_ai_process 命令
- `src-tauri/src/lib.rs` - 注册新命令
- `src/types/index.ts` - Link 接口添加 ai_processing_status 字段
- `src/composables/useApi.ts` - 添加 startAiProcess 方法
- `src/components/LinkCard.vue` - 添加 AI 处理状态显示和禁用逻辑
- `src/components/TagInput.vue` - 修改默认标签颜色
- `src/views/ContentView.vue` - 添加确认弹窗、吸顶功能、事件监听
- `src/views/LinksView.vue` - 添加 AI 处理完成事件监听
- `src/App.vue` - 集成 ScrollToTop 组件

---

## Task 1: 数据库迁移 - 添加 ai_processing_status 字段

**Files:**
- Modify: `src-tauri/src/models.rs:4-13`
- Modify: `src-tauri/src/db/schema.rs:7-16,76-101`

- [ ] **Step 1: 修改 Link 模型添加 ai_processing_status 字段**

在 `src-tauri/src/models.rs` 的 `Link` 结构体中添加字段：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub platform: Option<String>,
    pub source: String,
    pub status: String,
    pub ai_processing_status: String,
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 2: 更新 init_schema 中的 links 表定义**

在 `src-tauri/src/db/schema.rs` 的 `init_schema` 函数中，修改 links 表定义：

```rust
"CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    platform TEXT,
    source TEXT DEFAULT 'manual',
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending','parsing','parsed','failed')),
    ai_processing_status TEXT DEFAULT 'idle' CHECK(ai_processing_status IN ('idle','organizing','expanding','both')),
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);"
```

- [ ] **Step 3: 添加数据库迁移逻辑**

在 `src-tauri/src/db/schema.rs` 的 `migrate` 函数末尾（`drop_classification` 调用之后）添加：

```rust
// ai_processing_status migration
migrate_ai_processing_status(conn)?;

Ok(())
```

然后在文件末尾添加迁移函数：

```rust
fn migrate_ai_processing_status(conn: &Connection) -> AppResult<()> {
    let has_ai_processing: bool = conn
        .prepare("SELECT ai_processing_status FROM links LIMIT 0")
        .is_ok();
    if !has_ai_processing {
        conn.execute_batch(
            "ALTER TABLE links ADD COLUMN ai_processing_status TEXT NOT NULL DEFAULT 'idle' 
             CHECK(ai_processing_status IN ('idle','organizing','expanding','both'));"
        )?;
    }
    Ok(())
}
```

- [ ] **Step 4: 编译测试后端**

Run: `cd src-tauri && cargo build`
Expected: 编译成功，无错误

- [ ] **Step 5: 提交数据库迁移**

```bash
git add src-tauri/src/models.rs src-tauri/src/db/schema.rs
git commit -m "feat(db): 添加 ai_processing_status 字段到 Link 模型"
```

---

## Task 2: 前端类型同步

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: 修改 Link 接口添加 ai_processing_status 字段**

在 `src/types/index.ts` 的 `Link` 接口中添加：

```typescript
export interface Link {
  id: number;
  url: string;
  title: string | null;
  platform: string | null;
  source: string;
  status: 'pending' | 'parsing' | 'parsed' | 'failed';
  ai_processing_status: 'idle' | 'organizing' | 'expanding' | 'both';
  created_at: string;
  updated_at: string;
}
```

- [ ] **Step 2: 提交类型定义**

```bash
git add src/types/index.ts
git commit -m "feat(types): 添加 ai_processing_status 字段到 Link 接口"
```

---

## Task 3: 创建 ScrollToTop 组件

**Files:**
- Create: `src/components/ScrollToTop.vue`

- [ ] **Step 1: 创建 ScrollToTop 组件文件**

创建 `src/components/ScrollToTop.vue`：

```vue
<template>
  <transition name="fade">
    <button v-if="visible" class="scroll-to-top" @click="scrollToTop" aria-label="回到顶部">
      ↑
    </button>
  </transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const visible = ref(false);
let scrollContainer: HTMLElement | null = null;

function handleScroll() {
  if (scrollContainer) {
    visible.value = scrollContainer.scrollTop > 300;
  }
}

function scrollToTop() {
  if (scrollContainer) {
    scrollContainer.scrollTo({ top: 0, behavior: 'smooth' });
  }
}

onMounted(() => {
  scrollContainer = document.querySelector('.main-content');
  if (scrollContainer) {
    scrollContainer.addEventListener('scroll', handleScroll);
  }
});

onUnmounted(() => {
  if (scrollContainer) {
    scrollContainer.removeEventListener('scroll', handleScroll);
  }
});
</script>

<style scoped>
.scroll-to-top {
  position: fixed;
  right: 24px;
  bottom: 24px;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: #6366f1;
  color: white;
  border: none;
  font-size: 20px;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
  z-index: 100;
  transition: transform 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.scroll-to-top:hover {
  transform: translateY(-2px);
  background: #4f46e5;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
```

- [ ] **Step 2: 提交 ScrollToTop 组件**

```bash
git add src/components/ScrollToTop.vue
git commit -m "feat(components): 创建 ScrollToTop 回到顶部组件"
```

---

## Task 4: 创建 StickyHeader 组件

**Files:**
- Create: `src/components/StickyHeader.vue`

- [ ] **Step 1: 创建 StickyHeader 组件文件**

创建 `src/components/StickyHeader.vue`：

```vue
<template>
  <div 
    ref="headerRef"
    class="sticky-header" 
    :class="{ 'is-stuck': isStuck }"
    :style="{ 
      zIndex: zIndex, 
      background: background,
      top: 0
    }"
  >
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Props {
  zIndex: number;
  background?: string;
}

const props = withDefaults(defineProps<Props>(), {
  background: '#ffffff'
});

const headerRef = ref<HTMLElement | null>(null);
const isStuck = ref(false);
let observer: IntersectionObserver | null = null;

onMounted(() => {
  if (headerRef.value) {
    observer = new IntersectionObserver(
      ([entry]) => {
        isStuck.value = entry.intersectionRatio < 1;
      },
      { threshold: [1], rootMargin: '-1px 0px 0px 0px' }
    );
    observer.observe(headerRef.value);
  }
});

onUnmounted(() => {
  if (observer && headerRef.value) {
    observer.unobserve(headerRef.value);
    observer.disconnect();
  }
});
</script>

<style scoped>
.sticky-header {
  position: sticky;
  transition: box-shadow 0.2s;
}

.sticky-header.is-stuck {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
</style>
```

- [ ] **Step 2: 提交 StickyHeader 组件**

```bash
git add src/components/StickyHeader.vue
git commit -m "feat(components): 创建 StickyHeader 吸顶容器组件"
```

---

## Task 5: 统一标签颜色

**Files:**
- Modify: `src/components/TagInput.vue:69-76`
- Modify: `src/views/ContentView.vue:180-191`
- Modify: `src-tauri/src/db/schema.rs:30-36`

- [ ] **Step 1: 修改 TagInput 组件默认标签颜色**

在 `src/components/TagInput.vue` 的 `handleEnter` 函数中，将 `color: '#6366f1'` 改为 `color: '#8b9dc3'`：

```typescript
function handleEnter() {
  if (suggestions.value.length === 1) {
    addTag(suggestions.value[0]);
  } else if (query.value.trim()) {
    const name = query.value.trim();
    const existing = props.allTags.find(t => t.name.toLowerCase() === name.toLowerCase());
    if (existing) {
      addTag(existing);
    } else {
      emit('update:modelValue', [...props.modelValue, {
        id: -Date.now(),
        name,
        color: '#8b9dc3',
        tag_type: 'manual' as const,
        content_count: 0,
        created_at: new Date().toISOString(),
      }]);
      query.value = '';
      showSuggestions.value = false;
    }
  }
}
```

- [ ] **Step 2: 修改 ContentView 组件默认标签颜色**

在 `src/views/ContentView.vue` 的 `fetchDetail` 函数中，将 `color: '#6366f1'` 改为 `color: '#8b9dc3'`：

```typescript
if (detail.value?.content && detail.value?.ai) {
  contentTags.value = detail.value.ai.tags.map(name => {
    const existing = tagsStore.tags.find(t => t.name === name);
    return existing || {
      id: -Date.now(),
      name,
      color: '#8b9dc3',
      tag_type: 'auto' as const,
      content_count: 0,
      created_at: '',
    };
  });
}
```

- [ ] **Step 3: 修改数据库 tags 表默认颜色**

在 `src-tauri/src/db/schema.rs` 的 `init_schema` 函数中，修改 tags 表定义：

```rust
"CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT DEFAULT '#8b9dc3',
    type TEXT DEFAULT 'manual' CHECK(type IN ('auto','manual')),
    created_at TEXT DEFAULT (datetime('now'))
);"
```

- [ ] **Step 4: 提交标签颜色修改**

```bash
git add src/components/TagInput.vue src/views/ContentView.vue src-tauri/src/db/schema.rs
git commit -m "feat(ui): 统一标签颜色为低饱和度 #8b9dc3"
```

---

## Task 6: 扩展 LinkCard 组件显示 AI 处理状态

**Files:**
- Modify: `src/components/LinkCard.vue:2-31,33-66`

- [ ] **Step 1: 修改 LinkCard 状态文本计算逻辑**

在 `src/components/LinkCard.vue` 的 `<script setup>` 部分，修改 `statusText` 计算属性：

```typescript
const statusText = computed(() => {
  if (props.link.ai_processing_status !== 'idle') {
    const aiStatusMap: Record<string, string> = {
      organizing: 'AI 整理中',
      expanding: 'AI 扩展中',
      both: 'AI 整理并扩展中'
    };
    return aiStatusMap[props.link.ai_processing_status] || 'AI 处理中';
  }
  const map: Record<string, string> = { 
    pending: '待解析', 
    parsing: '解析中', 
    parsed: '已解析', 
    failed: '失败' 
  };
  return map[props.link.status] || props.link.status;
});
```

- [ ] **Step 2: 添加 processing 类名绑定**

在 `src/components/LinkCard.vue` 的 `<template>` 部分，修改根元素：

```vue
<div 
  class="link-card" 
  :class="{ 'processing': link.ai_processing_status !== 'idle' }"
  @click="$emit('click')"
>
```

- [ ] **Step 3: 添加 processing 状态样式**

在 `src/components/LinkCard.vue` 的 `<style scoped>` 部分末尾添加：

```css
.link-card.processing {
  pointer-events: none;
  opacity: 0.6;
}
.link-card.processing .status {
  color: #6366f1;
}
```

- [ ] **Step 4: 提交 LinkCard 扩展**

```bash
git add src/components/LinkCard.vue
git commit -m "feat(components): LinkCard 支持显示 AI 处理状态并禁用点击"
```

---

## Task 7: 后端添加 update_ai_processing_status 方法

**Files:**
- Modify: `src-tauri/src/repositories/link_repo.rs`

- [ ] **Step 1: 在 link_repo.rs 添加 update_ai_processing_status 函数**

在 `src-tauri/src/repositories/link_repo.rs` 文件末尾添加：

```rust
pub fn update_ai_processing_status(
    conn: &Connection,
    link_id: i64,
    status: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE links SET ai_processing_status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, link_id],
    )?;
    Ok(())
}
```

- [ ] **Step 2: 编译测试**

Run: `cd src-tauri && cargo build`
Expected: 编译成功

- [ ] **Step 3: 提交 link_repo 扩展**

```bash
git add src-tauri/src/repositories/link_repo.rs
git commit -m "feat(repo): 添加 update_ai_processing_status 方法"
```

---

## Task 8: 后端实现 start_ai_process 命令（第一部分）

**Files:**
- Modify: `src-tauri/src/commands/link.rs`

- [ ] **Step 1: 在 link.rs 添加 start_ai_process 命令函数**

在 `src-tauri/src/commands/link.rs` 文件末尾添加：

```rust
#[tauri::command]
pub async fn start_ai_process(
    link_id: i64,
    mode: String,
    state: tauri::State<'_, crate::AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use crate::repositories::link_repo;
    use crate::ai::ai_service;
    
    // 1. 更新 ai_processing_status
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        link_repo::update_ai_processing_status(&db, link_id, &mode)
            .map_err(|e| e.to_string())?;
    }
    
    // 2. 异步执行 AI 处理
    let db_clone = state.db.clone();
    let settings_clone = state.settings.clone();
    tauri::async_runtime::spawn(async move {
        // 获取 content_id
        let content_id_result = {
            let db = db_clone.lock().unwrap();
            db.query_row(
                "SELECT id FROM contents WHERE link_id = ?1",
                params![link_id],
                |row| row.get::<_, i64>(0),
            )
        };
        
        if let Ok(content_id) = content_id_result {
            // 执行 AI 处理
            let result = ai_service::process_content(
                content_id,
                &mode,
                &db_clone,
                &settings_clone,
            ).await;
            
            // 3. 处理完成，重置状态
            {
                let db = db_clone.lock().unwrap();
                let _ = link_repo::update_ai_processing_status(&db, link_id, "idle");
            }
            
            // 4. 发送前端事件
            if result.is_ok() {
                let _ = app_handle.emit_all("ai-process-complete", link_id);
            } else {
                let _ = app_handle.emit_all("ai-process-failed", link_id);
            }
        }
    });
    
    Ok(())
}
```

- [ ] **Step 2: 编译测试**

Run: `cd src-tauri && cargo build`
Expected: 编译成功

- [ ] **Step 3: 提交 start_ai_process 命令**

```bash
git add src-tauri/src/commands/link.rs
git commit -m "feat(commands): 实现 start_ai_process 异步命令"
```

---

## Task 9: 后端注册 start_ai_process 命令

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 lib.rs 的 invoke_handler 中注册命令**

在 `src-tauri/src/lib.rs` 的 `.invoke_handler(tauri::generate_handler![...])` 中添加 `link::start_ai_process`：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    link::start_ai_process,
])
```

- [ ] **Step 2: 编译测试**

Run: `cd src-tauri && cargo build`
Expected: 编译成功

- [ ] **Step 3: 提交命令注册**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(tauri): 注册 start_ai_process 命令"
```

---

## Task 10: 前端 API 封装

**Files:**
- Modify: `src/composables/useApi.ts`

- [ ] **Step 1: 在 useApi.ts 添加 startAiProcess 方法**

在 `src/composables/useApi.ts` 的返回对象中添加：

```typescript
async startAiProcess(linkId: number, mode: string): Promise<void> {
  await invoke('start_ai_process', { linkId, mode });
}
```

- [ ] **Step 2: 提交 API 封装**

```bash
git add src/composables/useApi.ts
git commit -m "feat(api): 添加 startAiProcess 方法"
```

---

## Task 11: ContentView 添加确认弹窗

**Files:**
- Modify: `src/views/ContentView.vue:100-147,273-290,293-334,339-380`

- [ ] **Step 1: 在 ContentView 添加确认弹窗状态**

在 `src/views/ContentView.vue` 的 `<script setup>` 部分，添加状态变量（在现有状态变量之后）：

```typescript
const showConfirmModal = ref(false);
const pendingAiMode = ref<string>('');
```

- [ ] **Step 2: 修改 handleAiProcess 函数**

替换现有的 `handleAiProcess` 函数：

```typescript
async function handleAiProcess(mode: string) {
  if (!detail.value?.content || aiProcessing.value) return;
  showAiProcessModal.value = false;
  pendingAiMode.value = mode;
  showConfirmModal.value = true;
}
```

- [ ] **Step 3: 添加确认弹窗相关函数**

在 `handleAiProcess` 函数之后添加：

```typescript
function closeConfirmModal() {
  showConfirmModal.value = false;
  pendingAiMode.value = '';
}

async function confirmAiProcess() {
  if (!detail.value?.link || !pendingAiMode.value) return;
  showConfirmModal.value = false;
  
  try {
    await api.startAiProcess(detail.value.link.id, pendingAiMode.value);
    toast.show('AI 处理已启动', 'success');
    router.push({ name: 'links' });
  } catch (e: any) {
    toast.show(e?.message || 'AI 处理启动失败', 'error');
  } finally {
    pendingAiMode.value = '';
  }
}

function handleBackToList() {
  if (isEditing.value) {
    if (confirm('有未保存的编辑，确认返回？')) {
      router.push({ name: 'links' });
    }
  } else {
    router.push({ name: 'links' });
  }
}
```

- [ ] **Step 4: 在 template 中添加确认弹窗 HTML**

在 `src/views/ContentView.vue` 的 `<template>` 部分，在 AI 整理选项弹窗之前添加：

```vue
<!-- 确认弹窗 -->
<div v-if="showConfirmModal" class="modal-overlay" @click.self="closeConfirmModal" @keydown.escape="closeConfirmModal">
  <div class="modal-content" role="dialog" aria-modal="true">
    <div class="modal-header">
      <h3>确认操作</h3>
      <button class="modal-close" @click="closeConfirmModal" aria-label="关闭">×</button>
    </div>
    <p class="modal-message">此操作会覆盖原文，确认是否要继续？</p>
    <div class="modal-actions">
      <button class="btn-cancel" @click="closeConfirmModal">取消</button>
      <button class="btn-confirm" @click="confirmAiProcess">继续</button>
    </div>
  </div>
</div>
```

- [ ] **Step 5: 添加确认弹窗样式**

在 `src/views/ContentView.vue` 的 `<style scoped>` 部分末尾添加：

```css
.modal-message {
  font-size: 14px;
  color: #334155;
  margin: 16px 0 24px;
  line-height: 1.6;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn-cancel {
  padding: 8px 20px;
  background: #f1f5f9;
  color: #64748b;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.btn-cancel:hover {
  background: #e2e8f0;
}

.btn-confirm {
  padding: 8px 20px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.btn-confirm:hover {
  background: #4f46e5;
}
```

- [ ] **Step 6: 提交确认弹窗功能**

```bash
git add src/views/ContentView.vue
git commit -m "feat(content): 添加 AI 整理二次确认弹窗"
```

---

## Task 12: LinksView 添加事件监听

**Files:**
- Modify: `src/views/LinksView.vue:43-74,162-166`

- [ ] **Step 1: 导入 Tauri 事件监听 API**

在 `src/views/LinksView.vue` 的 `<script setup>` 部分顶部添加导入：

```typescript
import { listen } from '@tauri-apps/api/event';
```

- [ ] **Step 2: 在 onMounted 中添加事件监听**

修改 `onMounted` 函数，在现有代码之后添加事件监听：

```typescript
onMounted(async () => {
  const el = document.querySelector('.main-content');
  if (el) {
    contentWidth.value = el.clientWidth;
    resizeObs = new ResizeObserver(([e]) => { contentWidth.value = e.contentRect.width });
    resizeObs.observe(el);
  }
  
  // 监听 AI 处理完成事件
  await listen('ai-process-complete', async () => {
    toast.show('AI 整理完成', 'success');
    await loadLinks();
  });
  
  await listen('ai-process-failed', async () => {
    toast.show('AI 整理失败', 'error');
    await loadLinks();
  });
});
```

- [ ] **Step 3: 提交事件监听**

```bash
git add src/views/LinksView.vue
git commit -m "feat(links): 添加 AI 处理完成事件监听"
```

---

## Task 13: ContentView 应用吸顶功能

**Files:**
- Modify: `src/views/ContentView.vue:1-98,278-290`

- [ ] **Step 1: 导入 StickyHeader 组件**

在 `src/views/ContentView.vue` 的 `<script setup>` 部分添加导入：

```typescript
import StickyHeader from '../components/StickyHeader.vue';
```

- [ ] **Step 2: 修改返回按钮应用吸顶**

在 `<template>` 部分，将返回按钮包裹在 StickyHeader 中：

```vue
<StickyHeader :z-index="30" background="#ffffff">
  <button class="btn-back" @click="handleBackToList">← 返回</button>
</StickyHeader>
```

- [ ] **Step 3: 修改内容头部应用吸顶**

将 `<div class="content-header">` 包裹在 StickyHeader 中：

```vue
<StickyHeader :z-index="20" background="#ffffff">
  <div class="content-header">
    <div class="top-btn">
      <h2>{{ detail.link.title || '未命名' }}</h2>
      <div class="actions">
        <button v-if="detail.link.status !== 'parsing'" class="btn-primary" @click="parseLink" :disabled="parsing">
          {{ parsing ? '解析中...' : (detail.content ? '重新解析' : '解析内容') }}
        </button>
        <button v-if="detail.content" class="btn-secondary" @click="analyzeContent"
          :disabled="analyzing">
          {{ analyzing ? '摘要生成中...' : (detail.ai ? '重新生成摘要' : 'AI 摘要') }}
        </button>
        <button v-if="detail.content" class="btn-secondary" @click="showAiProcessModal = true">
          AI 整理
        </button>
        <button v-if="detail.content && !isEditing" class="btn-secondary" @click="isEditing = true">
          编辑
        </button>
      </div>
    </div>
    <div class="meta">
      <span class="platform-badge">{{ detail.link.platform || 'web' }}</span>
      <a :href="detail.link.url" target="_blank" class="original-link">查看原文 →</a>
    </div>
    <TagInput v-if="detail.content" :model-value="contentTags" :all-tags="tagsStore.tags"
      @update:model-value="handleTagsUpdate" />
  </div>
</StickyHeader>
```

- [ ] **Step 4: 修改 AI 摘要应用吸顶**

将 `<div v-if="detail.ai" class="ai-summary">` 包裹在 StickyHeader 中：

```vue
<StickyHeader v-if="detail.ai" :z-index="10" background="#f0f0ff">
  <div class="ai-summary">
    <p><span>AI 摘要：</span>{{ detail.ai.summary }}</p>
  </div>
</StickyHeader>
```

- [ ] **Step 5: 移除 ai-summary 的原有 sticky 样式**

在 `<style scoped>` 部分，修改 `.ai-summary` 样式，移除 `position: sticky` 相关属性：

```css
.ai-summary {
  background: #f0f0ff;
  border-radius: 10px;
  padding: 16px;
  margin-top: 10px;
  margin-bottom: 10px;
}
```

- [ ] **Step 6: 提交吸顶功能**

```bash
git add src/views/ContentView.vue
git commit -m "feat(content): 应用 StickyHeader 实现吸顶功能"
```

---

## Task 14: App.vue 集成 ScrollToTop 组件

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 导入 ScrollToTop 组件**

在 `src/App.vue` 的 `<script setup>` 部分添加导入：

```typescript
import ScrollToTop from './components/ScrollToTop.vue';
```

- [ ] **Step 2: 在 template 中添加 ScrollToTop 组件**

在 `src/App.vue` 的 `<template>` 部分，在 `<router-view />` 之后添加：

```vue
<ScrollToTop />
```

- [ ] **Step 3: 提交 ScrollToTop 集成**

```bash
git add src/App.vue
git commit -m "feat(app): 集成 ScrollToTop 全局回到顶部按钮"
```

---

## Task 15: 完整功能测试

**Files:**
- Test: 所有修改的文件

- [ ] **Step 1: 启动开发服务器**

Run: `npm run dev:all`
Expected: 前端和后端都成功启动

- [ ] **Step 2: 测试数据库迁移**

1. 打开应用
2. 检查控制台无错误
3. 使用 SQLite 工具查看数据库，确认 links 表有 `ai_processing_status` 字段

Expected: 字段存在，默认值为 'idle'

- [ ] **Step 3: 测试 AI 整理确认流程**

1. 进入任意已解析的内容详情页
2. 点击"AI 整理"按钮
3. 选择任意处理方式（如"AI 整理内容"）
4. 确认弹窗显示，内容为"此操作会覆盖原文，确认是否要继续？"
5. 点击"取消"，所有弹窗关闭
6. 再次点击"AI 整理"，选择方式，点击"继续"
7. 页面跳转到列表页
8. 对应的 LinkCard 显示"AI 整理中"状态，透明度降低，无法点击
9. 等待处理完成，显示 toast "AI 整理完成"
10. LinkCard 状态恢复正常

Expected: 所有步骤按预期执行

- [ ] **Step 4: 测试回到顶部按钮**

1. 在列表页或详情页滚动到顶部
2. 确认回到顶部按钮不可见
3. 向下滚动超过 300px
4. 确认回到顶部按钮出现在右下角
5. 点击按钮
6. 页面平滑滚动到顶部

Expected: 按钮显示/隐藏正确，点击功能正常

- [ ] **Step 5: 测试吸顶功能**

1. 进入内容详情页
2. 向下滚动
3. 确认返回按钮首先吸顶（z-index 最高）
4. 继续滚动，确认内容头部吸顶
5. 继续滚动，确认 AI 摘要吸顶
6. 确认吸顶元素有阴影效果
7. 确认各层级无遮挡问题

Expected: 吸顶顺序正确，视觉效果符合预期

- [ ] **Step 6: 测试标签颜色**

1. 在内容详情页添加新标签
2. 确认新标签颜色为 `#8b9dc3`
3. 确认标签在白色背景下清晰可见

Expected: 标签颜色正确，可读性良好

- [ ] **Step 7: 测试边界情况**

1. 在编辑状态下点击返回按钮，确认有确认提示
2. 同时对多个链接执行 AI 整理，确认状态独立管理
3. 在 AI 处理中关闭应用，重新打开，确认状态正确恢复

Expected: 边界情况处理正确

- [ ] **Step 8: 最终提交**

```bash
git add -A
git commit -m "test: 完成 P0 优化功能完整测试"
```

---

## 自查清单

**规格覆盖检查**:
- ✅ AI 整理确认流程 - Task 11
- ✅ 回到顶部按钮 - Task 3, 14
- ✅ 吸顶功能 - Task 4, 13
- ✅ 标签颜色统一 - Task 5

**占位符检查**:
- ✅ 无 TBD、TODO、待补充
- ✅ 所有代码步骤都包含完整代码
- ✅ 所有命令都有预期输出

**类型一致性检查**:
- ✅ `ai_processing_status` 字段在 Rust 和 TypeScript 中类型一致
- ✅ `startAiProcess` 方法参数在前后端一致
- ✅ 事件名称 `ai-process-complete` 和 `ai-process-failed` 在发送和监听端一致
- ✅ 组件 props 命名一致（`zIndex`, `background`）

---

## 执行说明

本计划包含 15 个任务，每个任务都是独立的、可测试的单元。建议按顺序执行，每完成一个任务就提交一次。

**预计时间**: 2-3 小时

**关键依赖**:
- Task 1-2 必须先完成（数据模型基础）
- Task 7-9 必须按顺序完成（后端 API）
- Task 11-12 依赖 Task 10（前端 API）
- Task 13 依赖 Task 4（StickyHeader 组件）
- Task 14 依赖 Task 3（ScrollToTop 组件）
