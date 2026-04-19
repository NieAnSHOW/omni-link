import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useApi } from '../composables/useApi';
import type { Link } from '../types/index';

export const useLinksStore = defineStore('links', () => {
  const api = useApi();
  const links = ref<Link[]>([]);
  const total = ref(0);
  const loading = ref(false);

  async function fetchLinks(offset = 0) {
    loading.value = true;
    try {
      const res = await api.getLinks({ limit: 20, offset });
      links.value = res.links;
      total.value = res.total;
    } finally {
      loading.value = false;
    }
  }

  async function addLink(url: string) {
    await api.addLink(url);
    await fetchLinks();
  }

  async function removeLink(id: number) {
    await api.deleteLink(id);
    links.value = links.value.filter(l => l.id !== id);
  }

  return { links, total, loading, fetchLinks, addLink, removeLink };
});
