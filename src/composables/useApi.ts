import { invoke } from '@tauri-apps/api/core';
import type { Link, LinkDetail, TagWithCount, Note, NoteDetail } from '../types/index';

export function useApi() {
  return {
    getLinks: (params?: { limit?: number; offset?: number; status?: string }) =>
      invoke<{ links: Link[]; total: number }>('get_links', {
        limit: params?.limit ?? null,
        offset: params?.offset ?? null,
        status: params?.status ?? null,
      }),

    addLink: (url: string) =>
      invoke<Link[]>('create_link', { url, urls: null, source: null }),

    addLinks: (urls: string[]) =>
      invoke<Link[]>('create_link', { url: null, urls, source: null }),

    getLink: (id: number) =>
      invoke<Link>('get_link', { id }),

    deleteLink: (id: number) =>
      invoke<boolean>('delete_link', { id }),

    parseLink: (id: number) =>
      invoke<{ link_id: number; platform: string; content: unknown; error: string | null }>('parse_link_cmd', { id }),

    getLinkDetail: (id: number) =>
      invoke<LinkDetail>('get_link_detail', { id }),

    analyzeContent: (contentId: number) =>
      invoke<{ summary: string; tags: string[] }>('analyze_content_cmd', { contentId }),

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

    // Tags
    getTags: () =>
      invoke<TagWithCount[]>('get_tags'),

    createTag: (name: string, color?: string) =>
      invoke<TagWithCount>('create_tag', { input: { name, color: color ?? null } }),

    deleteTag: (id: number) =>
      invoke<boolean>('delete_tag', { id }),

    updateContentTags: (contentId: number, tagIds: number[]) =>
      invoke<void>('update_content_tags', { contentId, tagIds }),

    updateContent: (id: number, title: string | null, bodyText: string, bodyHtml: string) =>
      invoke<boolean>('update_content_cmd', {
        input: { id, title, body_text: bodyText, body_html: bodyHtml },
      }),

    aiProcessContent: (contentId: number, mode: string) =>
      invoke<{ success: boolean }>('ai_process_content_cmd', { contentId, mode }),

    startAiProcess: (linkId: number, mode: string) =>
      invoke<void>('start_ai_process', { linkId, mode }),
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
};
