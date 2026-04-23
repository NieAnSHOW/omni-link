<template>
  <div class="w-full">
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-2xl font-semibold">我的知识库</h2>
      <Button @click="showDialog = true">+ 添加知识</Button>
    </div>

    <div class="flex gap-2 mb-4">
      <Button
        v-for="f in filters"
        :key="f.value"
        :variant="activeFilter === f.value ? 'default' : 'outline'"
        size="sm"
        @click="setFilter(f.value)"
      >
        {{ f.label }}
      </Button>
    </div>

    <div v-if="loading" class="flex gap-3 items-start">
      <div v-for="col in columnCount" :key="col" class="flex-1 flex flex-col gap-3">
        <LinkCardSkeleton v-for="i in skeletonRows(col)" :key="i" />
      </div>
    </div>
    <div v-else-if="links.length === 0" class="text-center text-muted-foreground py-10">
      还没有知识，点击上方按钮添加
    </div>
    <div v-else class="flex gap-3 items-start">
      <div v-for="(col, ci) in masonryColumns" :key="ci" class="flex-1 flex flex-col gap-3">
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
import { Button } from '@/components/ui/button';
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
