import { invoke } from '@tauri-apps/api/core';
import type { Link, LinkDetail, TagWithCount, CategoryNode } from '../types/index';

export function useApi() {
  return {
    getLinks: (params?: { limit?: number; offset?: number; status?: string; category_id?: number }) =>
      invoke<{ links: Link[]; total: number }>('get_links', {
        limit: params?.limit ?? null,
        offset: params?.offset ?? null,
        status: params?.status ?? null,
        category_id: params?.category_id ?? null,
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

    // Categories
    getCategories: () =>
      invoke<CategoryNode[]>('get_categories'),

    upsertCategory: (params: { id?: number; name: string; parentId?: number | null }) =>
      invoke<CategoryNode>('upsert_category', { input: { id: params.id ?? null, name: params.name, parent_id: params.parentId ?? null } }),

    deleteCategory: (id: number) =>
      invoke<boolean>('delete_category', { id }),

    updateLinkCategory: (linkId: number, categoryId: number | null) =>
      invoke<void>('update_link_category', { linkId, categoryId }),

    updateContent: (id: number, title: string | null, bodyText: string, bodyHtml: string) =>
      invoke<boolean>('update_content_cmd', {
        input: { id, title, body_text: bodyText, body_html: bodyHtml },
      }),
  };
}
