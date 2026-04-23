# UI 重构实现计划：迁移到 shadcn-vue + TailwindCSS

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 OmniLink 的 UI 系统从纯 scoped CSS 迁移到 shadcn-vue + TailwindCSS，实现统一的设计系统和更好的可维护性。

**Architecture:** 采用一次性迁移策略，使用 shadcn-vue 默认主题。按需引入 shadcn-vue 组件，所有业务组件使用 Tailwind 工具类重写，移除所有 scoped CSS。

**Tech Stack:** Vue 3, TailwindCSS 4.x, shadcn-vue, Radix Vue, class-variance-authority, clsx, tailwind-merge

---

## 文件映射

### 新增文件
- `tailwind.config.js` - TailwindCSS 配置
- `postcss.config.js` - PostCSS 配置
- `components.json` - shadcn-vue 配置
- `src/assets/index.css` - Tailwind 入口文件
- `src/lib/utils.ts` - shadcn-vue 工具函数
- `src/components/ui/*.vue` - shadcn-vue 组件（10 个）

### 修改文件
- `package.json` - 添加依赖
- `src/main.ts` - 导入 Tailwind 样式
- `src/App.vue` - 移除 scoped CSS，改用 Tailwind
- `src/components/*.vue` - 12 个组件重写
- `src/views/*.vue` - 5 个视图重写

---

## Task 1: 安装 TailwindCSS 依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 TailwindCSS 及相关依赖**

```bash
npm install -D tailwindcss@latest postcss@latest autoprefixer@latest
```

Expected output: 依赖安装成功，package.json 更新

- [ ] **Step 2: 初始化 TailwindCSS 配置**

```bash
npx tailwindcss init -p
```

Expected output: 创建 `tailwind.config.js` 和 `postcss.config.js`

- [ ] **Step 3: 验证配置文件已创建**

```bash
ls -la tailwind.config.js postcss.config.js
```

Expected output: 两个文件存在

- [ ] **Step 4: Commit**

```bash
git add package.json package-lock.json tailwind.config.js postcss.config.js
git commit -m "chore: 安装 TailwindCSS 依赖"
```

---

## Task 2: 配置 TailwindCSS

**Files:**
- Modify: `tailwind.config.js`

- [ ] **Step 1: 配置 TailwindCSS content 路径**

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

- [ ] **Step 2: 验证配置语法**

```bash
node -e "import('./tailwind.config.js').then(() => console.log('Config valid'))"
```

Expected output: "Config valid"

- [ ] **Step 3: Commit**

```bash
git add tailwind.config.js
git commit -m "chore: 配置 TailwindCSS content 路径"
```

---

## Task 3: 创建 Tailwind 入口文件

**Files:**
- Create: `src/assets/index.css`

- [ ] **Step 1: 创建 assets 目录**

```bash
mkdir -p src/assets
```

- [ ] **Step 2: 创建 Tailwind 入口文件**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

- [ ] **Step 3: 验证文件创建**

```bash
cat src/assets/index.css
```

Expected output: 显示三行 @tailwind 指令

- [ ] **Step 4: Commit**

```bash
git add src/assets/index.css
git commit -m "chore: 创建 Tailwind 入口文件"
```

---

## Task 4: 在 main.ts 中导入 Tailwind 样式

**Files:**
- Modify: `src/main.ts`

- [ ] **Step 1: 在 main.ts 顶部导入 Tailwind 样式**

在 `import { createApp } from 'vue'` 之后添加：

```typescript
import './assets/index.css'
```

- [ ] **Step 2: 验证导入语句**

```bash
grep "import './assets/index.css'" src/main.ts
```

Expected output: 显示导入语句

- [ ] **Step 3: 启动开发服务器验证 Tailwind 加载**

```bash
npm run dev
```

Expected: 开发服务器启动，无错误

- [ ] **Step 4: 停止开发服务器**

按 Ctrl+C 停止

- [ ] **Step 5: Commit**

```bash
git add src/main.ts
git commit -m "chore: 在 main.ts 中导入 Tailwind 样式"
```

---

## Task 5: 初始化 shadcn-vue

**Files:**
- Create: `components.json`
- Create: `src/lib/utils.ts`

- [ ] **Step 1: 运行 shadcn-vue 初始化命令**

```bash
npx shadcn-vue@latest init
```

交互式选项：
- TypeScript: Yes
- Framework: Vite
- Style: Default
- Base color: Slate
- CSS variables: Yes
- Components directory: src/components/ui
- Utils directory: src/lib
- Tailwind config: tailwind.config.js
- Import alias: @

- [ ] **Step 2: 验证生成的文件**

```bash
ls -la components.json src/lib/utils.ts
```

Expected output: 两个文件存在

- [ ] **Step 3: 检查 tailwind.config.js 是否被更新**

```bash
grep "tailwindcss-animate" tailwind.config.js
```

Expected output: 显示 tailwindcss-animate 插件配置

- [ ] **Step 4: Commit**

```bash
git add components.json src/lib/utils.ts tailwind.config.js package.json package-lock.json
git commit -m "chore: 初始化 shadcn-vue 配置"
```

---

## Task 6: 安装 shadcn-vue 组件 - Button

**Files:**
- Create: `src/components/ui/button.vue`

- [ ] **Step 1: 安装 Button 组件**

```bash
npx shadcn-vue@latest add button
```

