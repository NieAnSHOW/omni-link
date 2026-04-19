<template>
  <div class="content-view">
    <button class="btn-back" @click="router.back()">← 返回</button>

    <div v-if="loading" class="empty-state">加载中...</div>
    <div v-else-if="!detail" class="empty-state">内容不存在</div>
    <template v-else>
      <div class="content-header">
        <h2>{{ detail.link.title || '未命名' }}</h2>
        <div class="meta">
          <span class="platform-badge">{{ detail.link.platform || 'web' }}</span>
          <a :href="detail.link.url" target="_blank" class="original-link">查看原文 →</a>
        </div>
      </div>

      <div v-if="detail.ai" class="ai-summary">
        <h3>AI 摘要</h3>
        <p>{{ detail.ai.summary }}</p>
        <div class="tags" v-if="detail.ai.tags.length">
          <span v-for="tag in detail.ai.tags" :key="tag" class="tag">{{ tag }}</span>
        </div>
      </div>

      <div class="actions">
        <button v-if="detail.link.status === 'pending'" class="btn-primary" @click="parseLink" :disabled="parsing">
          {{ parsing ? '解析中...' : '解析内容' }}
        </button>
        <button v-if="detail.content && !detail.ai" class="btn-primary" @click="analyzeContent" :disabled="analyzing">
          {{ analyzing ? '分析中...' : 'AI 分析' }}
        </button>
      </div>

      <div v-if="detail.content" class="content-body">
        <div v-if="detail.content.body_text" class="text-content">{{ detail.content.body_text }}</div>
        <div v-else-if="detail.content.body_html" v-html="detail.content.body_html" class="html-content"></div>
        <p v-else class="empty-state">无可显示内容</p>
      </div>
      <div v-else class="empty-state">
        <p>内容尚未解析</p>
        <button class="btn-primary" @click="parseLink" :disabled="parsing">
          {{ parsing ? '解析中...' : '开始解析' }}
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useApi } from '../composables/useApi';
import type { LinkDetail } from '../types/index';

const props = defineProps<{ id: string }>();
const router = useRouter();
const api = useApi();
const detail = ref<LinkDetail | null>(null);
const loading = ref(true);
const parsing = ref(false);
const analyzing = ref(false);

onMounted(async () => {
  await fetchDetail();
});

async function fetchDetail() {
  loading.value = true;
  try {
    detail.value = await api.getLinkDetail(Number(props.id));
  } finally {
    loading.value = false;
  }
}

async function parseLink() {
  parsing.value = true;
  try {
    await api.parseLink(Number(props.id));
    await fetchDetail();
  } finally {
    parsing.value = false;
  }
}

async function analyzeContent() {
  if (!detail.value?.content) return;
  analyzing.value = true;
  try {
    await api.analyzeContent(detail.value.content.id);
    await fetchDetail();
  } finally {
    analyzing.value = false;
  }
}
</script>

<style scoped>
.content-view { max-width: 720px; }
.btn-back { background: none; border: none; color: #6366f1; cursor: pointer; font-size: 14px; margin-bottom: 16px; }
.content-header h2 { font-size: 22px; margin-bottom: 8px; }
.meta { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; }
.platform-badge { font-size: 11px; background: #f1f5f9; padding: 2px 8px; border-radius: 4px; color: #64748b; }
.original-link { font-size: 13px; color: #6366f1; text-decoration: none; }
.ai-summary {
  background: #f0f0ff; border-radius: 10px; padding: 16px;
  margin-bottom: 20px;
}
.ai-summary h3 { font-size: 14px; color: #6366f1; margin-bottom: 8px; }
.tags { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 10px; }
.tag {
  font-size: 12px; background: #6366f1; color: white;
  padding: 2px 10px; border-radius: 12px;
}
.actions { display: flex; gap: 8px; margin-bottom: 20px; }
.btn-primary {
  padding: 8px 16px; background: #6366f1; color: white;
  border: none; border-radius: 8px; font-size: 14px; cursor: pointer;
}
.btn-primary:disabled { opacity: 0.5; }
.content-body {
  line-height: 1.8; font-size: 15px; color: #334155;
  white-space: pre-wrap;
}
.text-content { max-height: 600px; overflow-y: auto; }
.empty-state { text-align: center; color: #94a3b8; padding: 40px; }
</style>
