import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { Link } from '../types/index';

export const useLinksStore = defineStore('links', () => {
  const api = useApi();
  const links = ref<Link[]>([]);
  const total = ref(0);
  const loading = ref(false);

  async function addLink(url: string) {
    await api.addLink(url);
  }

  async function addLinks(urls: string[]) {
    await api.addLinks(urls);
  }

  async function removeLink(id: number) {
    await api.deleteLink(id);
  }

  return { links, total, loading, addLink, addLinks, removeLink };
});
