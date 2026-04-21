<template>
  <div class="content-view">
    <button class="btn-back" @click="router.back()">← 返回</button>

    <div v-if="loading" class="skeleton-detail">
      <div class="skeleton-bar" style="width: 60%; height: 24px; margin-bottom: 12px;"></div>
      <div class="skeleton-bar" style="width: 30%; height: 18px; margin-bottom: 20px;"></div>
      <div class="skeleton-bar" style="width: 100%; height: 16px; margin-bottom: 8px;"></div>
      <div class="skeleton-bar" style="width: 100%; height: 16px; margin-bottom: 8px;"></div>
      <div class="skeleton-bar" style="width: 85%; height: 16px; margin-bottom: 8px;"></div>
      <div class="skeleton-bar" style="width: 90%; height: 16px; margin-bottom: 8px;"></div>
      <div class="skeleton-bar" style="width: 40%; height: 16px;"></div>
    </div>
    <div v-else-if="!detail" class="empty-state">内容不存在</div>
    <template v-else>
      <div class="content-header">
        <div class="top-btn">
          <h2>{{ detail.link.title || '未命名' }}</h2>
          <div class="actions">
            <button v-if="detail.link.status !== 'parsing'" class="btn-primary" @click="parseLink" :disabled="parsing">
              {{ parsing ? '解析中...' : (detail.content ? '重新解析' : '解析内容') }}
            </button>
            <button v-if="detail.content" class="btn-secondary" @click="analyzeContent"
              :disabled="analyzing">
              {{ analyzing ? '摘要生成中...' : (detail.ai ? '重新生成摘要' : 'AI 摘要') }}
            </button>
            <button v-if="detail.content" class="btn-secondary" @click="showAiProcessModal = true">
              AI 整理
            </button>
            <button v-if="detail.content && !isEditing" class="btn-secondary" @click="isEditing = true">
              编辑
            </button>
          </div>
        </div>
        <div class="meta">
          <span class="platform-badge">{{ detail.link.platform || 'web' }}</span>
          <a :href="detail.link.url" target="_blank" class="original-link">查看原文 →</a>
        </div>
          <TagInput v-if="detail.content" :model-value="contentTags" :all-tags="tagsStore.tags"
          @update:model-value="handleTagsUpdate" />
      </div>

      <div v-if="detail.ai" class="ai-summary">
        <p><span>AI 摘要：</span>{{ detail.ai.summary }}</p>
      
      </div>

      <div v-if="autoAnalyzing" class="auto-analyzing-hint">
        正在生成 AI 摘要...
      </div>

      <ContentEditor v-if="isEditing"
        :initial-title="detail.content?.title ?? null"
        :initial-body="detail.content?.body_text ?? ''"
        :saving="savingContent"
        @save="handleSaveContent"
        @cancel="isEditing = false"
      />
      <template v-else>
        <div v-if="detail.content" class="content-body">
          <div v-if="detail.content.body_text" class="markdown-content" v-html="renderedMarkdown"></div>
          <div v-else-if="detail.content.body_html" v-html="detail.content.body_html" class="html-content"></div>
          <p v-else class="empty-state">无可显示内容</p>
        </div>
        <div v-else class="empty-state">
          <p>内容尚未解析</p>
        </div>
      </template>
    </template>

    <!-- AI 整理选项弹窗 -->
    <div v-if="showAiProcessModal" class="modal-overlay" @click.self="closeAiProcessModal" @keydown.escape="closeAiProcessModal">
      <div class="modal-content" role="dialog" aria-modal="true">
        <div class="modal-header">
          <h3>选择 AI 处理方式</h3>
          <button class="modal-close" @click="closeAiProcessModal" aria-label="关闭">×</button>
        </div>
        <button class="modal-option" @click="handleAiProcess('organize')" aria-label="AI 整理内容">
          <strong>AI 整理内容</strong>
          <span>清理排版、去除冗余、补全结构</span>
        </button>
        <button class="modal-option" @click="handleAiProcess('expand')" aria-label="AI 扩展内容">
          <strong>AI 扩展内容</strong>
          <span>基于当前内容搜索并扩展补充</span>
        </button>
        <button class="modal-option" @click="handleAiProcess('both')" aria-label="整理并扩展">
          <strong>整理并扩展</strong>
          <span>先整理后扩展，完整处理</span>
        </button>
      </div>
    </div>

    <!-- AI 处理中提示 -->
    <div v-if="aiProcessing" class="ai-processing-hint">
      {{ aiProcessingText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { marked } from 'marked';
import { useApi } from '../composables/useApi';
import { useToast } from '../composables/useToast';
import { useTagsStore } from '../stores/tags';
import TagInput from '../components/TagInput.vue';
import ContentEditor from '../components/ContentEditor.vue';
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
const aiProcessingText = ref('');

const modeLabels: Record<string, string> = {
  organize: 'AI 整理中...',
  expand: 'AI 扩展中...',
  both: 'AI 整理并扩展中...',
};

async function handleAiProcess(mode: string) {
  if (!detail.value?.content || aiProcessing.value) return;
  showAiProcessModal.value = false;
  aiProcessingText.value = modeLabels[mode] || '处理中...';
  aiProcessing.value = true;
  try {
    await api.aiProcessContent(detail.value.content.id, mode);
    toast.show('AI 处理完成', 'success');
    await fetchDetail();
  } catch (e: any) {
    toast.show(e?.message || 'AI 处理失败', 'error');
  } finally {
    aiProcessing.value = false;
  }
}

function closeAiProcessModal() {
  showAiProcessModal.value = false;
}

// Focus management for modal
watch(showAiProcessModal, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      const firstOption = document.querySelector('.modal-option') as HTMLElement;
      firstOption?.focus();
    });
  }
});

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
          color: '#6366f1',
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
      // 自动触发 AI 摘要
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

