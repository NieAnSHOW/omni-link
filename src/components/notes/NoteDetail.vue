<template>
  <input v-model="localTitle" type="text"
    class="w-full border-none bg-transparent text-2xl font-semibold outline-none placeholder:text-muted-foreground"
    placeholder="笔记标题" @blur="handleSave" />
  <div class="mt-3 flex items-center gap-2">
    <Button :disabled="saving" @click="handleSave">
      保存
    </Button>
    <Button variant="secondary" @click="showPersonaDialog = true">
      ✨ 人格撰写
    </Button>
    <Button v-if="currentSessionId" variant="outline" @click="reopenTerminal">
      🖥️ 查看终端
    </Button>
    <Button variant="destructive" @click="handleDelete">
      删除
    </Button>
  </div>
  <div class="flex items-center gap-4 border-b px-6 py-2 text-sm text-muted-foreground">
    <span>创建于 {{ formatDate(noteDetail.note.created_at) }}</span>
    <span>更新于 {{ formatDate(noteDetail.note.updated_at) }}</span>
    <Badge v-if="saveStatus" :variant="saveStatus === 'success' ? 'default' : 'destructive'" class="ml-auto">
      {{ saveStatus === 'success' ? '已保存' : '保存失败' }}
    </Badge>
  </div>

  <div class="flex-1">
    <MdEditor v-model="localContent" :language="'zh-CN'" :style="{ height: 'calc(100vh - 260px)' }"
      @update:model-value="handleContentChange" />
  </div>

  <PersonaDialog
    :open="showPersonaDialog"
    :note-id="noteDetail.note.id"
    @update:open="showPersonaDialog = $event"
    @confirm="handlePersonaConfirm"
  />

  <Dialog :open="showTerminalDialog && showTerminal" @update:open="handleTerminalDialogChange">
    <DialogContent class="flex h-[90vh] w-[95vw] max-w-[95vw] flex-col gap-0 overflow-hidden p-0 sm:h-[90vh] sm:max-w-[95vw]">
      <TerminalPanel
        v-if="showTerminal"
        :session-id="currentSessionId"
        :note-id="noteDetail.note.id"
        :persona-name="currentPersonaName"
        @close="handleTerminalClose"
        @completed="handleRewriteCompleted"
        @failed="handleRewriteFailed"
      />
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { NoteDetail } from '../../types/index';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import { notesApi } from '../../composables/useApi';
import { useNotesStore } from '../../stores/notes';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent } from '@/components/ui/dialog';
import PersonaDialog from '@/components/persona/PersonaDialog.vue';
import TerminalPanel from '@/components/persona/TerminalPanel.vue';
import { usePersona } from '@/composables/usePersona';
import type { Persona } from '@/types/persona';

interface Props {
  noteDetail: NoteDetail;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [id: number, title: string, content: string];
  delete: [id: number];
}>();

const { startPersonaRewrite } = usePersona();

const showPersonaDialog = ref(false);
const showTerminal = ref(false);
const showTerminalDialog = ref(false);
const currentSessionId = ref('');
const currentPersonaName = ref('');

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

function handleDelete() {
  if (confirm('确定要删除这篇笔记吗？')) {
    emit('delete', props.noteDetail.note.id);
  }
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN');
}

async function handlePersonaConfirm(persona: Persona) {
  showPersonaDialog.value = false;
  try {
    const notePath = `~/.omnilink/notes/${props.noteDetail.note.file_name}`;
    const sessionId = await startPersonaRewrite({
      noteId: props.noteDetail.note.id,
      notePath,
      personaSkill: persona.skill_name,
      mode: 'manual',
    });
    currentSessionId.value = sessionId;
    currentPersonaName.value = persona.name;
    showTerminal.value = true;
    showTerminalDialog.value = true;
  } catch (error) {
    console.error('启动人格重构失败:', error);
  }
}

function handleTerminalClose() {
  showTerminal.value = false;
  showTerminalDialog.value = false;
  currentSessionId.value = '';
  currentPersonaName.value = '';
}

function handleTerminalDialogChange(open: boolean) {
  if (!open) {
    showTerminal.value = false;
    showTerminalDialog.value = false;
  }
}

function reopenTerminal() {
  showTerminal.value = true;
  showTerminalDialog.value = true;
}

async function handleRewriteCompleted() {
  const detail = await notesApi.getNote(props.noteDetail.note.id);
  props.noteDetail.note = detail.note;
  props.noteDetail.content = detail.content;
  localTitle.value = detail.note.title;
  localContent.value = detail.content;
}

function handleRewriteFailed(error: string) {
  console.error('人格重构失败:', error);
}
</script>
