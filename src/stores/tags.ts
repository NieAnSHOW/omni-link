import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { TagWithCount } from '../types/index';

export const useTagsStore = defineStore('tags', () => {
  const api = useApi();
  const tags = ref<TagWithCount[]>([]);
  const loading = ref(false);

  async function fetchTags() {
    loading.value = true;
    try {
      tags.value = await api.getTags();
    } finally {
      loading.value = false;
    }
  }

  async function createTag(name: string, color?: string) {
    const tag = await api.createTag(name, color);
    tags.value.push(tag);
    return tag;
  }

  async function deleteTag(id: number) {
    await api.deleteTag(id);
    tags.value = tags.value.filter(t => t.id !== id);
  }

  return { tags, loading, fetchTags, createTag, deleteTag };
});
