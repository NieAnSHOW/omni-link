<template>
  <div class="notes-list">
    <div class="list-header">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索笔记..."
        class="search-input"
      />
      <button class="btn-create" @click="$emit('create')">
        + 新建笔记
      </button>
    </div>

    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="filteredNotes.length === 0" class="empty">
      {{ searchQuery ? '未找到匹配的笔记' : '暂无笔记' }}
    </div>
    <div v-else class="notes-items">
      <div
        v-for="note in filteredNotes"
        :key="note.id"
        class="note-item"
        :class="{ active: note.id === selectedId }"
        @click="$emit('select', note.id)"
      >
        <div class="note-title">{{ note.title || '未命名笔记' }}</div>
        <div class="note-meta">
          <span class="note-time">{{ formatTime(note.updated_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { Note } from '../../types/index';

interface Props {
  notes: Note[];
  selectedId?: number;
  loading?: boolean;
}

const props = defineProps<Props>();
const searchQuery = ref('');

defineEmits<{
  select: [id: number];
  create: [];
}>();

const filteredNotes = computed(() => {
  if (!searchQuery.value) return props.notes;
  const query = searchQuery.value.toLowerCase();
  return props.notes.filter(note =>
    note.title.toLowerCase().includes(query)
  );
});

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) return '今天';
  if (days === 1) return '昨天';
  if (days < 7) return `${days} 天前`;
  return date.toLocaleDateString('zh-CN');
}
</script>

<style scoped>
.notes-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
}

.list-header {
  padding: 16px;
  border-bottom: 1px solid #e5e7eb;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  font-size: 14px;
  margin-bottom: 12px;
}

.search-input:focus {
  outline: none;
  border-color: #6366f1;
}

.btn-create {
  width: 100%;
  padding: 10px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.btn-create:hover {
  background: #4f46e5;
}

.loading,
.empty {
  padding: 32px 16px;
  text-align: center;
  color: #9ca3af;
  font-size: 14px;
}

.notes-items {
  flex: 1;
  overflow-y: auto;
}

.note-item {
  padding: 16px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
  transition: background 0.2s;
}

.note-item:hover {
  background: #f9fafb;
}

.note-item.active {
  background: #eef2ff;
  border-left: 3px solid #6366f1;
}

.note-title {
  font-size: 15px;
  font-weight: 500;
  color: #1f2937;
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.note-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #9ca3af;
}
</style>