<style scoped>
.content-view {
  width: 100%;
}

.btn-back {
  background: none;
  border: none;
  color: #6366f1;
  cursor: pointer;
  font-size: 14px;
  margin-bottom: 16px;
}

.content-header h2 {
  font-size: 22px;
  margin-bottom: 8px;
}

.content-header .top-btn {
  display: flex;
  justify-content: space-between;
  align-items: center
}

.meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}

.platform-badge {
  font-size: 11px;
  background: #f1f5f9;
  padding: 2px 8px;
  border-radius: 4px;
  color: #64748b;
}

.original-link {
  font-size: 13px;
  color: #6366f1;
  text-decoration: none;
}

.ai-summary {
  position: sticky;
  top: 0;
  z-index: 10;
  background: #f0f0ff;
  border-radius: 10px;
  padding: 16px;
  margin-top: 10px;
  margin-bottom: 10px;
}

.ai-summary span {
  font-size: 14px;
  color: #6366f1;
  margin-bottom: 8px;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}

.tag {
  font-size: 12px;
  background: #6366f1;
  color: white;
  padding: 2px 10px;
  border-radius: 12px;
}

.actions {
  display: flex;
  gap: 8px;
  width: 100%;
  max-width: 370px;
  justify-content: end;
}

.btn-primary {
  padding: 8px 16px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.btn-primary:disabled {
  opacity: 0.5;
}

.btn-secondary {
  padding: 8px 16px;
  background: white;
  color: #6366f1;
  border: 1px solid #6366f1;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.btn-secondary:disabled {
  opacity: 0.5;
}

.content-body {
  line-height: 1.8;
  font-size: 15px;
  color: #334155;
}

.markdown-content {
  overflow-y: auto;
}

.markdown-content :deep(h1) {
  font-size: 24px;
  margin: 24px 0 12px;
  border-bottom: 1px solid #e2e8f0;
  padding-bottom: 8px;
}

.markdown-content :deep(h2) {
  font-size: 20px;
  margin: 20px 0 10px;
  border-bottom: 1px solid #e2e8f0;
  padding-bottom: 6px;
}

.markdown-content :deep(h3) {
  font-size: 18px;
  margin: 16px 0 8px;
}

.markdown-content :deep(h4) {
  font-size: 16px;
  margin: 14px 0 6px;
}

.markdown-content :deep(p) {
  margin: 8px 0;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}

.markdown-content :deep(li) {
  margin: 4px 0;
}

.markdown-content :deep(blockquote) {
  border-left: 4px solid #6366f1;
  padding: 4px 16px;
  margin: 12px 0;
  color: #64748b;
  background: #f8fafc;
}

.markdown-content :deep(code) {
  background: #f1f5f9;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 13px;
}

.markdown-content :deep(pre) {
  background: #1e293b;
  color: #e2e8f0;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 12px 0;
}

.markdown-content :deep(pre code) {
  background: none;
  padding: 0;
  color: inherit;
}

.markdown-content :deep(a) {
  color: #6366f1;
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

.markdown-content :deep(img) {
  max-width: 100%;
  border-radius: 8px;
  margin: 8px 0;
}

.markdown-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 12px 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid #e2e8f0;
  padding: 8px 12px;
  text-align: left;
}

.markdown-content :deep(th) {
  background: #f8fafc;
}

.empty-state {
  text-align: center;
  color: #94a3b8;
  padding: 40px;
}

.skeleton-bar {
  background: linear-gradient(90deg, #e2e8f0 25%, #f1f5f9 50%, #e2e8f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}

.auto-analyzing-hint {
  text-align: center;
  padding: 12px;
  color: #6366f1;
  font-size: 14px;
  background: #f0f0ff;
  border-radius: 8px;
  margin-bottom: 16px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-content {
  background: white;
  border-radius: 12px;
  padding: 24px;
  width: 360px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.modal-content h3 {
  font-size: 16px;
  margin: 0;
  color: #1e293b;
}

.modal-close {
  background: none;
  border: none;
  font-size: 24px;
  color: #64748b;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transition: color 0.15s;
}

.modal-close:hover {
  color: #1e293b;
}

.modal-option {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  padding: 12px;
  margin-bottom: 8px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  transition: border-color 0.15s;
}

.modal-option:hover {
  border-color: #6366f1;
}

.modal-option:last-child {
  margin-bottom: 0;
}

.modal-option strong {
  font-size: 14px;
  color: #1e293b;
}

.modal-option span {
  font-size: 12px;
  color: #64748b;
}

.ai-processing-hint {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: #6366f1;
  color: white;
  padding: 10px 24px;
  border-radius: 8px;
  font-size: 14px;
  z-index: 200;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}
</style>
