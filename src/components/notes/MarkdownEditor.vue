<template>
  <div class="markdown-editor">
    <div class="editor-toolbar">
      <button @click="insertBold" title="加粗">B</button>
      <button @click="insertItalic" title="斜体">I</button>
      <button @click="insertHeading" title="标题">H</button>
      <button @click="insertLink" title="链接">🔗</button>
      <button @click="insertCode" title="代码">{ }</button>
      <button @click="insertList" title="列表">•</button>
    </div>

    <div class="editor-content">
      <textarea
        ref="textareaRef"
        v-model="localContent"
        class="editor-textarea"
        placeholder="开始编写..."
        @input="handleInput"
      />
      <div class="editor-preview" v-html="renderedHtml" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js';
import 'highlight.js/styles/github.css';

interface Props {
  modelValue: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const localContent = ref(props.modelValue);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

const md = new MarkdownIt({
  highlight: (str, lang) => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value;
      } catch {}
    }
    return '';
  },
});

const renderedHtml = computed(() => {
  return md.render(localContent.value);
});

watch(() => props.modelValue, (newVal) => {
  if (newVal !== localContent.value) {
    localContent.value = newVal;
  }
});

function handleInput() {
  emit('update:modelValue', localContent.value);
}

function insertAtCursor(before: string, after = '') {
  const textarea = textareaRef.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selectedText = localContent.value.substring(start, end);
  const newText = before + selectedText + after;

  localContent.value =
    localContent.value.substring(0, start) +
    newText +
    localContent.value.substring(end);

  emit('update:modelValue', localContent.value);

  setTimeout(() => {
    textarea.focus();
    textarea.setSelectionRange(start + before.length, start + before.length + selectedText.length);
  }, 0);
}

function insertBold() {
  insertAtCursor('**', '**');
}

function insertItalic() {
  insertAtCursor('*', '*');
}

function insertHeading() {
  insertAtCursor('## ');
}

function insertLink() {
  insertAtCursor('[', '](url)');
}

function insertCode() {
  insertAtCursor('`', '`');
}

function insertList() {
  insertAtCursor('- ');
}
</script>

<style scoped>
.markdown-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
}

.editor-toolbar {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
}

.editor-toolbar button {
  padding: 6px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.editor-toolbar button:hover {
  background: #f3f4f6;
}

.editor-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.editor-textarea,
.editor-preview {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.editor-textarea {
  border: none;
  border-right: 1px solid #e5e7eb;
  resize: none;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 14px;
  line-height: 1.6;
}

.editor-textarea:focus {
  outline: none;
}

.editor-preview {
  background: #fafafa;
  font-size: 15px;
  line-height: 1.7;
}

.editor-preview :deep(h1),
.editor-preview :deep(h2),
.editor-preview :deep(h3) {
  margin-top: 24px;
  margin-bottom: 12px;
  font-weight: 600;
}

.editor-preview :deep(h1) {
  font-size: 28px;
}

.editor-preview :deep(h2) {
  font-size: 24px;
}

.editor-preview :deep(h3) {
  font-size: 20px;
}

.editor-preview :deep(p) {
  margin-bottom: 12px;
}

.editor-preview :deep(code) {
  padding: 2px 6px;
  background: #f3f4f6;
  border-radius: 3px;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 13px;
}

.editor-preview :deep(pre) {
  padding: 12px;
  background: #1f2937;
  border-radius: 6px;
  overflow-x: auto;
  margin-bottom: 12px;
}

.editor-preview :deep(pre code) {
  background: transparent;
  color: #e5e7eb;
  padding: 0;
}

.editor-preview :deep(ul),
.editor-preview :deep(ol) {
  margin-left: 24px;
  margin-bottom: 12px;
}

.editor-preview :deep(li) {
  margin-bottom: 6px;
}
</style>
