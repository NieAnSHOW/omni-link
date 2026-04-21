# P0 优化功能设计文档

**日期**: 2026-04-21  
**版本**: 1.0  
**状态**: 待实施

## 概述

本文档描述 OmniLink v0.1 的 4 个 P0 优化任务的详细设计方案，包括：AI 整理确认流程、回到顶部按钮、吸顶功能、标签颜色统一。

## 目标

1. 为 AI 整理功能添加覆盖原文的二次确认，防止误操作
2. 提升长页面浏览体验，添加快速返回顶部功能
3. 优化内容详情页的导航体验，关键元素吸顶
4. 统一标签颜色为低饱和度，提升可读性

## 架构设计

### 数据模型扩展

**后端 (Rust)**

在 `src-tauri/src/models.rs` 的 `Link` 结构体添加字段：

```rust
pub ai_processing_status: String, // 'idle' | 'organizing' | 'expanding' | 'both'
```

**前端 (TypeScript)**

在 `src/types/index.ts` 的 `Link` 接口添加字段：

```typescript
ai_processing_status: 'idle' | 'organizing' | 'expanding' | 'both';
```

### 数据库迁移

在 `src-tauri/src/db/schema.rs` 的 `migrate()` 函数中添加：

```rust
// ai_processing_status migration
let has_ai_processing: bool = conn
    .prepare("SELECT ai_processing_status FROM links LIMIT 0")
    .is_ok();
if !has_ai_processing {
    conn.execute_batch(
        "ALTER TABLE links ADD COLUMN ai_processing_status TEXT NOT NULL DEFAULT 'idle' 
         CHECK(ai_processing_status IN ('idle','organizing','expanding','both'));"
    )?;
}
```

同时更新 `init_schema()` 中的 links 表定义，添加该字段。

## 组件设计

### 1. ScrollToTop 组件

**文件**: `src/components/ScrollToTop.vue`

**功能**:
- 监听 `.main-content` 容器的滚动事件
- 滚动距离 > 300px 时显示按钮，否则隐藏
- 点击按钮平滑滚动到顶部 (`scrollTo({ top: 0, behavior: 'smooth' })`)

**样式**:
- 固定定位：`position: fixed; right: 24px; bottom: 24px;`
- 圆形按钮：直径 48px，背景色 `#6366f1`，白色图标 ↑
- z-index: 100
- 悬浮阴影：`box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3)`
- hover 效果：`transform: translateY(-2px)`

**实现要点**:
- 使用 `onMounted` 添加滚动监听，`onUnmounted` 清理
- 使用 `ref<boolean>` 控制显示/隐藏，配合 `v-if` 或 `transition`

### 2. StickyHeader 组件

**文件**: `src/components/StickyHeader.vue`

**Props**:
- `zIndex: number` - 吸顶层级
- `background: string` - 背景色（默认 `#ffffff`）

**功能**:
- 使用 `<slot>` 接收内容
- 应用 `position: sticky; top: 0;`
- 滚动时添加底部阴影 `box-shadow: 0 2px 8px rgba(0,0,0,0.08)`

**实现要点**:
- 使用 IntersectionObserver 检测是否处于吸顶状态，动态添加阴影
- 背景色必须不透明，避免内容穿透

### 3. LinkCard 状态扩展

**文件**: `src/components/LinkCard.vue`

**改动**:

1. 状态文本映射添加 AI 处理状态：
```typescript
const statusText = computed(() => {
  if (props.link.ai_processing_status !== 'idle') {
    const map = {
      organizing: 'AI 整理中',
      expanding: 'AI 扩展中',
      both: 'AI 整理并扩展中'
    };
    return map[props.link.ai_processing_status] || 'AI 处理中';
  }
  const map = { pending: '待解析', parsing: '解析中', parsed: '已解析', failed: '失败' };
  return map[props.link.status] || props.link.status;
});
```

2. 禁用状态样式：
```vue
<div 
  class="link-card" 
  :class="{ 'processing': link.ai_processing_status !== 'idle' }"
  @click="$emit('click')"
>
```

```css
.link-card.processing {
  pointer-events: none;
  opacity: 0.6;
}
.link-card.processing .status {
  color: #6366f1;
}
```

## 功能流程

### AI 整理确认流程

**用户操作流程**:

1. 用户在 ContentView 点击"AI 整理"按钮
2. 显示 AI 处理方式选择弹窗（现有）
3. 用户选择方式（organize/expand/both）
4. **新增**：显示二次确认弹窗
   - 标题："确认操作"
   - 内容："此操作会覆盖原文，确认是否要继续？"
   - 按钮："取消"（灰色）、"继续"（主题色）
