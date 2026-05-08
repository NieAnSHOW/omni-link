<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import { ListTree, X } from 'lucide-vue-next';

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

const wrapperRef = ref<HTMLDivElement>();
const editorRef = ref<HTMLDivElement>();
const outlineVisible = ref(false);
let vditor: Vditor | null = null;
let isInternalUpdate = false;

function toggleOutline() {
  outlineVisible.value = !outlineVisible.value;
  if (!wrapperRef.value) return;
  const outlineEl = wrapperRef.value.querySelector('.vditor-outline') as HTMLElement | null;
  if (outlineEl) {
    outlineEl.style.display = outlineVisible.value ? '' : 'none';
    if (outlineVisible.value) {
      vditor?.outline?.render?.();
    }
  }
}

onMounted(() => {
  if (!editorRef.value || !wrapperRef.value) return;

  vditor = new Vditor(editorRef.value, {
    mode: 'ir',
    value: props.modelValue,
    height: '100%',
    theme: props.theme === 'dark' ? 'dark' : 'classic',
    toolbar: [
      'headings', 'bold', 'italic', 'strike', '|',
      'line', 'quote', 'list', 'ordered-list', 'check', '|',
      'code', 'inline-code', 'link', 'table', '|',
      'undo', 'redo', 'fullscreen', 'edit-mode', '|',
      'more',
    ],
    toolbarConfig: { hide: false },
    outline: { enable: true, position: 'right' },
    cache: { enable: false },
    input: (value: string) => {
      if (isInternalUpdate) return;
      emit('update:modelValue', value);
    },
  });

  // 默认隐藏大纲面板
  requestAnimationFrame(() => {
    const outlineEl = wrapperRef.value?.querySelector('.vditor-outline') as HTMLElement | null;
    if (outlineEl) outlineEl.style.display = 'none';
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
  <div ref="wrapperRef" class="relative h-full min-h-0 overflow-hidden" style="height: calc(100vh - 220px)">
    <button
      class="absolute top-2 right-2 z-10 flex h-7 w-7 items-center justify-center rounded bg-transparent text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
      :title="outlineVisible ? '隐藏大纲' : '显示大纲'"
      @click="toggleOutline"
    >
      <X v-if="outlineVisible" :size="16" />
      <ListTree v-else :size="16" />
    </button>
    <div ref="editorRef" />
  </div>
</template>

<style scoped>
:deep(.vditor) {
  height: 100% !important;
  min-height: 0 !important;
}

:deep(.vditor-content) {
  min-height: 0 !important;
  overflow: auto;
}

:deep(.vditor-ir) {
  min-height: 0 !important;
  overflow: auto;
}

:deep(.vditor-outline) {
  border-left: 1px solid var(--border);
  background: var(--background);
  font-size: 13px;
  overflow: auto;
}

:deep(.vditor-outline__title) {
  display: none;
}

:deep(.vditor-outline a) {
  color: var(--muted-foreground);
  text-decoration: none;
}

:deep(.vditor-outline a:hover) {
  color: var(--foreground);
}
</style>