Expected output: Button 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/button.vue
```

Expected output: 文件存在

- [ ] **Step 3: 检查依赖是否安装**

```bash
grep "class-variance-authority\|clsx\|tailwind-merge" package.json
```

Expected output: 显示相关依赖

- [ ] **Step 4: Commit**

```bash
git add src/components/ui/button.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Button 组件"
```

---

## Task 7: 安装 shadcn-vue 组件 - Card

**Files:**
- Create: `src/components/ui/card.vue`

- [ ] **Step 1: 安装 Card 组件**

```bash
npx shadcn-vue@latest add card
```

Expected output: Card 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/card.vue
```

Expected output: 文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/card.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Card 组件"
```

---

## Task 8: 安装 shadcn-vue 组件 - Badge

**Files:**
- Create: `src/components/ui/badge.vue`

- [ ] **Step 1: 安装 Badge 组件**

```bash
npx shadcn-vue@latest add badge
```

Expected output: Badge 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/badge.vue
```

Expected output: 文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/badge.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Badge 组件"
```

---

## Task 9: 安装 shadcn-vue 组件 - Input

**Files:**
- Create: `src/components/ui/input.vue`

- [ ] **Step 1: 安装 Input 组件**

```bash
npx shadcn-vue@latest add input
```

Expected output: Input 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/input.vue
```

Expected output: 文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/input.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Input 组件"
```

---

## Task 10: 安装 shadcn-vue 组件 - Dialog

**Files:**
- Create: `src/components/ui/dialog.vue`

- [ ] **Step 1: 安装 Dialog 组件**

```bash
npx shadcn-vue@latest add dialog
```

Expected output: Dialog 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/dialog.vue
```

Expected output: 文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/dialog.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Dialog 组件"
```

---

## Task 11: 安装 shadcn-vue 组件 - Skeleton

**Files:**
- Create: `src/components/ui/skeleton.vue`

- [ ] **Step 1: 安装 Skeleton 组件**

```bash
npx shadcn-vue@latest add skeleton
```

Expected output: Skeleton 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/skeleton.vue
```

Expected output: 文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/skeleton.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Skeleton 组件"
```

---

## Task 12: 安装 shadcn-vue 组件 - Toast

**Files:**
- Create: `src/components/ui/toast.vue`
- Create: `src/components/ui/toaster.vue`

- [ ] **Step 1: 安装 Toast 组件**

```bash
npx shadcn-vue@latest add toast
```

Expected output: Toast 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/toast.vue src/components/ui/toaster.vue
```

Expected output: 两个文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/toast.vue src/components/ui/toaster.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Toast 组件"
```

---

## Task 13: 安装 shadcn-vue 组件 - Textarea

**Files:**
- Create: `src/components/ui/textarea.vue`

- [ ] **Step 1: 安装 Textarea 组件**

```bash
npx shadcn-vue@latest add textarea
```

Expected output: Textarea 组件安装成功

- [ ] **Step 2: 验证组件文件创建**

```bash
ls -la src/components/ui/textarea.vue
```

Expected output: 文件存在

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/textarea.vue package.json package-lock.json
git commit -m "chore: 安装 shadcn-vue Textarea 组件"
```

---

## Task 14: 重写 ScrollToTop 组件

**Files:**
- Modify: `src/components/ScrollToTop.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/ScrollToTop.vue src/components/ScrollToTop.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Button 和 Tailwind**

```vue
<template>
  <Transition
    enter-active-class="transition-opacity duration-300"
    leave-active-class="transition-opacity duration-300"
    enter-from-class="opacity-0"
    leave-to-class="opacity-0"
  >
    <Button
      v-if="visible"
      @click="scrollToTop"
      class="fixed bottom-6 right-6 z-50 h-12 w-12 rounded-full shadow-lg"
      size="icon"
      variant="default"
    >
      ↑
    </Button>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Button } from '@/components/ui/button';

const visible = ref(false);

function handleScroll() {
  visible.value = window.scrollY > 300;
}

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll);
});

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll);
});
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/ScrollToTop.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/ScrollToTop.vue
git commit -m "refactor: 重写 ScrollToTop 组件使用 shadcn-vue"
```

---

## Task 15: 重写 LinkCardSkeleton 组件

**Files:**
- Modify: `src/components/LinkCardSkeleton.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/LinkCardSkeleton.vue src/components/LinkCardSkeleton.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Skeleton**

```vue
<template>
  <div class="rounded-lg border border-slate-200 p-4 space-y-3">
    <div class="flex justify-between items-center">
      <Skeleton class="h-5 w-16" />
      <Skeleton class="h-5 w-12" />
    </div>
    <Skeleton class="h-6 w-3/4" />
    <Skeleton class="h-4 w-full" />
    <div class="flex justify-between items-center">
      <Skeleton class="h-4 w-24" />
      <Skeleton class="h-4 w-4" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton';
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/LinkCardSkeleton.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/LinkCardSkeleton.vue
git commit -m "refactor: 重写 LinkCardSkeleton 组件使用 shadcn-vue"
```

---

## Task 16: 重写 LinkCard 组件

**Files:**
- Modify: `src/components/LinkCard.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/LinkCard.vue src/components/LinkCard.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Card 和 Badge**

```vue
<template>
  <Card 
    class="cursor-pointer transition-shadow hover:shadow-md"
    :class="{ 'opacity-60 pointer-events-none': link.ai_processing_status !== 'idle' }"
    @click="$emit('click')"
  >
    <CardHeader class="pb-3">
      <div class="flex justify-between items-center">
        <Badge variant="secondary" class="text-xs uppercase">
          {{ link.platform || 'web' }}
        </Badge>
        <Badge :variant="statusVariant" class="text-xs">
          {{ statusText }}
        </Badge>
      </div>
    </CardHeader>
    <CardContent class="space-y-2">
      <CardTitle class="text-base line-clamp-3">
        {{ link.title || link.url }}
      </CardTitle>
      <p class="text-sm text-muted-foreground truncate">
        {{ link.url }}
      </p>
    </CardContent>
    <CardFooter class="flex justify-between items-center pt-3">
      <span class="text-xs text-muted-foreground">
        {{ formatDate(link.created_at) }}
      </span>
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        @click.stop="$emit('delete', link.id)"
        title="删除"
      >
        ✕
      </Button>
    </CardFooter>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Link } from '../types/index';
