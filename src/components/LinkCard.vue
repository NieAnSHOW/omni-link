<template>
  <div class="link-card" :class="{ 'processing': link.ai_processing_status !== 'idle' }" @click="$emit('click')">
    <div class="link-header">
      <span class="platform-badge">{{ link.platform || 'web' }}</span>
      <span class="status" :class="link.status">{{ statusText }}</span>
    </div>
    <h4 class="link-title">{{ link.title || link.url }}</h4>
    <p class="link-url">{{ link.url }}</p>
    <div class="link-footer">
      <span class="time">{{ formatDate(link.created_at) }}</span>
      <button class="btn-icon" @click.stop="$emit('delete', link.id)" title="删除">✕</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Link } from '../types/index';

const props = defineProps<{ link: Link }>();
defineEmits<{ click: []; delete: [id: number] }>();

const statusText = computed(() => {
  if (props.link.ai_processing_status !== 'idle') {
    const aiStatusMap: Record<string, string> = {
      organizing: 'AI 整理中',
      expanding: 'AI 扩展中',
      both: 'AI 整理并扩展中'
    };
    return aiStatusMap[props.link.ai_processing_status] || 'AI 处理中';
  }
  const map: Record<string, string> = {
    pending: '待解析',
    parsing: '解析中',
    parsed: '已解析',
    failed: '失败'
  };
  return map[props.link.status] || props.link.status;
});

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}
</script>

<style scoped>
.link-card {
  border: 1px solid #e2e8f0; border-radius: 10px;
  padding: 16px; cursor: pointer;
  transition: box-shadow 0.15s;
  break-inside: avoid;
}
.link-card:hover { box-shadow: 0 4px 12px rgba(0,0,0,0.08); }
.link-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.platform-badge {
  font-size: 11px; background: #f1f5f9; padding: 2px 8px;
  border-radius: 4px; color: #64748b; text-transform: uppercase;
}
.status { font-size: 12px; }
.status.parsed { color: #10b981; }
.status.pending, .status.parsing { color: #f59e0b; }
.status.failed { color: #ef4444; }
.link-title {
  font-size: 15px; font-weight: 500; color: #1e293b;
  margin-bottom: 4px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
.link-url { font-size: 12px; color: #94a3b8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.link-footer { display: flex; justify-content: space-between; align-items: center; margin-top: 8px; }
.time { font-size: 12px; color: #94a3b8; }
.btn-icon {
  background: none; border: none; cursor: pointer;
  color: #94a3b8; font-size: 14px; padding: 4px;
}
.btn-icon:hover { color: #ef4444; }
.link-card.processing {
  pointer-events: none;
  opacity: 0.6;
}
.link-card.processing .status {
  color: #6366f1;
}
</style>