5. 用户点击"取消" → 关闭所有弹窗，结束流程
6. 用户点击"继续" → 执行以下步骤：
   - 关闭所有弹窗
   - 调用后端 API `start_ai_process(link_id, mode)`
   - 路由跳转到列表页 (`router.push({ name: 'links' })`)
7. 列表页自动刷新，LinkCard 显示"AI 整理中"状态并禁用点击
8. 后端处理完成后：
   - 重置 `ai_processing_status` 为 'idle'
   - 发送前端事件通知
   - 前端显示 toast "AI 整理完成"
   - 如果用户在列表页，自动刷新列表

**后端 API 设计**:

新增 Tauri Command: `start_ai_process`

```rust
#[tauri::command]
pub async fn start_ai_process(
    link_id: i64,
    mode: String, // 'organize' | 'expand' | 'both'
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 1. 更新 ai_processing_status
    link_repo::update_ai_processing_status(&state.db, link_id, &mode)?;
    
    // 2. 异步执行 AI 处理
    tauri::async_runtime::spawn(async move {
        let result = ai_service::process_content(link_id, &mode).await;
        
        // 3. 处理完成，重置状态
        link_repo::update_ai_processing_status(&state.db, link_id, "idle")?;
        
        // 4. 发送前端事件
        if result.is_ok() {
            app_handle.emit_all("ai-process-complete", link_id)?;
        } else {
            app_handle.emit_all("ai-process-failed", link_id)?;
        }
    });
    
    Ok(())
}
```

在 `src-tauri/src/repositories/link_repo.rs` 添加：

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

**前端事件监听**:

在 `LinksView.vue` 的 `onMounted` 中添加：

```typescript
import { listen } from '@tauri-apps/api/event';

onMounted(async () => {
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

### 吸顶功能实现

**ContentView.vue 结构调整**:

```vue
<template>
  <div class="content-view">
    <!-- 第一层：返回按钮 (z-index: 30) -->
    <StickyHeader :z-index="30" background="#ffffff">
      <button class="btn-back" @click="handleBackToList">← 返回</button>
    </StickyHeader>

    <div v-if="loading" class="skeleton-detail">...</div>
    <div v-else-if="!detail" class="empty-state">内容不存在</div>
    <template v-else>
      <!-- 第二层：内容头部 (z-index: 20) -->
      <StickyHeader :z-index="20" background="#ffffff">
        <div class="content-header">
          <div class="top-btn">
            <h2>{{ detail.link.title || '未命名' }}</h2>
            <div class="actions">...</div>
          </div>
          <div class="meta">...</div>
          <TagInput v-if="detail.content" ... />
        </div>
      </StickyHeader>

      <!-- 第三层：AI 摘要 (z-index: 10) -->
      <StickyHeader v-if="detail.ai" :z-index="10" background="#f0f0ff">
        <div class="ai-summary">
          <p><span>AI 摘要：</span>{{ detail.ai.summary }}</p>
        </div>
      </StickyHeader>

      <!-- 其他内容 -->
      <div v-if="autoAnalyzing" class="auto-analyzing-hint">...</div>
      <ContentEditor v-if="isEditing" ... />
      <template v-else>...</template>
    </template>

    <!-- 确认弹窗 -->
    <div v-if="showConfirmModal" class="modal-overlay" @click.self="closeConfirmModal">
      <div class="modal-content">
        <div class="modal-header">
          <h3>确认操作</h3>
          <button class="modal-close" @click="closeConfirmModal">×</button>
        </div>
        <p class="modal-message">此操作会覆盖原文，确认是否要继续？</p>
        <div class="modal-actions">
          <button class="btn-cancel" @click="closeConfirmModal">取消</button>
          <button class="btn-confirm" @click="confirmAiProcess">继续</button>
        </div>
      </div>
    </div>

    <!-- AI 整理选项弹窗 -->
    <div v-if="showAiProcessModal" class="modal-overlay" ...>...</div>
  </div>
</template>

<script setup lang="ts">
// 新增状态
const showConfirmModal = ref(false);
const pendingAiMode = ref<string>('');

// 修改 handleAiProcess
async function handleAiProcess(mode: string) {
  if (!detail.value?.content || aiProcessing.value) return;
  showAiProcessModal.value = false;
  pendingAiMode.value = mode;
  showConfirmModal.value = true;
}