import { Card, CardHeader, CardContent, CardTitle, CardFooter } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

const props = defineProps<{ link: Link }>();
defineEmits<{ click: []; delete: [id: number] }>();

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

const statusVariant = computed(() => {
  if (props.link.ai_processing_status !== 'idle') return 'default';
  const variantMap: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
    parsed: 'default',
    pending: 'secondary',
    parsing: 'secondary',
    failed: 'destructive'
  };
  return variantMap[props.link.status] || 'outline';
});

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', { 
    month: 'short', 
    day: 'numeric', 
    hour: '2-digit', 
    minute: '2-digit' 
  });
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/LinkCard.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/LinkCard.vue
git commit -m "refactor: 重写 LinkCard 组件使用 shadcn-vue"
```

---

## Task 17: 重写 TagInput 组件

**Files:**
- Modify: `src/components/TagInput.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/TagInput.vue src/components/TagInput.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Input 和 Badge**

```vue
<template>
  <div class="space-y-2">
    <div class="flex gap-2">
      <Input
        v-model="inputValue"
        @keydown.enter.prevent="addTag"
        @keydown.comma.prevent="addTag"
        placeholder="输入标签，按回车或逗号添加"
        class="flex-1"
      />
      <Button @click="addTag" variant="outline">
        添加
      </Button>
    </div>
    <div v-if="tags.length > 0" class="flex flex-wrap gap-2">
      <Badge
        v-for="(tag, index) in tags"
        :key="index"
        variant="secondary"
        class="px-3 py-1"
      >
        {{ tag }}
        <button
          @click="removeTag(index)"
          class="ml-2 hover:text-destructive"
        >
          ×
        </button>
      </Badge>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

const props = defineProps<{ modelValue: string[] }>();
const emit = defineEmits<{ 'update:modelValue': [value: string[]] }>();

const inputValue = ref('');
const tags = ref<string[]>(props.modelValue || []);

function addTag() {
  const tag = inputValue.value.trim();
  if (tag && !tags.value.includes(tag)) {
    tags.value.push(tag);
    emit('update:modelValue', tags.value);
    inputValue.value = '';
  }
}

function removeTag(index: number) {
  tags.value.splice(index, 1);
  emit('update:modelValue', tags.value);
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/TagInput.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/TagInput.vue
git commit -m "refactor: 重写 TagInput 组件使用 shadcn-vue"
```

---

## Task 18: 重写 AddLinkDialog 组件

**Files:**
- Modify: `src/components/AddLinkDialog.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/AddLinkDialog.vue src/components/AddLinkDialog.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Dialog**

```vue
<template>
  <Dialog :open="visible" @update:open="handleClose">
    <DialogContent class="sm:max-w-[500px]">
      <DialogHeader>
        <DialogTitle>添加知识</DialogTitle>
        <DialogDescription>
          输入一个或多个链接（每行一个）
        </DialogDescription>
      </DialogHeader>
      <div class="space-y-4 py-4">
        <Textarea
          v-model="urls"
          placeholder="https://example.com&#10;https://another.com"
          class="min-h-[120px]"
        />
      </div>
      <DialogFooter>
        <Button variant="outline" @click="handleClose">
          取消
        </Button>
        <Button @click="handleSubmit" :disabled="!urls.trim()">
          添加
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  close: [];
  submit: [urls: string[]];
}>();

const urls = ref('');

watch(() => props.visible, (newVal) => {
  if (!newVal) {
    urls.value = '';
  }
});

function handleClose() {
  emit('close');
}

