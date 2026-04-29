import { defineStore } from 'pinia';
import { ref } from 'vue';
import { notesApi } from '../composables/useApi';
import type { Note, NoteDetail } from '../types/index';

export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([]);
  const currentNote = ref<NoteDetail | null>(null);
  const isEditing = ref(false);
  const searchQuery = ref('');
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchNotes(limit?: number, offset?: number) {
    loading.value = true;
    error.value = null;
    try {
      notes.value = await notesApi.listNotes(limit, offset);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取笔记列表失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function createNote(title: string) {
    loading.value = true;
    error.value = null;
    try {
      const newNote = await notesApi.createNote(title);
      notes.value.unshift(newNote);
      return newNote;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建笔记失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadNote(id: number) {
    loading.value = true;
    error.value = null;
    try {
      currentNote.value = await notesApi.getNote(id);
      isEditing.value = false;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载笔记失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function saveNote(id: number, title: string, content: string) {
    loading.value = true;
    error.value = null;
    try {
      await notesApi.updateNote(id, title, content);

      // 更新本地状态
      if (currentNote.value && currentNote.value.note.id === id) {
        currentNote.value.note.title = title;
        currentNote.value.content = content;
        currentNote.value.note.updated_at = new Date().toISOString();
      }

      // 更新列表中的笔记
      const index = notes.value.findIndex(n => n.id === id);
      if (index !== -1) {
        notes.value[index].title = title;
        notes.value[index].updated_at = new Date().toISOString();
      }

      isEditing.value = false;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存笔记失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteNote(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await notesApi.deleteNote(id);

      // 从列表中移除
      notes.value = notes.value.filter(n => n.id !== id);

      // 如果删除的是当前笔记，清空当前笔记
      if (currentNote.value && currentNote.value.note.id === id) {
        currentNote.value = null;
        isEditing.value = false;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除笔记失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    notes,
    currentNote,
    isEditing,
    searchQuery,
    loading,
    error,
    fetchNotes,
    createNote,
    loadNote,
    saveNote,
    deleteNote,
  };
});