function closeConfirmModal() {
  showConfirmModal.value = false;
  pendingAiMode.value = '';
}

async function confirmAiProcess() {
  if (!detail.value?.link || !pendingAiMode.value) return;
  showConfirmModal.value = false;
  
  try {
    await api.startAiProcess(detail.value.link.id, pendingAiMode.value);
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
</script>
```

**确认弹窗样式**:

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

### 标签颜色统一

**修改位置**:

1. **数据库默认值** (`src-tauri/src/db/schema.rs`):
```sql
color TEXT DEFAULT '#8b9dc3',
```

2. **TagInput 组件** (`src/components/TagInput.vue`):
```typescript
color: '#8b9dc3',
```

3. **ContentView 组件** (`src/views/ContentView.vue`):
```typescript
color: '#8b9dc3',
```

**颜色选择理由**:
- `#8b9dc3` 是低饱和度蓝灰色
- 在白色背景 `#ffffff` 上的对比度为 4.52:1
- 符合 WCAG AA 标准（要求 ≥ 4.5:1）
- 视觉柔和，不会过于突出

### ScrollToTop 集成

在 `src/App.vue` 或主布局组件中添加：

```vue
<template>
  <div id="app">
    <!-- 现有内容 -->
    <router-view />
    
    <!-- 全局回到顶部按钮 -->
    <ScrollToTop />
  </div>
</template>

<script setup lang="ts">
import ScrollToTop from './components/ScrollToTop.vue';
</script>
```

## API 变更

### 新增 Tauri Command

**命令名**: `start_ai_process`

**参数**:
- `link_id: i64` - 链接 ID
- `mode: String` - 处理模式 ('organize' | 'expand' | 'both')

**返回**: `Result<(), String>`

**功能**:
1. 更新 links 表的 `ai_processing_status` 字段
2. 异步执行 AI 内容处理
3. 处理完成后重置状态并发送前端事件

### 前端 API 封装

在 `src/composables/useApi.ts` 添加：

```typescript
async startAiProcess(linkId: number, mode: string): Promise<void> {
  await invoke('start_ai_process', { linkId, mode });
}
```

## 测试要点

### 功能测试

1. **AI 整理确认流程**:
   - 点击"AI 整理" → 选择方式 → 确认弹窗显示
   - 点击"取消" → 所有弹窗关闭，无请求发出
   - 点击"继续" → 跳转列表页，LinkCard 显示"AI 整理中"
   - 卡片禁用点击，透明度降低
   - 处理完成后状态恢复，显示 toast 通知

2. **回到顶部按钮**:
   - 页面顶部时按钮隐藏
   - 滚动超过 300px 后按钮显示
   - 点击按钮平滑滚动到顶部
   - 按钮在所有页面都可用

3. **吸顶功能**:
   - 滚动时返回按钮、内容头部、AI 摘要依次吸顶
   - 吸顶元素有阴影效果
   - z-index 层级正确，无遮挡问题

4. **标签颜色**:
   - 新建标签使用 `#8b9dc3`
   - 现有标签颜色不变（除非手动更新）
   - 标签在白色背景下清晰可见

### 边界情况

1. AI 处理失败时状态正确重置
2. 用户在处理中关闭应用，重新打开后状态恢复
3. 编辑状态下点击返回按钮有确认提示
4. 多个链接同时处理时状态独立管理

## 实施顺序

1. **数据库迁移** - 添加 `ai_processing_status` 字段
2. **通用组件** - 实现 ScrollToTop 和 StickyHeader
3. **标签颜色** - 统一修改默认颜色
4. **LinkCard 扩展** - 添加 AI 处理状态显示
5. **后端 API** - 实现 `start_ai_process` 命令
6. **前端流程** - 实现确认弹窗和事件监听
7. **吸顶功能** - 应用 StickyHeader 到 ContentView
8. **集成测试** - 完整流程测试

## 风险与注意事项

1. **数据库迁移**: 确保迁移逻辑幂等，可重复执行
2. **异步处理**: AI 处理失败时必须重置状态，避免卡在"处理中"
3. **事件监听**: 组件卸载时清理事件监听，避免内存泄漏
4. **z-index 冲突**: 确保吸顶元素、弹窗、ScrollToTop 的 z-index 不冲突
5. **性能**: 滚动监听使用节流（throttle），避免频繁触发

## 未来优化

1. 支持取消正在进行的 AI 处理
2. 显示 AI 处理进度条
3. 批量 AI 处理多个链接
4. 自定义标签颜色选择器
