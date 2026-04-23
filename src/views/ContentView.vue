<template>
  <div class="flex min-h-full flex-col">
    <StickyHeader>
      <Button variant="ghost" size="sm" @click="router.back()">
        &larr; 返回
      </Button>
    </StickyHeader>

    <!-- Loading skeleton -->
    <div v-if="loading" class="flex flex-col gap-3 p-6">
      <Skeleton class="h-6 w-3/5" />
      <Skeleton class="h-4 w-1/3" />
      <div class="flex flex-col gap-2 pt-2">
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-4/5" />
        <Skeleton class="h-4 w-[90%]" />
        <Skeleton class="h-4 w-2/5" />
      </div>
    </div>

    <!-- Empty state -->
    <div v-else-if="!detail" class="flex flex-1 items-center justify-center p-10 text-muted-foreground">
      内容不存在
    </div>

    <!-- Main content -->
    <template v-else>
      <StickyHeader :class="'pt-2'">
        <div class="flex w-full flex-col gap-3">
          <div class="flex items-start justify-between gap-4">
            <h2 class="text-xl font-semibold leading-tight">{{ detail.link.title || '未命名' }}</h2>
            <div class="flex shrink-0 gap-2">
              <Button v-if="detail.link.status !== 'parsing'" @click="parseLink" :disabled="parsing">
                {{ parsing ? '解析中...' : (detail.content ? '重新解析' : '解析内容') }}
              </Button>
              <Button v-if="detail.content" variant="outline" @click="analyzeContent" :disabled="analyzing">
                {{ analyzing ? '摘要生成中...' : (detail.ai ? '重新生成摘要' : 'AI 摘要') }}
              </Button>
              <Button v-if="detail.content" variant="outline" @click="showAiProcessModal = true">
                AI 整理
              </Button>
              <Button v-if="detail.content && !isEditing" variant="outline" @click="isEditing = true">
                编辑
              </Button>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <Badge variant="secondary">{{ detail.link.platform || 'web' }}</Badge>
            <a :href="detail.link.url" target="_blank" class="text-sm text-primary hover:underline">查看原文 &rarr;</a>
          </div>

          <TagInput v-if="detail.content" :model-value="contentTags" :all-tags="tagsStore.tags"
            @update:model-value="handleTagsUpdate" />

          <!-- AI summary card -->
          <Card v-if="detail.ai" class="border-primary/10 bg-primary/5">
            <CardContent class="p-4">
              <p class="text-sm leading-relaxed"><span class="font-medium text-primary">AI 摘要：</span>{{ detail.ai.summary }}</p>
            </CardContent>
          </Card>
        </div>
      </StickyHeader>

      <!-- Auto analyzing hint -->
      <div v-if="autoAnalyzing" class="mx-6 mt-4 rounded-lg bg-primary/5 px-4 py-3 text-center text-sm text-primary">
        正在生成 AI 摘要...
      </div>

      <!-- Content area -->
      <div class="flex-1 p-6">
        <ContentEditor v-if="isEditing" :initial-title="detail.content?.title ?? null"
          :initial-body="detail.content?.body_text ?? ''" :saving="savingContent" @save="handleSaveContent"
          @cancel="isEditing = false" />
        <template v-else>
          <div v-if="detail.content" class="prose prose-slate max-w-none leading-relaxed text-slate-700">
            <div v-if="detail.content.body_text" class="markdown-content" v-html="renderedMarkdown"></div>
            <div v-else-if="detail.content.body_html" v-html="detail.content.body_html"></div>
            <p v-else class="text-center text-muted-foreground">无可显示内容</p>
          </div>
          <div v-else class="flex flex-1 items-center justify-center py-10 text-muted-foreground">
            <p>内容尚未解析</p>
          </div>
        </template>
      </div>
    </template>

    <!-- AI process options dialog -->
    <Dialog v-model:open="showAiProcessModal">
      <DialogContent class="sm:max-w-[360px]">
        <DialogHeader>
          <DialogTitle>选择 AI 处理方式</DialogTitle>
        </DialogHeader>
        <div class="flex flex-col gap-2">
          <button
            class="flex flex-col gap-1 rounded-lg border border-border bg-muted/50 p-3 text-left transition-colors hover:border-primary hover:bg-muted"
            @click="handleAiProcess('organize')"
          >
            <strong class="text-sm text-foreground">AI 整理内容</strong>
            <span class="text-xs text-muted-foreground">清理排版、去除冗余、补全结构</span>
          </button>
          <button
            class="flex flex-col gap-1 rounded-lg border border-border bg-muted/50 p-3 text-left transition-colors hover:border-primary hover:bg-muted"
            @click="handleAiProcess('expand')"
          >
            <strong class="text-sm text-foreground">AI 扩展内容</strong>
            <span class="text-xs text-muted-foreground">基于当前内容搜索并扩展补充</span>
          </button>
          <button
            class="flex flex-col gap-1 rounded-lg border border-border bg-muted/50 p-3 text-left transition-colors hover:border-primary hover:bg-muted"
            @click="handleAiProcess('both')"
          >
            <strong class="text-sm text-foreground">整理并扩展</strong>
            <span class="text-xs text-muted-foreground">先整理后扩展，完整处理</span>
          </button>
        </div>
      </DialogContent>
    </Dialog>

    <!-- Confirm dialog -->
    <Dialog v-model:open="showConfirmModal">
      <DialogContent class="sm:max-w-[360px]">
        <DialogHeader>
          <DialogTitle>确认操作</DialogTitle>
        </DialogHeader>
        <p class="text-sm leading-relaxed text-slate-600">此操作会覆盖原文，确认是否要继续？</p>
        <DialogFooter>
          <Button variant="secondary" @click="closeConfirmModal">取消</Button>
          <Button @click="confirmAiProcess">继续</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- AI processing hint -->
    <div v-if="aiProcessing"
      class="fixed left-1/2 top-5 z-[200] -translate-x-1/2 rounded-lg bg-primary px-6 py-2.5 text-sm text-primary-foreground shadow-lg">
      {{ aiProcessingText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { marked } from 'marked';
import { useApi } from '../composables/useApi';
import { useToast } from '../composables/useToast';
import { useTagsStore } from '../stores/tags';
import TagInput from '../components/TagInput.vue';
import ContentEditor from '../components/ContentEditor.vue';
import StickyHeader from '../components/StickyHeader.vue';
import Button from '@/components/ui/button/Button.vue';
import Badge from '@/components/ui/badge/Badge.vue';
import Card from '@/components/ui/card/Card.vue';
import CardContent from '@/components/ui/card/CardContent.vue';
import Skeleton from '@/components/ui/skeleton/Skeleton.vue';
import Dialog from '@/components/ui/dialog/Dialog.vue';
import DialogContent from '@/components/ui/dialog/DialogContent.vue';
import DialogHeader from '@/components/ui/dialog/DialogHeader.vue';
import DialogTitle from '@/components/ui/dialog/DialogTitle.vue';
import DialogFooter from '@/components/ui/dialog/DialogFooter.vue';
import type { LinkDetail, TagWithCount } from '../types/index';

const props = defineProps<{ id: string }>();
const router = useRouter();
const api = useApi();
const toast = useToast();
const detail = ref<LinkDetail | null>(null);
const loading = ref(true);
const parsing = ref(false);
const analyzing = ref(false);
const tagsStore = useTagsStore();
const contentTags = ref<TagWithCount[]>([]);
const isEditing = ref(false);
const autoAnalyzing = ref(false);
const showAiProcessModal = ref(false);
const aiProcessing = ref(false);
const showConfirmModal = ref(false);
const pendingAiMode = ref('');
const aiProcessingText = ref('');

async function handleAiProcess(mode: string) {
  if (!detail.value?.content || aiProcessing.value) return;
  showAiProcessModal.value = false;
  pendingAiMode.value = mode;
  showConfirmModal.value = true;
}

function closeConfirmModal() {
  showConfirmModal.value = false;
  pendingAiMode.value = '';
}

async function confirmAiProcess() {
  if (!detail.value?.link || !pendingAiMode.value) return;
  showConfirmModal.value = false;

  try {
    await api.startAiProcess(detail.value.link.id, pendingAiMode.value);
    toast.show('AI 处理已启动', 'success');
    router.push({ name: 'links' });
  } catch (e: any) {
    toast.show(e?.message || 'AI 处理启动失败', 'error');
  } finally {
    pendingAiMode.value = '';
  }
}

onMounted(async () => {
  await fetchDetail();
  await tagsStore.fetchTags();
});

async function fetchDetail() {
  loading.value = true;
  try {
    detail.value = await api.getLinkDetail(Number(props.id));
    console.log('[ContentView] fetchDetail:', {
      linkId: detail.value?.link.id,
      status: detail.value?.link.status,
      hasContent: !!detail.value?.content,
      bodyTextLen: detail.value?.content?.body_text?.length ?? 0,
      bodyHtmlLen: detail.value?.content?.body_html?.length ?? 0,
      contentStatus: detail.value?.content?.content_status,
    });
    if (detail.value?.content && detail.value?.ai) {
      contentTags.value = detail.value.ai.tags.map(name => {
        const existing = tagsStore.tags.find(t => t.name === name);
        return existing || {
          id: -Date.now(),
          name,
          color: '#8b9dc3',
          tag_type: 'auto' as const,
          content_count: 0,
          created_at: '',
        };
      });
    }
  } finally {
    loading.value = false;
  }
}

async function parseLink() {
  parsing.value = true;
  try {
    const result = await api.parseLink(Number(props.id));
    if (result.error) {
      toast.show(result.error, 'error');
      await fetchDetail();
    } else {
      toast.show('解析完成', 'success');
      await fetchDetail();
      if (detail.value?.content) {
        autoAnalyzing.value = true;
        try {
          await api.analyzeContent(detail.value.content.id);
          toast.show('AI 摘要完成', 'success');
          await fetchDetail();
          await tagsStore.fetchTags();
        } catch (e) {
          console.error(e);
          toast.show('AI 摘要失败', 'error');
        } finally {
          autoAnalyzing.value = false;
        }
      }
    }
  } catch (e) {
    toast.show('解析失败', 'error');
  } finally {
    parsing.value = false;
  }
}

async function analyzeContent() {
  if (!detail.value?.content) return;
  if (analyzing.value || autoAnalyzing.value) return;
  analyzing.value = true;
  try {
    await api.analyzeContent(detail.value.content.id);
    toast.show('AI 摘要完成', 'success');
    await fetchDetail();
    await tagsStore.fetchTags();
  } catch (e) {
    toast.show('AI 摘要失败', 'error');
  } finally {
    analyzing.value = false;
  }
}

async function handleTagsUpdate(newTags: TagWithCount[]) {
  if (!detail.value?.content) return;
  contentTags.value = newTags;
  const tagIds = newTags.map(t => t.id).filter(id => id > 0);
  await api.updateContentTags(detail.value.content.id, tagIds);
}

const savingContent = ref(false);

async function handleSaveContent(title: string | null, bodyText: string) {
  if (!detail.value?.content) return;
  savingContent.value = true;
  try {
    const html = marked.parse(bodyText, { async: false });
    await api.updateContent(detail.value.content.id, title, bodyText, html);
    toast.show('内容已保存', 'success');
    isEditing.value = false;
    await fetchDetail();
  } catch (e) {
    toast.show('保存失败', 'error');
  } finally {
    savingContent.value = false;
  }
}

const renderedMarkdown = computed(() => {
  if (!detail.value?.content?.body_text) return '';
  return marked.parse(detail.value.content.body_text, { async: false });
});
</script>
