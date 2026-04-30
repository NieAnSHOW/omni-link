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

const wrapperRef = ref<HTMLDivElement>();
const editorRef = ref<HTMLDivElement>();
let vditor: Vditor | null = null;
let isInternalUpdate = false;

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
  <div ref="wrapperRef" class="h-full min-h-0 overflow-hidden" style="height: calc(100vh - 200px)">
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
</style>