function handleSubmit() {
  const urlList = urls.value
    .split('\n')
    .map(u => u.trim())
    .filter(u => u.length > 0);
  
  if (urlList.length > 0) {
    emit('submit', urlList);
    urls.value = '';
  }
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/AddLinkDialog.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/AddLinkDialog.vue
git commit -m "refactor: 重写 AddLinkDialog 组件使用 shadcn-vue"
```

---

## Task 19: 重写 ContentEditor 组件

**Files:**
- Modify: `src/components/ContentEditor.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/ContentEditor.vue src/components/ContentEditor.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Textarea 和 Button**

```vue
<template>
  <div class="space-y-4">
    <Textarea
      v-model="localContent"
      placeholder="编辑内容..."
      class="min-h-[300px] font-mono text-sm"
    />
    <div class="flex justify-end gap-2">
      <Button variant="outline" @click="handleCancel">
        取消
      </Button>
      <Button @click="handleSave">
        保存
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button';

const props = defineProps<{ content: string }>();
const emit = defineEmits<{
  save: [content: string];
  cancel: [];
}>();

const localContent = ref(props.content);

watch(() => props.content, (newVal) => {
  localContent.value = newVal;
});

function handleSave() {
  emit('save', localContent.value);
}

function handleCancel() {
  localContent.value = props.content;
  emit('cancel');
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/ContentEditor.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/ContentEditor.vue
git commit -m "refactor: 重写 ContentEditor 组件使用 shadcn-vue"
```

---

## Task 20: 重写 StickyHeader 组件

**Files:**
- Modify: `src/components/StickyHeader.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/StickyHeader.vue src/components/StickyHeader.vue.bak
```

- [ ] **Step 2: 重写组件使用 Tailwind 粘性定位**

```vue
<template>
  <div class="sticky top-0 z-40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b">
    <div class="container flex h-14 items-center">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
// 纯展示组件，无逻辑
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/StickyHeader.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/StickyHeader.vue
git commit -m "refactor: 重写 StickyHeader 组件使用 Tailwind"
```

---

## Task 21: 重写 AppLayout 组件

**Files:**
- Modify: `src/components/AppLayout.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/AppLayout.vue src/components/AppLayout.vue.bak
```

- [ ] **Step 2: 重写组件使用 Tailwind 布局类**

```vue
<template>
  <div class="flex h-screen bg-background">
    <!-- 侧边栏 -->
    <aside class="w-64 border-r bg-muted/40">
      <div class="flex h-14 items-center border-b px-4">
        <h1 class="text-lg font-semibold">Omni-Link</h1>
      </div>
      <nav class="space-y-1 p-4">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
          active-class="bg-accent text-accent-foreground"
        >
          <span>{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </router-link>
      </nav>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 overflow-auto">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
const navItems = [
  { path: '/links', label: '知识库', icon: '📚' },
  { path: '/tags', label: '标签', icon: '🏷️' },
  { path: '/notes', label: '笔记', icon: '📝' },
  { path: '/settings', label: '设置', icon: '⚙️' },
];
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/AppLayout.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/AppLayout.vue
git commit -m "refactor: 重写 AppLayout 组件使用 Tailwind"
```

---

## Task 22: 重写 NotesList 组件

**Files:**
- Modify: `src/components/notes/NotesList.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/notes/NotesList.vue src/components/notes/NotesList.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Card**

```vue
<template>
  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    <Card
      v-for="note in notes"
      :key="note.id"
      class="cursor-pointer transition-shadow hover:shadow-md"
      @click="$emit('select', note.id)"
    >
      <CardHeader>
        <CardTitle class="line-clamp-2">{{ note.title }}</CardTitle>
      </CardHeader>
      <CardContent>
        <p class="text-sm text-muted-foreground line-clamp-3">
          {{ note.content }}
        </p>
      </CardContent>
      <CardFooter class="text-xs text-muted-foreground">
        {{ formatDate(note.updated_at) }}
      </CardFooter>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '@/components/ui/card';

interface Note {
  id: number;
  title: string;
  content: string;
  updated_at: string;
}

defineProps<{ notes: Note[] }>();
defineEmits<{ select: [id: number] }>();

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/notes/NotesList.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/notes/NotesList.vue
git commit -m "refactor: 重写 NotesList 组件使用 shadcn-vue"
```

---

## Task 23: 重写 NoteDetail 组件

**Files:**
- Modify: `src/components/notes/NoteDetail.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/notes/NoteDetail.vue src/components/notes/NoteDetail.vue.bak
```

- [ ] **Step 2: 重写组件使用 shadcn-vue Card 和 Button**

```vue
<template>
  <Card>
    <CardHeader>
      <div class="flex items-center justify-between">
        <CardTitle>{{ note.title }}</CardTitle>
        <div class="flex gap-2">
          <Button variant="outline" size="sm" @click="$emit('edit')">
            编辑
          </Button>
          <Button variant="destructive" size="sm" @click="$emit('delete')">
            删除
          </Button>
        </div>
      </div>
    </CardHeader>
    <CardContent class="prose prose-sm max-w-none">
      <div v-html="renderedContent"></div>
    </CardContent>
    <CardFooter class="text-sm text-muted-foreground">
      最后更新：{{ formatDate(note.updated_at) }}
    </CardFooter>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

interface Note {
  id: number;
  title: string;
  content: string;
  updated_at: string;
}

const props = defineProps<{ note: Note }>();
defineEmits<{ edit: []; delete: [] }>();

const renderedContent = computed(() => {
  return marked(props.note.content);
});

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/notes/NoteDetail.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/notes/NoteDetail.vue
git commit -m "refactor: 重写 NoteDetail 组件使用 shadcn-vue"
```

---

## Task 24: 重写 MarkdownEditor 组件

**Files:**
- Modify: `src/components/notes/MarkdownEditor.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/components/notes/MarkdownEditor.vue src/components/notes/MarkdownEditor.vue.bak
```

- [ ] **Step 2: 重写组件外层使用 Tailwind**

```vue
<template>
  <div class="space-y-4">
    <Input
      v-model="localTitle"
      placeholder="笔记标题"
      class="text-lg font-semibold"
    />
    <div class="border rounded-lg overflow-hidden">
      <MdEditor
        v-model="localContent"
        language="zh-CN"
        :toolbars="toolbars"
        :preview="true"
        class="min-h-[400px]"
      />
    </div>
    <div class="flex justify-end gap-2">
      <Button variant="outline" @click="handleCancel">
        取消
      </Button>
      <Button @click="handleSave">
        保存
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import MdEditor from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

const props = defineProps<{
  title: string;
  content: string;
}>();

const emit = defineEmits<{
  save: [data: { title: string; content: string }];
  cancel: [];
}>();

const localTitle = ref(props.title);
const localContent = ref(props.content);

const toolbars = [
  'bold',
  'italic',
  'strikeThrough',
  '-',
  'title',
  'sub',
  'sup',
  'quote',
  'unorderedList',
  'orderedList',
  '-',
  'codeRow',
  'code',
  'link',
  'image',
  'table',
  '-',
  'revoke',
  'next',
  'save',
];

watch(() => props.title, (newVal) => {
  localTitle.value = newVal;
});

watch(() => props.content, (newVal) => {
  localContent.value = newVal;
});

function handleSave() {
  emit('save', {
    title: localTitle.value,
    content: localContent.value,
  });
}

function handleCancel() {
  localTitle.value = props.title;
  localContent.value = props.content;
  emit('cancel');
}
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/components/notes/MarkdownEditor.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/components/notes/MarkdownEditor.vue
git commit -m "refactor: 重写 MarkdownEditor 组件外层使用 Tailwind"
```

---

## Task 25: 重写 LinksView 视图

**Files:**
- Modify: `src/views/LinksView.vue`

- [ ] **Step 1: 备份原视图内容**

```bash
cp src/views/LinksView.vue src/views/LinksView.vue.bak
```

- [ ] **Step 2: 重写视图使用 Tailwind 样式**

```vue
<template>
  <div class="container py-6 space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-3xl font-bold tracking-tight">我的知识库</h2>
      <Button @click="showDialog = true">
        <span class="mr-2">+</span>
        添加知识
      </Button>
    </div>

    <div class="flex gap-2">
      <Button
        v-for="f in filters"
        :key="f.value"
        :variant="activeFilter === f.value ? 'default' : 'outline'"
        size="sm"
        @click="setFilter(f.value)"
      >
        {{ f.label }}
      </Button>
    </div>

    <div v-if="loading" class="columns-1 md:columns-2 lg:columns-3 gap-4">
      <div v-for="i in 6" :key="i" class="break-inside-avoid mb-4">
        <LinkCardSkeleton />
      </div>
    </div>

    <div v-else-if="links.length === 0" class="flex items-center justify-center h-64">
      <p class="text-muted-foreground">还没有知识，点击上方按钮添加</p>
    </div>

    <div v-else class="columns-1 md:columns-2 lg:columns-3 gap-4">
      <div v-for="link in links" :key="link.id" class="break-inside-avoid mb-4">
        <LinkCard
          :link="link"
          @click="goToDetail(link.id)"
          @delete="handleDelete"
        />
      </div>
    </div>

    <AddLinkDialog
      :visible="showDialog"
      @close="showDialog = false"
      @submit="handleAddLinks"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useLinksStore } from '../stores/links';
