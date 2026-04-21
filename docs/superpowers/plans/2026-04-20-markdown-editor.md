# Markdown Editor 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 ContentView 中添加 Markdown 编辑器，允许用户编辑解析后的标题和正文并保存。

**Architecture:** 抽取 ContentEditor.vue 子组件（MdEditorV3），ContentView 通过 isEditing 状态切换只读/编辑模式。后端新增 update_content command 和对应的 repo 方法。

**Tech Stack:** md-editor-v3 (Vue 3 Markdown 编辑器)、marked (已有)、Tauri IPC

---

## File Structure

| Action | File | Responsibility |
|--------|------|---------------|
| Modify | `src-tauri/src/db/schema.rs` | 添加 contents.updated_at 迁移 |
| Modify | `src-tauri/src/models.rs` | 添加 UpdateContentInput struct |
| Modify | `src-tauri/src/repositories/content_repo.rs` | 添加 update_content 方法 |
| Modify | `src-tauri/src/commands/content_commands.rs` | 添加 update_content_cmd command |
| Modify | `src-tauri/src/lib.rs` | 注册新 command |
| Modify | `src/composables/useApi.ts` | 添加 updateContent API 方法 |
| Create | `src/components/ContentEditor.vue` | Markdown 编辑器子组件 |
| Modify | `src/views/ContentView.vue` | 集成编辑模式切换 |

---

### Task 1: 安装 md-editor-v3 依赖

- [ ] **Step 1: 安装依赖**

Run: `npm install md-editor-v3`

- [ ] **Step 2: 验证安装成功**

Run: `cat package.json | grep md-editor-v3`
Expected: 看到 `md-editor-v3` 版本号

---

### Task 2: 后端 — 添加 contents 表 updated_at 迁移

**Files:**
- Modify: `src-tauri/src/db/schema.rs:98-107`

- [ ] **Step 1: 在 migrate 函数末尾添加 updated_at 迁移**

在 `src-tauri/src/db/schema.rs` 的 `migrate` 函数中，`content_status` 迁移之后添加：

```rust
    // new: updated_at for contents
    let has_updated_at: bool = conn
        .prepare("SELECT updated_at FROM contents LIMIT 0")
        .is_ok();
    if !has_updated_at {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));")?;
    }
```

插入位置：在现有 `content_status` 迁移块（第 99-104 行）之后，`Ok(())` 之前。

- [ ] **Step 2: 验证 Rust 编译通过**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` 无错误

---

### Task 3: 后端 — 添加 UpdateContentInput model

**Files:**
- Modify: `src-tauri/src/models.rs` (文件末尾追加)

- [ ] **Step 1: 在 models.rs 末尾添加 UpdateContentInput**

在 `src-tauri/src/models.rs` 末尾追加：

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateContentInput {
    pub id: i64,
    pub title: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` 无错误

---

### Task 4: 后端 — 添加 update_content repo 方法

**Files:**
- Modify: `src-tauri/src/repositories/content_repo.rs`

- [ ] **Step 1: 添加 update_content 函数**

在 `src-tauri/src/repositories/content_repo.rs` 的 `get_content_by_id` 函数之后添加：

```rust
pub fn update_content(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    body_text: Option<&str>,
    body_html: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE contents SET title = ?, body_text = ?, body_html = ?, updated_at = datetime('now') WHERE id = ?",
        params![title, body_text, body_html, id],
    )?;
    Ok(())
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` 无错误

---

### Task 5: 后端 — 添加 update_content_cmd command + 注册

**Files:**
- Modify: `src-tauri/src/commands/content_commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 content_commands.rs 添加 import 和 command 函数**

在 `src-tauri/src/commands/content_commands.rs` 的 import 区域（第 7 行之后）添加 `UpdateContentInput`：

```rust
use crate::models::{AiResultParsed, ContentParsed, LinkDetail, UpdateContentInput};
```

替换原有 import 行：
```rust
use crate::models::{AiResultParsed, ContentParsed, LinkDetail};
```

在文件末尾添加新 command：

```rust
#[tauri::command]
pub async fn update_content_cmd(state: State<'_, DbState>, input: UpdateContentInput) -> AppResult<bool> {
    let conn = state.0.lock().unwrap();
    content_repo::update_content(
        &conn,
        input.id,
        input.title.as_deref(),
        input.body_text.as_deref(),
        input.body_html.as_deref(),
    )?;
    Ok(true)
}
```

- [ ] **Step 2: 在 lib.rs 注册新 command**

在 `src-tauri/src/lib.rs` 的 `tauri::generate_handler![]` 列表中，在 `analyze_content_cmd` 行之后添加：

```rust
            commands::content_commands::update_content_cmd,
```

- [ ] **Step 3: 验证 Rust 编译通过**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` 无错误

---

### Task 6: 前端 — 添加 updateContent API 方法

**Files:**
- Modify: `src/composables/useApi.ts`

- [ ] **Step 1: 在 useApi 中添加 updateContent 方法**

在 `src/composables/useApi.ts` 的 return 对象中，`updateLinkCategory` 方法之后添加：

