<template>
  <div class="note-detail">
    <div class="detail-header">
      <input
        v-model="localTitle"
        type="text"
        class="title-input"
        placeholder="笔记标题"
        @blur="handleSave"
      />
      <div class="detail-actions">
        <button class="btn-save" @click="handleSave" :disabled="saving">
          {{ saving ? '保存中...' : '保存' }}
        </button>
        <button class="btn-ai" disabled title="功能开发中">
          AI 整理
        </button>
        <button class="btn-delete" @click="handleDelete">
          删除
        </button>
      </div>
    </div>

    <div class="detail-meta">
      <span>创建于 {{ formatDate(noteDetail.note.created_at) }}</span>
      <span>更新于 {{ formatDate(noteDetail.note.updated_at) }}</span>
      <span>{{ noteDetail.note.word_count }} 字</span>
    </div>

    <MarkdownEditor
      v-model="localContent"
      @update:model-value="handleContentChange"
    />

    <div v-if="saveStatus" class="save-status" :class="saveStatus">
      {{ saveStatus === 'success' ? '已保存' : '保存失败' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { NoteDetail } from '../../types/index';
import MarkdownEditor from './MarkdownEditor.vue';

interface Props {
  noteDetail: NoteDetail;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [id: number, title: string, content: string];
  delete: [id: number];
}>();

const localTitle = ref(props.noteDetail.note.title);
const localContent = ref(props.noteDetail.content);
const saving = ref(false);
const saveStatus = ref<'success' | 'error' | null>(null);
let saveTimer: number | null = null;

watch(() => props.noteDetail, (newVal) => {
  localTitle.value = newVal.note.title;
  localContent.value = newVal.content;
});

function handleContentChange() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    handleSave();
  }, 2000);
}

async function handleSave() {
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }

  saving.value = true;
  saveStatus.value = null;

  try {
    emit('save', props.noteDetail.note.id, localTitle.value, localContent.value);
    saveStatus.value = 'success';
    setTimeout(() => {
      saveStatus.value = null;
    }, 2000);
  } catch (e) {
    saveStatus.value = 'error';
    console.error('Save failed:', e);
  } finally {
    saving.value = false;
  }
}

function handleDelete() {
  if (confirm('确定要删除这篇笔记吗？')) {
    emit('delete', props.noteDetail.note.id);
  }
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN');
}
</script>

<style scoped>
.note-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
  position: relative;
}

.detail-header {
  padding: 16px;
  border-bottom: 1px solid #e5e7eb;
}

.title-input {
  width: 100%;
  padding: 8px 0;
  border: none;
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 12px;
}

.title-input:focus {
  outline: none;
}

.detail-actions {
  display: flex;
  gap: 8px;
}

.detail-actions button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.btn-save {
  background: #6366f1;
  color: white;
}

.btn-save:hover:not(:disabled) {
  background: #4f46e5;
}

.btn-save:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-ai {
  background: #f3f4f6;
  color: #9ca3af;
  cursor: not-allowed;
}

.btn-delete {
  background: #fee;
  color: #dc2626;
}

.btn-delete:hover {
  background: #fdd;
}

.detail-meta {
  padding: 12px 16px;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #6b7280;
}

.save-status {
  position: absolute;
  bottom: 24px;
  right: 24px;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  animation: fadeIn 0.3s;
}

.save-status.success {
  background: #d1fae5;
  color: #065f46;
}

.save-status.error {
  background: #fee2e2;
  color: #991b1b;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
