# Vditor 替换 md-editor-v3 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 NoteDetail.vue 中的 md-editor-v3 替换为 Vditor，实现 Typora 风格的即时渲染（IR）编辑体验。

**Architecture:** 新建 VditorEditor.vue wrapper 组件封装 Vditor 实例，对外暴露 v-model 接口；修改 NoteDetail.vue 替换编辑器组件；通过 useTheme() composable 响应暗色模式切换。

**Tech Stack:** Vue 3 Composition API、Vditor、TypeScript、Tailwind CSS

---

### Task 1: 安装 Vditor 依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 添加 vditor 依赖**

```bash
cd /Users/niean/Documents/project/omni-link && npm install vditor
```

- [ ] **Step 2: 移除 md-editor-v3 依赖**

```bash
npm uninstall md-editor-v3
```

- [ ] **Step 3: 确认依赖变更**

运行 `cat package.json | grep -E "vditor|md-editor"` 确认：
- `vditor` 出现在 dependencies 中
- `md-editor-v3` 已移除

- [ ] **Step 4: 提交**

```bash
git add package.json package-lock.json
git commit -m "chore(deps): 替换 md-editor-v3 为 vditor"
```

---

### Task 2: 创建 VditorEditor.vue wrapper 组件

**Files:**
- Create: `src/components/notes/VditorEditor.vue`

- [ ] **Step 1: 创建 VditorEditor.vue 组件**

新建 `src/components/notes/VditorEditor.vue`，完整内容如下：

```vue
<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';

interface Props {
  modelValue: string;
  theme?: 'light' | 'dark';
}

const props = withDefaults(defineProps<Props>(), {
  theme: 'light',
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const editorRef = ref<HTMLDivElement>();
let vditor: Vditor | null = null;
let isInternalUpdate = false;

onMounted(() => {
  if (!editorRef.value) return;

  vditor = new Vditor(editorRef.value, {
    mode: 'ir',
    value: props.modelValue,
    theme: props.theme === 'dark' ? 'dark' : 'classic',
    toolbar: [],
    cache: { enable: false },
    input: (value: string) => {
      if (isInternalUpdate) return;
      emit('update:modelValue', value);
    },
  });
});

onUnmounted(() => {
  vditor?.destroy();
  vditor = null;
});

// 外部内容更新 → 同步到编辑器（如人格重写刷新）
watch(() => props.modelValue, (newVal) => {
  if (!vditor || newVal === vditor.getValue()) return;
  isInternalUpdate = true;
  vditor.setValue(newVal);
  isInternalUpdate = false;
});

// 主题切换
watch(() => props.theme, (newTheme) => {
  if (!vditor) return;
  vditor.setTheme(newTheme === 'dark' ? 'dark' : 'classic');
});
</script>

<template>
  <div ref="editorRef" class="vditor-editor-container" />
</template>
```

- [ ] **Step 2: 提交**

```bash
git add src/components/notes/VditorEditor.vue
git commit -m "feat(note): 添加 VditorEditor wrapper 组件（IR 模式）"
```

---

### Task 3: 修改 NoteDetail.vue 集成 VditorEditor

**Files:**
- Modify: `src/components/notes/NoteDetail.vue`

- [ ] **Step 1: 替换 import 语句**

将 NoteDetail.vue 第 54-55 行：

```ts
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
```

替换为：

```ts
import VditorEditor from './VditorEditor.vue';
import { useTheme } from '../../composables/useTheme';
```

- [ ] **Step 2: 获取 theme 状态**

在 `const notesStore = useNotesStore();` 之后（约第 80 行后）添加：

```ts
const { theme } = useTheme();
```

- [ ] **Step 3: 替换模板中的编辑器组件**

将 NoteDetail.vue 第 27-32 行的：

```html
<MdEditor
  v-model="localContent"
  :language="'zh-CN'"
  :style="{ height: 'calc(100vh - 200px)' }"
  @update:model-value="handleContentChange"
/>
```

替换为：

```html
<VditorEditor
  v-model="localContent"
  :theme="theme"
  @update:model-value="handleContentChange"
/>
```

- [ ] **Step 4: 确认无残留 md-editor-v3 引用**

运行 `grep -r "md-editor-v3\|MdEditor" src/` 确认无输出。

- [ ] **Step 5: 提交**

```bash
git add src/components/notes/NoteDetail.vue
git commit -m "feat(note): NoteDetail 集成 VditorEditor 替换 MdEditor"
```

---

### Task 4: 验证构建和基础功能

**Files:** 无文件变更

- [ ] **Step 1: 运行 TypeScript 类型检查**

```bash
npm run build
```

预期：无 TypeScript 错误，构建成功。

- [ ] **Step 2: 启动开发服务器验证**

```bash
npm run dev:all
```

在浏览器中验证：
- 打开一条笔记，编辑器以 IR 模式显示（Typora 风格，无分屏）
- 输入 Markdown 语法（`# 标题`、`**加粗**`、`` `代码` ``），验证即时渲染
- 切换亮色/暗色主题，编辑器跟随切换
- 修改内容后等待 2 秒，验证自动保存

- [ ] **Step 3: 发现问题则修复，无问题则提交最终状态**

```bash
git status
```

如有额外修复，一并提交。