import { useApi } from '../composables/useApi';
import { listen } from '@tauri-apps/api/event';
import { useToast } from '../composables/useToast';
import { Button } from '@/components/ui/button';
import LinkCard from '../components/LinkCard.vue';
import LinkCardSkeleton from '../components/LinkCardSkeleton.vue';
import AddLinkDialog from '../components/AddLinkDialog.vue';

const router = useRouter();
const store = useLinksStore();
const { links, loading } = storeToRefs(store);
const api = useApi();
const toast = useToast();
const showDialog = ref(false);
const activeFilter = ref<string>('');

const filters = [
  { label: '全部', value: '' },
  { label: '已解析', value: 'parsed' },
  { label: '待解析', value: 'pending' },
  { label: '失败', value: 'failed' },
];

function setFilter(value: string) {
  activeFilter.value = value;
}

function goToDetail(id: number) {
  router.push(`/links/${id}`);
}

async function handleDelete(id: number) {
  try {
    await api.deleteLink(id);
    await store.fetchLinks();
    toast.success('删除成功');
  } catch (error) {
    toast.error('删除失败');
  }
}

async function handleAddLinks(urls: string[]) {
  try {
    await api.addLinks(urls);
    showDialog.value = false;
    await store.fetchLinks();
    toast.success('添加成功');
  } catch (error) {
    toast.error('添加失败');
  }
}

let unlisten: (() => void) | null = null;

onMounted(async () => {
  await store.fetchLinks();
  unlisten = await listen('link-parsed', async () => {
    await store.fetchLinks();
  });
});

onUnmounted(() => {
  if (unlisten) unlisten();
});
</script>
```

- [ ] **Step 3: 验证视图语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/views/LinksView.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/views/LinksView.vue
git commit -m "refactor: 重写 LinksView 视图使用 Tailwind"
```

---

## Task 26: 重写 ContentView 视图

**Files:**
- Modify: `src/views/ContentView.vue`

- [ ] **Step 1: 备份原视图内容**

```bash
cp src/views/ContentView.vue src/views/ContentView.vue.bak
```

- [ ] **Step 2: 重写视图使用 Tailwind 样式**

