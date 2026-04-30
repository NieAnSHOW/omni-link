<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center justify-end gap-2 border-b border-border px-4 py-2">
      <Button
        v-if="notesStore.currentNote"
        variant="outline"
        size="sm"
        @click="showPersonaDialog = true"
      >
        ✨ 人格
      </Button>
    </div>

    <div class="flex-1 overflow-hidden">
      <NoteDetail
        v-if="notesStore.currentNote"
        ref="noteDetailRef"
        :note-detail="notesStore.currentNote"
        :terminal-session-id="currentSessionId"
        :persona-name="currentPersonaName"
        @save="handleSaveNote"
        @delete="handleDeleteNote"
        @terminal-close="handleTerminalClose"
        @rewrite-completed="handleRewriteCompleted"
      />
      <div v-else class="flex h-full items-center justify-center text-muted-foreground">
        <p>选择一个笔记或创建新笔记</p>
      </div>
    </div>

    <PersonaDialog
      v-if="notesStore.currentNote"
      :open="showPersonaDialog"
      :note-id="notesStore.currentNote.note.id"
      @update:open="showPersonaDialog = $event"
      @confirm="handlePersonaConfirm"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useNotesStore } from '../stores/notes';
import NoteDetail from '../components/notes/NoteDetail.vue';
import PersonaDialog from '../components/persona/PersonaDialog.vue';
import { Button } from '@/components/ui/button';
import { usePersona } from '@/composables/usePersona';
import type { Persona } from '@/types/persona';

const route = useRoute();
const router = useRouter();
const notesStore = useNotesStore();
const { startPersonaRewrite } = usePersona();

const showPersonaDialog = ref(false);
const currentSessionId = ref('');
const currentPersonaName = ref('');
const noteDetailRef = ref<InstanceType<typeof NoteDetail> | null>(null);

watch(
  () => route.params.id,
  async (id) => {
    if (id) {
      await notesStore.loadNote(Number(id));
    } else {
      notesStore.currentNote = null;
    }
  },
  { immediate: true }
);

async function handleSaveNote(id: number, title: string, content: string) {
  try {
    await notesStore.saveNote(id, title, content);
  } catch (e) {
    console.error('Failed to save note:', e);
  }
}

async function handleDeleteNote(id: number) {
  try {
    await notesStore.deleteNote(id);
    router.push('/notes');
  } catch (e) {
    console.error('Failed to delete note:', e);
  }
}

async function handlePersonaConfirm(persona: Persona, mode: 'smart' | 'manual') {
  if (!notesStore.currentNote) return;
  try {
    const notePath = `~/.omnilink/notes/${notesStore.currentNote.note.file_name}`;
    const sessionId = await startPersonaRewrite({
      noteId: notesStore.currentNote.note.id,
      notePath,
      personaSkill: persona.skill_name,
      mode,
    });
    currentSessionId.value = sessionId;
    currentPersonaName.value = persona.name;
  } catch (error) {
    console.error('启动人格重构失败:', error);
  }
}

function handleTerminalClose() {
  currentSessionId.value = '';
  currentPersonaName.value = '';
}

async function handleRewriteCompleted() {
  if (notesStore.currentNote) {
    await notesStore.loadNote(notesStore.currentNote.note.id);
  }
}
</script>
