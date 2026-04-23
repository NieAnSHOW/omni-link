<template>
  <Card class="flex h-full flex-col">
    <CardHeader class="pb-4">
      <input
        v-model="localTitle"
        type="text"
        class="w-full border-none bg-transparent text-2xl font-semibold outline-none placeholder:text-muted-foreground"
        placeholder="笔记标题"
        @blur="handleSave"
      />
      <div class="mt-3 flex items-center gap-2">
        <Button :disabled="saving" @click="handleSave">
          保存
        </Button>
        <Button variant="secondary" disabled title="功能开发中">
          人格撰写
        </Button>
        <Button variant="destructive" @click="handleDelete">
          删除
        </Button>
      </div>
    </CardHeader>

    <CardContent class="flex flex-col gap-0 p-0">
      <div class="flex items-center gap-4 border-b px-6 py-2 text-sm text-muted-foreground">
        <span>创建于 {{ formatDate(noteDetail.note.created_at) }}</span>
        <span>更新于 {{ formatDate(noteDetail.note.updated_at) }}</span>
        <Badge
          v-if="saveStatus"
          :variant="saveStatus === 'success' ? 'default' : 'destructive'"
          class="ml-auto"
        >
          {{ saveStatus === 'success' ? '已保存' : '保存失败' }}
        </Badge>
      </div>

      <div class="flex-1">
        <MdEditor
          v-model="localContent"
          :language="'zh-CN'"
          :style="{ height: 'calc(100vh - 260px)' }"
          @update:model-value="handleContentChange"
        />
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { NoteDetail } from '../../types/index';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import { notesApi } from '../../composables/useApi';
import { useNotesStore } from '../../stores/notes';
import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

interface Props {
  noteDetail: NoteDetail;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [id: number, title: string, content: string];
  delete: [id: number];
}>();

const notesStore = useNotesStore();
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
    // 直接调用 API 保存，不触发 store 更新列表
    await notesApi.updateNote(props.noteDetail.note.id, localTitle.value, localContent.value);

    const updatedAt = new Date().toISOString();

    // 只更新当前笔记的本地状态
    props.noteDetail.note.title = localTitle.value;
    props.noteDetail.content = localContent.value;
    props.noteDetail.note.updated_at = updatedAt;

    // 同步更新列表中的标题
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
