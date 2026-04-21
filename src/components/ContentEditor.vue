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
import { ref } from 'vue'

import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const props = defineProps<{
  initialTitle: string | null;
  initialBody: string;
  saving?: boolean;
}>();

const emit = defineEmits<{
  save: [title: string | null, bodyText: string];
  cancel: [];
}>();

const editTitle = ref(props.initialTitle ?? '');
const editBody = ref(props.initialBody);

function handleSave() {
  emit('save', editTitle.value || null, editBody.value);
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