```vue
<template>
  <div class="container py-6 space-y-6">
    <div class="flex items-center justify-between">
      <Button variant="ghost" @click="goBack">
        ← 返回
      </Button>
      <div class="flex gap-2">
        <Button variant="outline" @click="handleEdit">
          编辑
        </Button>
        <Button variant="destructive" @click="handleDelete">
          删除
        </Button>
      </div>
    </div>

    <Card v-if="link">
      <CardHeader>
        <div class="flex items-center gap-2 mb-2">
          <Badge variant="secondary">{{ link.platform || 'web' }}</Badge>
          <Badge :variant="statusVariant">{{ statusText }}</Badge>
        </div>
        <CardTitle class="text-2xl">{{ link.title || link.url }}</CardTitle>
        <a :href="link.url" target="_blank" class="text-sm text-primary hover:underline">
          {{ link.url }}
        </a>
      </CardHeader>
      <CardContent v-if="content" class="prose prose-sm max-w-none">
        <div v-html="content.content"></div>
      </CardContent>
    </Card>

    <div v-else class="flex items-center justify-center h-64">
      <p class="text-muted-foreground">加载中...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useApi } from '../composables/useApi';
import { useToast } from '../composables/useToast';
import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { Link, Content } from '../types/index';

const route = useRoute();
const router = useRouter();
const api = useApi();
const toast = useToast();

const link = ref<Link | null>(null);
const content = ref<Content | null>(null);

const statusText = computed(() => {
  if (!link.value) return '';
  const map: Record<string, string> = {
    pending: '待解析',
    parsing: '解析中',
    parsed: '已解析',
    failed: '失败'
  };
  return map[link.value.status] || link.value.status;
});

const statusVariant = computed(() => {
  if (!link.value) return 'outline';
  const variantMap: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
    parsed: 'default',
    pending: 'secondary',
    parsing: 'secondary',
    failed: 'destructive'
  };
  return variantMap[link.value.status] || 'outline';
});

function goBack() {
  router.push('/links');
}

function handleEdit() {
  // TODO: 实现编辑功能
  toast.info('编辑功能开发中');
}

async function handleDelete() {
  if (!link.value) return;
  try {
    await api.deleteLink(link.value.id);
    toast.success('删除成功');
    router.push('/links');
  } catch (error) {
    toast.error('删除失败');
  }
}

onMounted(async () => {
  const id = Number(route.params.id);
  try {
    link.value = await api.getLink(id);
    content.value = await api.getContent(id);
  } catch (error) {
    toast.error('加载失败');
    router.push('/links');
  }
});
</script>
```

- [ ] **Step 3: 验证视图语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/views/ContentView.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/views/ContentView.vue
git commit -m "refactor: 重写 ContentView 视图使用 Tailwind"
```

---

## Task 27: 重写 TagsView 视图

**Files:**
- Modify: `src/views/TagsView.vue`

- [ ] **Step 1: 备份原视图内容**

```bash
cp src/views/TagsView.vue src/views/TagsView.vue.bak
```

- [ ] **Step 2: 重写视图使用 Tailwind 样式**

```vue
<template>
  <div class="container py-6 space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-3xl font-bold tracking-tight">标签管理</h2>
    </div>

    <div v-if="loading" class="flex items-center justify-center h-64">
      <p class="text-muted-foreground">加载中...</p>
    </div>

    <div v-else-if="tags.length === 0" class="flex items-center justify-center h-64">
      <p class="text-muted-foreground">暂无标签</p>
    </div>

    <div v-else class="flex flex-wrap gap-3">
      <Badge
        v-for="tag in tags"
        :key="tag.id"
        variant="secondary"
        class="px-4 py-2 text-base cursor-pointer hover:bg-secondary/80"
        @click="handleTagClick(tag.id)"
      >
        {{ tag.name }}
        <span class="ml-2 text-xs text-muted-foreground">
          ({{ tag.count }})
        </span>
      </Badge>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useTagsStore } from '../stores/tags';
import { Badge } from '@/components/ui/badge';

const router = useRouter();
const store = useTagsStore();
const { tags, loading } = storeToRefs(store);

function handleTagClick(tagId: number) {
  // TODO: 实现标签筛选功能
  console.log('Tag clicked:', tagId);
}

onMounted(async () => {
  await store.fetchTags();
});
</script>
```

- [ ] **Step 3: 验证视图语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/views/TagsView.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/views/TagsView.vue
git commit -m "refactor: 重写 TagsView 视图使用 Tailwind"
```

---

## Task 28: 重写 SettingsView 视图

**Files:**
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: 备份原视图内容**

```bash
cp src/views/SettingsView.vue src/views/SettingsView.vue.bak
```

- [ ] **Step 2: 重写视图使用 Tailwind 样式**

```vue
<template>
  <div class="container py-6 space-y-6">
    <h2 class="text-3xl font-bold tracking-tight">设置</h2>

    <Card>
      <CardHeader>
        <CardTitle>AI 配置</CardTitle>
        <CardDescription>配置 AI 服务提供商和 API 密钥</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <label class="text-sm font-medium">AI 提供商</label>
          <Input
            v-model="settings.ai_provider"
            placeholder="openai / ollama"
          />
        </div>
        <div class="space-y-2">
          <label class="text-sm font-medium">API 密钥</label>
          <Input
            v-model="settings.api_key"
            type="password"
            placeholder="sk-..."
          />
        </div>
        <div class="space-y-2">
          <label class="text-sm font-medium">模型名称</label>
          <Input
            v-model="settings.model_name"
            placeholder="gpt-4"
          />
        </div>
      </CardContent>
      <CardFooter>
        <Button @click="handleSave">
          保存设置
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useApi } from '../composables/useApi';
import { useToast } from '../composables/useToast';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

const api = useApi();
const toast = useToast();

const settings = ref({
  ai_provider: '',
  api_key: '',
  model_name: '',
});

async function handleSave() {
  try {
    await api.updateSettings(settings.value);
    toast.success('保存成功');
  } catch (error) {
    toast.error('保存失败');
  }
}

onMounted(async () => {
  try {
    const data = await api.getSettings();
    settings.value = data;
  } catch (error) {
    toast.error('加载设置失败');
  }
});
</script>
```

