import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteDetail } from '../types/index';

export function useApi() {
  return {
    getSettings: () =>
      invoke<Record<string, string>>('get_settings'),

    getAiConfig: () =>
      invoke<{ provider: string; openai: { api_key: string; base_url: string; model: string }; ollama: { base_url: string; model: string } }>('get_ai_config'),

    updateAiConfig: (params: {
      provider: string;
      openaiApiKey?: string;
      openaiBaseUrl?: string;
      openaiModel?: string;
      ollamaBaseUrl?: string;
      ollamaModel?: string;
    }) =>
      invoke<void>('update_ai_config', {
        provider: params.provider,
        openaiApiKey: params.openaiApiKey ?? null,
        openaiBaseUrl: params.openaiBaseUrl ?? null,
        openaiModel: params.openaiModel ?? null,
        ollamaBaseUrl: params.ollamaBaseUrl ?? null,
        ollamaModel: params.ollamaModel ?? null,
      }),
  };
}

export const notesApi = {
  async createNote(title: string): Promise<Note> {
    return invoke('create_note', { title });
  },

  async getNote(id: number): Promise<NoteDetail> {
    return invoke('get_note', { id });
  },

  async listNotes(limit?: number, offset?: number): Promise<Note[]> {
    return invoke('list_notes', { limit, offset });
  },

  async updateNote(id: number, title: string, content: string): Promise<void> {
    return invoke('update_note', { id, title, content });
  },

  async deleteNote(id: number): Promise<void> {
    return invoke('delete_note', { id });
  },

  async createNoteFromLink(url: string): Promise<string> {
    return invoke('create_note_from_link', { url });
  },
};