```typescript
    updateContent: (id: number, title: string | null, bodyText: string, bodyHtml: string) =>
      invoke<boolean>('update_content_cmd', {
        input: { id, title, body_text: bodyText, body_html: bodyHtml },
      }),
```

---

### Task 7: 前端 — 创建 ContentEditor.vue 组件

**Files:**
- Create: `src/components/ContentEditor.vue`

- [ ] **Step 1: 创建 ContentEditor.vue**

创建 `src/components/ContentEditor.vue`，内容如下：

```vue
<template>
  <div class="content-editor">
    <div class="editor-header">
      <input v-model="editTitle" class="title-input" placeholder="标题" />
      <div class="editor-actions">
        <button class="btn-save" @click="handleSave" :disabled="saving">
          {{ saving ? '保存中...' : '保存' }}
        </button>
        <button class="btn-cancel" @click="emit('cancel')">取消</button>
      </div>
    </div>
    <MdEditor v-model="editBody" :language="'zh-CN'" :style="{ height: '60vh' }" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const props = defineProps<{
  initialTitle: string | null;
  initialBody: string;
}>();

const emit = defineEmits<{
  save: [title: string | null, bodyText: string];
  cancel: [];
}>();

const editTitle = ref(props.initialTitle ?? '');
const editBody = ref(props.initialBody);
const saving = ref(false);

async function handleSave() {
  saving.value = true;
  try {
    emit('save', editTitle.value || null, editBody.value);
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.content-editor {
  margin-top: 16px;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.title-input {
  font-size: 18px;
  font-weight: 600;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 8px 12px;
  flex: 1;
  margin-right: 12px;
  outline: none;
}

.title-input:focus {
  border-color: #6366f1;
}

.editor-actions {
  display: flex;
  gap: 8px;
}

.btn-save {
  padding: 8px 16px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.btn-save:disabled {
  opacity: 0.5;
}

.btn-cancel {
  padding: 8px 16px;
  background: white;
  color: #6366f1;
  border: 1px solid #6366f1;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}
</style>
```

---

### Task 8: 前端 — 修改 ContentView.vue 集成编辑器

**Files:**
- Modify: `src/views/ContentView.vue`

- [ ] **Step 1: 添加 import 和 ref**

在 `<script setup>` 区域：
- 在 import 行（第 73 行 `import { ref, computed, onMounted } from 'vue';`）中添加 `import`
- 添加 ContentEditor 和 isEditing 相关代码

具体修改：

1. 在第 80 行 `import TagInput` 之后添加：
```typescript
import ContentEditor from '../components/ContentEditor.vue';
```

2. 在第 94 行 `const editingCategory = ref(false);` 之后添加：
```typescript
const isEditing = ref(false);
```

- [ ] **Step 2: 添加编辑按钮**

在模板的 `.actions` div 中（第 29 行之后，`</div>` 之前），添加编辑按钮：

```html
            <button v-if="detail.content && !isEditing" class="btn-secondary" @click="isEditing = true">
              编辑
            </button>
```

- [ ] **Step 3: 替换内容展示区域为条件渲染**

将模板中第 57-67 行的内容展示区域：

```html
      <div v-if="detail.content" class="content-body">
        <div v-if="detail.content.body_text" class="markdown-content" v-html="renderedMarkdown"></div>
        <div v-else-if="detail.content.body_html" v-html="detail.content.body_html" class="html-content"></div>
        <p v-else class="empty-state">无可显示内容</p>
      </div>
      <div v-else class="empty-state">
        <p>内容尚未解析</p>
        <!-- <button class="btn-primary" @click="parseLink" :disabled="parsing">
          {{ parsing ? '解析中...' : '开始解析' }}
        </button> -->
      </div>
```

替换为：

```html
      <ContentEditor v-if="isEditing"
        :initial-title="detail.content?.title ?? null"
        :initial-body="detail.content?.body_text ?? ''"
        @save="handleSaveContent"
        @cancel="isEditing = false"
      />
      <template v-else>
        <div v-if="detail.content" class="content-body">
          <div v-if="detail.content.body_text" class="markdown-content" v-html="renderedMarkdown"></div>
          <div v-else-if="detail.content.body_html" v-html="detail.content.body_html" class="html-content"></div>
          <p v-else class="empty-state">无可显示内容</p>
        </div>
        <div v-else class="empty-state">
          <p>内容尚未解析</p>
        </div>
      </template>
```

- [ ] **Step 4: 添加 handleSaveContent 函数**

在 `renderedMarkdown` computed 之前（第 211 行之前）添加：

```typescript
async function handleSaveContent(title: string | null, bodyText: string) {
  if (!detail.value?.content) return;
  try {
    const html = marked(bodyText) as string;
    await api.updateContent(detail.value.content.id, title, bodyText, html);
    toast.show('内容已保存', 'success');
    isEditing.value = false;
    await fetchDetail();
  } catch (e) {
    toast.show('保存失败', 'error');
  }
}
```

- [ ] **Step 5: 验证前端编译通过**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建成功无错误

---

### Task 9: 提交代码

- [ ] **Step 1: 查看变更**

Run: `git status`
Run: `git diff`

- [ ] **Step 2: 暂存并提交**

```bash
git add -A
git commit -m "feat: add markdown editor for content editing"
```