- [ ] **Step 3: 验证视图语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/views/SettingsView.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "refactor: 重写 SettingsView 视图使用 Tailwind"
```

---

## Task 29: 重写 NotesView 视图

**Files:**
- Modify: `src/views/NotesView.vue`

- [ ] **Step 1: 备份原视图内容**

```bash
cp src/views/NotesView.vue src/views/NotesView.vue.bak
```

- [ ] **Step 2: 重写视图使用 Tailwind 样式**

```vue
<template>
  <div class="container py-6 space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-3xl font-bold tracking-tight">笔记</h2>
      <Button @click="handleCreate">
        <span class="mr-2">+</span>
        新建笔记
      </Button>
    </div>

    <div v-if="loading" class="flex items-center justify-center h-64">
      <p class="text-muted-foreground">加载中...</p>
    </div>

    <div v-else-if="notes.length === 0" class="flex items-center justify-center h-64">
      <p class="text-muted-foreground">暂无笔记，点击上方按钮创建</p>
    </div>

    <NotesList
      v-else
      :notes="notes"
      @select="handleSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useApi } from '../composables/useApi';
import { useToast } from '../composables/useToast';
import { Button } from '@/components/ui/button';
import NotesList from '../components/notes/NotesList.vue';

const router = useRouter();
const api = useApi();
const toast = useToast();

const notes = ref<any[]>([]);
const loading = ref(false);

function handleCreate() {
  router.push('/notes/new');
}

function handleSelect(id: number) {
  router.push(`/notes/${id}`);
}

onMounted(async () => {
  loading.value = true;
  try {
    notes.value = await api.getNotes();
  } catch (error) {
    toast.error('加载笔记失败');
  } finally {
    loading.value = false;
  }
});
</script>
```

- [ ] **Step 3: 验证视图语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/views/NotesView.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/views/NotesView.vue
git commit -m "refactor: 重写 NotesView 视图使用 Tailwind"
```

---

## Task 30: 重写 App.vue 根组件

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 备份原组件内容**

```bash
cp src/App.vue src/App.vue.bak
```

- [ ] **Step 2: 重写组件使用 Tailwind 样式**

```vue
<template>
  <div v-if="showSplash" class="fixed inset-0 flex items-center justify-center bg-background transition-opacity duration-300">
    <div class="text-center space-y-4">
      <img src="/logo-no-text.png" alt="OmniLink" class="w-24 h-24 mx-auto" />
      <h1 class="text-3xl font-bold">Omni-Link</h1>
      <p class="text-muted-foreground">链接新思维</p>
    </div>
  </div>
  <template v-else>
    <AppLayout />
    <ScrollToTop />
  </template>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppLayout from './components/AppLayout.vue';
import ScrollToTop from './components/ScrollToTop.vue';

const SPLASH_DURATION = 1200;
const FADE_DURATION = 300;

const showSplash = ref(true);

if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

function minDelay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

onMounted(async () => {
  await minDelay(SPLASH_DURATION);
  // Trigger fade-out
  const splash = document.querySelector('.fixed.inset-0');
  if (splash) (splash as HTMLElement).style.opacity = '0';
  await minDelay(FADE_DURATION);
  showSplash.value = false;
});
</script>
```

- [ ] **Step 3: 验证组件语法**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 4: 删除备份文件**

```bash
rm src/App.vue.bak
```

- [ ] **Step 5: Commit**

```bash
git add src/App.vue
git commit -m "refactor: 重写 App.vue 根组件使用 Tailwind"
```

---

## Task 31: 替换 Toast 组件实现

**Files:**
- Modify: `src/components/Toast.vue`
- Modify: `src/composables/useToast.ts`

- [ ] **Step 1: 备份原文件**

```bash
cp src/components/Toast.vue src/components/Toast.vue.bak
cp src/composables/useToast.ts src/composables/useToast.ts.bak
```

- [ ] **Step 2: 使用 shadcn-vue Toast 替换自定义实现**

更新 `src/composables/useToast.ts`:

```typescript
import { useToast as useShadcnToast } from '@/components/ui/toast';

export function useToast() {
  const { toast } = useShadcnToast();

  return {
    success: (message: string) => {
      toast({
        title: '成功',
        description: message,
      });
    },
    error: (message: string) => {
      toast({
        title: '错误',
        description: message,
        variant: 'destructive',
      });
    },
    info: (message: string) => {
      toast({
        title: '提示',
        description: message,
      });
    },
  };
}
```

- [ ] **Step 3: 删除旧的 Toast 组件**

```bash
rm src/components/Toast.vue
```

- [ ] **Step 4: 在 App.vue 中添加 Toaster 组件**

在 `src/App.vue` 的 template 中添加：

```vue
<template>
  <!-- 现有内容 -->
  <Toaster />
</template>

<script setup lang="ts">
// 现有导入
import { Toaster } from '@/components/ui/toast';
</script>
```

- [ ] **Step 5: 验证 Toast 功能**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 6: 删除备份文件**

```bash
rm src/components/Toast.vue.bak src/composables/useToast.ts.bak
```

- [ ] **Step 7: Commit**

```bash
git add src/components/Toast.vue src/composables/useToast.ts src/App.vue
git commit -m "refactor: 使用 shadcn-vue Toast 替换自定义实现"
```

---

## Task 32: 验证所有页面功能

**Files:**
- N/A (测试任务)

- [ ] **Step 1: 启动开发服务器**

```bash
npm run dev:all
```

Expected: 开发服务器启动成功，无错误

- [ ] **Step 2: 测试链接列表页面**

访问 `/links` 页面，验证：
- 页面正常渲染
- 添加链接对话框正常打开和关闭
- 链接卡片正常显示
- 筛选按钮正常工作
- 点击卡片跳转到详情页

- [ ] **Step 3: 测试链接详情页面**

访问 `/links/:id` 页面，验证：
- 页面正常渲染
- 返回按钮正常工作
- 编辑和删除按钮正常显示

