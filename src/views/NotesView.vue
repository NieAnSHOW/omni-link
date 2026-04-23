<template>
  <div class="flex h-screen bg-background">
    <div class="w-[300px] shrink-0 border-r border-border overflow-y-auto">
      <NotesList
        :notes="notesStore.notes"
        :selected-id="notesStore.currentNote?.note.id"
        :loading="notesStore.loading"
        @select="handleSelectNote"
        @create="handleCreateNote"
      />
    </div>

    <div class="flex-1 overflow-hidden">
      <NoteDetail
        v-if="notesStore.currentNote"
        :note-detail="notesStore.currentNote"
        @save="handleSaveNote"
        @delete="handleDeleteNote"
      />
      <div v-else class="flex items-center justify-center h-full text-muted-foreground text-base">
        <p>选择一个笔记或创建新笔记</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useNotesStore } from '../stores/notes';
import NotesList from '../components/notes/NotesList.vue';
import NoteDetail from '../components/notes/NoteDetail.vue';

const notesStore = useNotesStore();

onMounted(() => {
  notesStore.fetchNotes();
});

async function handleSelectNote(id: number) {
  await notesStore.loadNote(id);
}

async function handleCreateNote() {
  try {
    await notesStore.createNote('未命名笔记');
  } catch (e) {
    console.error('Failed to create note:', e);
  }
}

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
  } catch (e) {
    console.error('Failed to delete note:', e);
  }
}
</script>
