<template>
  <div class="flex h-full flex-col p-4">
    <input
      v-model="localTitle"
      type="text"
      class="w-full border-none bg-transparent text-2xl font-semibold outline-none placeholder:text-muted-foreground"
      placeholder="笔记标题"
      @blur="handleSave"
    />
    <div class="mt-1 flex items-center gap-4 border-b py-2 text-xs text-muted-foreground">
      <span>创建于 {{ formatDate(noteDetail.note.created_at) }}</span>
      <span>更新于 {{ formatDate(noteDetail.note.updated_at) }}</span>
      <a
        v-if="noteDetail.note.source_url"
        :href="noteDetail.note.source_url"
        target="_blank"
        class="text-primary no-underline hover:underline"
      >
        来源链接
      </a>
      <Badge v-if="saveStatus" :variant="saveStatus === 'success' ? 'default' : 'destructive'" class="ml-auto">
        {{ saveStatus === 'success' ? '已保存' : '保存失败' }}
      </Badge>
    </div>

    <div class="flex-1">
      <VditorEditor
        v-model="localContent"
        :theme="theme"
        @update:model-value="handleContentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { NoteDetail } from '../../types/index';
import VditorEditor from './VditorEditor.vue';
import { useTheme } from '../../composables/useTheme';
import { notesApi } from '../../composables/useApi';
import { useNotesStore } from '../../stores/notes';
import { Badge } from '@/components/ui/badge';

interface Props {
  noteDetail: NoteDetail;
}

const props = defineProps<Props>();

const notesStore = useNotesStore();
const { theme } = useTheme();
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
    await notesApi.updateNote(props.noteDetail.note.id, localTitle.value, localContent.value);

    const updatedAt = new Date().toISOString();

    props.noteDetail.note.title = localTitle.value;
    props.noteDetail.content = localContent.value;
    props.noteDetail.note.updated_at = updatedAt;

    const index = notesStore.notes.findIndex(n => n.id === props.noteDetail.note.id);
    if (index !== -1) {
      notesStore.notes[index].title = localTitle.value;
      notesStore.notes[index].updated_at = updatedAt;
    }

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

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN');
}
</script>