- [ ] **Step 4: 测试标签页面**

访问 `/tags` 页面，验证：
- 页面正常渲染
- 标签列表正常显示

- [ ] **Step 5: 测试设置页面**

访问 `/settings` 页面，验证：
- 页面正常渲染
- 表单输入正常工作
- 保存按钮正常工作

- [ ] **Step 6: 测试笔记页面**

访问 `/notes` 页面，验证：
- 页面正常渲染
- 笔记列表正常显示
- 新建按钮正常工作

- [ ] **Step 7: 测试响应式布局**

调整浏览器窗口大小，验证：
- 瀑布流布局在不同屏幕尺寸下正常
- 侧边栏在小屏幕下正常显示
- 所有按钮和输入框在小屏幕下可用

- [ ] **Step 8: 检查控制台错误**

打开浏览器开发者工具，验证：
- 无 JavaScript 错误
- 无 CSS 警告
- 无网络请求失败

- [ ] **Step 9: 停止开发服务器**

按 Ctrl+C 停止

---

## Task 33: 清理所有备份文件

**Files:**
- N/A (清理任务)

- [ ] **Step 1: 查找所有备份文件**

```bash
find src -name "*.bak" -type f
```

Expected: 列出所有 .bak 文件（如果有遗漏）

- [ ] **Step 2: 删除所有备份文件**

```bash
find src -name "*.bak" -type f -delete
```

Expected: 所有备份文件被删除

- [ ] **Step 3: 验证备份文件已删除**

```bash
find src -name "*.bak" -type f
```

Expected: 无输出

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: 清理备份文件"
```

---

## Task 34: 验证打包构建

**Files:**
- N/A (构建任务)

- [ ] **Step 1: 运行前端构建**

```bash
npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 2: 检查构建产物大小**

```bash
du -sh dist/
```

Expected: 显示构建产物大小

- [ ] **Step 3: 运行 Tauri 构建**

```bash
npm run tauri build
```

Expected: 构建成功，生成应用程序

- [ ] **Step 4: 验证应用程序启动**

打开构建生成的应用程序，验证：
- 应用正常启动
- Splash screen 正常显示
- 所有页面正常工作

---

## Task 35: 更新 .gitignore

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: 添加 .superpowers 到 .gitignore**

在 `.gitignore` 文件末尾添加：

```
# Superpowers brainstorming sessions
.superpowers/
```

- [ ] **Step 2: 验证 .gitignore 更新**

```bash
grep ".superpowers" .gitignore
```

Expected: 显示添加的行

- [ ] **Step 3: Commit**

```bash
git add .gitignore
git commit -m "chore: 添加 .superpowers 到 .gitignore"
```

---

## Task 36: 创建迁移完成标记

**Files:**
- Create: `docs/UI_MIGRATION_COMPLETE.md`

- [ ] **Step 1: 创建迁移完成文档**

```markdown
# UI 迁移完成

**日期：** 2026-04-23  
**状态：** ✅ 完成

## 迁移内容

- ✅ TailwindCSS 4.x 安装和配置
- ✅ shadcn-vue 初始化和组件安装
- ✅ 12 个业务组件重写
- ✅ 5 个视图页面重写
- ✅ App.vue 根组件重写
- ✅ Toast 系统替换

## 技术栈

- TailwindCSS 4.x
- shadcn-vue
- Radix Vue
- class-variance-authority
- clsx + tailwind-merge

## 验收标准

- ✅ 所有页面功能正常运行
- ✅ 所有交互逻辑与原版一致
- ✅ 响应式布局在不同屏幕尺寸下正常
- ✅ 无 console 错误或警告
- ✅ 代码中无残留的 scoped CSS
- ✅ 打包构建成功
- ✅ Tauri 应用正常启动和运行

## 后续优化

- 考虑添加 dark mode 支持
- 优化 Tailwind 配置以减小打包体积
- 添加更多 shadcn-vue 组件（如需要）
```

- [ ] **Step 2: Commit**

```bash
git add docs/UI_MIGRATION_COMPLETE.md
git commit -m "docs: 添加 UI 迁移完成标记"
```

---

## Task 37: 最终验证和总结

**Files:**
- N/A (验证任务)

- [ ] **Step 1: 检查 Git 状态**

```bash
git status
```

Expected: 工作目录干净，无未提交的更改

- [ ] **Step 2: 查看提交历史**

```bash
git log --oneline -20
```

Expected: 显示所有迁移相关的提交

- [ ] **Step 3: 统计代码变更**

```bash
git diff --stat HEAD~37 HEAD
```

Expected: 显示所有文件的变更统计

- [ ] **Step 4: 验证所有 scoped CSS 已移除**

```bash
grep -r "<style scoped>" src/
```

Expected: 无输出（所有 scoped CSS 已移除）

- [ ] **Step 5: 验证 shadcn-vue 组件已安装**

```bash
ls -la src/components/ui/
```

Expected: 显示所有 shadcn-vue 组件文件

- [ ] **Step 6: 最终构建测试**

```bash
npm run build && npm run tauri build
```

Expected: 构建成功，无错误

---

## 完成

所有任务完成后，UI 重构迁移工作完成。项目已从纯 scoped CSS 成功迁移到 shadcn-vue + TailwindCSS。

**关键成果：**
- 统一的设计系统
- 更好的可维护性
- 现代化的 UI 工具链
- 按需引入的组件库

**下一步：**
- 在实际使用中测试所有功能
- 根据用户反馈进行微调
- 考虑添加 dark mode 等增强功能

