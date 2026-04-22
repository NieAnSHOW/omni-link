<template>
  <div class="notes-view">
    <div class="notes-sidebar">
      <NotesList
        :notes="notesStore.notes"
        :selected-id="notesStore.currentNote?.note.id"
        :loading="notesStore.loading"
        @select="handleSelectNote"
        @create="handleCreateNote"
      />
    </div>

    <div class="notes-content">
      <NoteDetail
        v-if="notesStore.currentNote"
        :note-detail="notesStore.currentNote"
        @save="handleSaveNote"
        @delete="handleDeleteNote"
      />
      <div v-else class="empty-state">
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

<style scoped>
.notes-view {
  display: flex;
  height: 100vh;
  background: #ffffff;
}

.notes-sidebar {
  width: 300px;
  border-right: 1px solid #e5e7eb;
  overflow-y: auto;
}

.notes-content {
  flex: 1;
  overflow: hidden;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #9ca3af;
  font-size: 16px;
}
</style>
