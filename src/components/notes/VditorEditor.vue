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
