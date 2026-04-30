<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center justify-end gap-2 border-b border-border px-4 py-2">
      <Button
        v-if="notesStore.currentNote"
        variant="outline"
        size="sm"
        @click="showPersonaDialog = true"
      >
        <SparklesIcon class="h-4 w-4" /> 人格
      </Button>
    </div>

    <div class="flex-1 overflow-hidden">
      <NoteDetail
        v-if="notesStore.currentNote"
        :note-detail="notesStore.currentNote"
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
import { ref, watch, inject } from 'vue';
import { useRoute } from 'vue-router';
import { useNotesStore } from '../stores/notes';
import NoteDetail from '../components/notes/NoteDetail.vue';
import PersonaDialog from '../components/persona/PersonaDialog.vue';
import { Button } from '@/components/ui/button';
import { SparklesIcon } from 'lucide-vue-next';
import type { Persona } from '@/types/persona';

const route = useRoute();
const notesStore = useNotesStore();

const claudeTerminal = inject<{
  open: () => void;
  sendPrompt: (text: string) => Promise<void>;
  startPrintSession: (prompt: string) => Promise<void>;
  isReady: () => boolean;
}>('claudeTerminal')!;

const showPersonaDialog = ref(false);

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

async function handlePersonaConfirm(persona: Persona, _mode: 'smart' | 'manual') {
  if (!notesStore.currentNote) return;
  try {
    const notePath = `~/.omnilink/notes/${notesStore.currentNote.note.file_name}`;
    const prompt = `先用 Read 工具读取 ${notePath} 的内容，然后以 ${persona.skill_name} 的视角重写全文，最后用 Write 工具将重写后的内容覆盖写回 ${notePath}。不要输出任何额外解释，直接完成文件操作。`;
    await claudeTerminal.startPrintSession(prompt);
  } catch (error) {
    console.error('启动人格重构失败:', error);
  }
}
</script>
