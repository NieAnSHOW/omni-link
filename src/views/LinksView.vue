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

    <div v-if="loading" class="masonry">
      <div v-for="col in columnCount" :key="col" class="masonry-col">
        <LinkCardSkeleton v-for="i in skeletonRows(col)" :key="i" />
      </div>
    </div>
    <div v-else-if="links.length === 0" class="empty-state">还没有知识，点击上方按钮添加</div>
    <div v-else class="masonry">
      <div v-for="(col, ci) in masonryColumns" :key="ci" class="masonry-col">
        <LinkCard
          v-for="link in col"
          :key="link.id"
          :link="link"
          @click="goToDetail(link.id)"
          @delete="handleDelete"
        />
      </div>
    </div>

    <AddLinkDialog
      :visible="showDialog"
      @close="showDialog = false"
      @submit="handleAddLinks"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useLinksStore } from '../stores/links';
import { useApi } from '../composables/useApi';
import { listen } from '@tauri-apps/api/event';
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
const contentWidth = ref(0);
const columnCount = computed(() =>
  Math.max(1, Math.min(5, Math.floor(contentWidth.value / 400))),
);

let resizeObs: ResizeObserver | undefined;
let unlistenComplete: (() => void) | undefined;
let unlistenFailed: (() => void) | undefined;

onMounted(async () => {
  const el = document.querySelector('.main-content');
  if (el) {
    contentWidth.value = el.clientWidth;
    resizeObs = new ResizeObserver(([e]) => { contentWidth.value = e.contentRect.width });
    resizeObs.observe(el);
  }

  // 监听 AI 处理完成事件
  unlistenComplete = await listen('ai-process-complete', async () => {
    toast.show('AI 整理完成', 'success');
    await loadLinks();
  });

  unlistenFailed = await listen('ai-process-failed', async () => {
    toast.show('AI 整理失败', 'error');
    await loadLinks();
  });
});

onUnmounted(() => {
  resizeObs?.disconnect();
  unlistenComplete?.();
  unlistenFailed?.();
});

function skeletonRows(col: number) {
  const total = 5;
  const perCol = Math.ceil(total / columnCount.value);
  return col <= (total % columnCount.value || columnCount.value) ? perCol : perCol - 1;
}

const masonryColumns = computed(() => {
  const cols: (typeof links.value)[] = Array.from({ length: columnCount.value }, () => []);
  links.value.forEach((link, i) => cols[i % columnCount.value].push(link));
  return cols;
});

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
.masonry {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}
.masonry-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-state { text-align: center; color: #94a3b8; padding: 40px; }
</style>
