<template>
  <div class="flex flex-col gap-3">
    <Input v-model="searchQuery" placeholder="搜索笔记..." />
    <Button class="w-full" @click="$emit('create')">
      + 新建笔记
    </Button>
  </div>

  <div v-if="loading" class="px-4 py-8 text-center text-sm text-muted-foreground">
    加载中...
  </div>
  <div v-else-if="filteredNotes.length === 0" class="px-4 py-8 text-center text-sm text-muted-foreground">
    {{ searchQuery ? '未找到匹配的笔记' : '暂无笔记' }}
  </div>
  <div v-else class="flex flex-col">
    <div v-for="note in filteredNotes" :key="note.id"
      class="cursor-pointer border-b px-4 py-4 transition-colors last:border-b-0 hover:bg-muted/50" :class="{
        'border-l-[3px] border-l-primary bg-primary/5': note.id === selectedId,
      }" @click="$emit('select', note.id)">
      <div class="mb-1.5 truncate text-[15px] font-medium text-foreground">
        {{ note.title || '未命名笔记' }}
      </div>
      <div class="text-xs text-muted-foreground">
        {{ formatTime(note.updated_at) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { Note } from '../../types/index';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

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
