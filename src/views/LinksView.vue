<template>
  <div class="links-view">
    <div class="page-header">
      <h2>我的知识库</h2>
      <button class="btn-primary" @click="showDialog = true">+ 添加知识</button>
    </div>

    <div class="filters">
      <button v-for="f in filters" :key="f.value"
        :class="['filter-btn', { active: activeFilter === f.value }]"
        @click="setFilter(f.value)">
        {{ f.label }}
      </button>
    </div>

    <div v-if="loading" class="link-list">
      <LinkCardSkeleton v-for="i in 5" :key="i" />
    </div>
    <div v-else-if="links.length === 0" class="empty-state">还没有知识，点击上方按钮添加</div>
    <div v-else class="link-list">
      <LinkCard
        v-for="link in links"
        :key="link.id"
        :link="link"
        @click="goToDetail(link.id)"
        @delete="handleDelete"
      />
    </div>

    <AddLinkDialog
      :visible="showDialog"
      @close="showDialog = false"
      @submit="handleAddLinks"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useLinksStore } from '../stores/links';
import { useApi } from '../composables/useApi';
import { useToast } from '../composables/useToast';
import LinkCard from '../components/LinkCard.vue';
import LinkCardSkeleton from '../components/LinkCardSkeleton.vue';
import AddLinkDialog from '../components/AddLinkDialog.vue';

const router = useRouter();
const store = useLinksStore();
const { links, loading } = storeToRefs(store);
const api = useApi();
const toast = useToast();
const showDialog = ref(false);
const activeFilter = ref<string>('');

const filters = [
  { label: '全部', value: '' },
  { label: '待解析', value: 'pending' },
  { label: '已解析', value: 'parsed' },
  { label: '失败', value: 'failed' },
];

async function loadLinks() {
  store.loading = true;
  try {
    const params: { limit: number; offset: number; status?: string } = { limit: 20, offset: 0 };
    if (activeFilter.value) {
      params.status = activeFilter.value;
    }
    const res = await api.getLinks(params);
    store.links = res.links;
    store.total = res.total;
  } finally {
    store.loading = false;
  }
}

watch(
  activeFilter,
  () => {
    loadLinks();
  },
  { immediate: true },
);

function setFilter(status: string) {
  activeFilter.value = status;
}

function goToDetail(id: number) {
  router.push({ name: 'content', params: { id } });
}

async function handleAddLinks(urls: string[]) {
  try {
    await api.addLinks(urls);
    toast.show(`成功添加 ${urls.length} 条知识`, 'success');
    await loadLinks();
  } catch (e) {
    toast.show('添加失败', 'error');
  }
}

async function handleDelete(id: number) {
  try {
    await api.deleteLink(id);
    toast.show('知识已删除', 'success');
    await loadLinks();
  } catch (e) {
    toast.show('删除失败', 'error');
  }
}
</script>

<style scoped>
.links-view { width: 100%; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-header h2 { font-size: 22px; }
.btn-primary {
  padding: 8px 16px; background: #6366f1; color: white;
  border: none; border-radius: 8px; font-size: 14px; cursor: pointer;
}
.filters { display: flex; gap: 8px; margin-bottom: 16px; }
.filter-btn {
  padding: 6px 14px; border: 1px solid #e2e8f0; background: white;
  border-radius: 6px; font-size: 13px; cursor: pointer; color: #64748b;
}
.filter-btn.active { background: #6366f1; color: white; border-color: #6366f1; }
.link-list {
  columns: 3;
  column-gap: 12px;
}

@media (max-width: 899px) {
  .link-list { columns: 2; }
}

@media (max-width: 599px) {
  .link-list { columns: 1; }
}

.empty-state { text-align: center; color: #94a3b8; padding: 40px; }
</style>
