import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { CategoryNode } from '../types/index';

export const useCategoriesStore = defineStore('categories', () => {
  const api = useApi();
  const categories = ref<CategoryNode[]>([]);
  const loading = ref(false);

  async function fetchCategories() {
    loading.value = true;
    try {
      categories.value = await api.getCategories();
    } finally {
      loading.value = false;
    }
  }

  async function createCategory(name: string, parentId?: number | null) {
    const node = await api.upsertCategory({ name, parentId });
    await fetchCategories();
    return node;
  }

  async function deleteCategory(id: number) {
    await api.deleteCategory(id);
    await fetchCategories();
  }

  return { categories, loading, fetchCategories, createCategory, deleteCategory };
});
