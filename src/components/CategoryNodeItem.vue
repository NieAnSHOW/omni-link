<template>
  <div>
    <div
      class="node-item"
      :style="{ paddingLeft: `${depth * 16 + 12}px` }"
      :class="{ active: node.id === selectedId }"
      @click="$emit('select', node.id)"
    >
      <span class="node-name">
        <span class="icon">{{ node.children.length ? '📂' : '📄' }}</span>
        {{ node.name }}
      </span>
      <span class="node-actions">
        <button class="btn-del" @click.stop="$emit('delete', node.id)" title="删除">×</button>
      </span>
    </div>
    <CategoryNodeItem
      v-for="child in node.children"
      :key="child.id"
      :node="child"
      :depth="depth + 1"
      :selected-id="selectedId"
      @select="$emit('select', $event)"
      @delete="$emit('delete', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import type { CategoryNode } from '../types/index';

defineProps<{
  node: CategoryNode;
  depth: number;
  selectedId: number | null;
}>();

defineEmits<{
  select: [id: number];
  delete: [id: number];
}>();
</script>

<style scoped>
.node-item {
  padding: 6px 12px 6px 12px; cursor: pointer; border-radius: 6px;
  margin: 1px 4px; display: flex; align-items: center; justify-content: space-between;
}
.node-item:hover { background: #f1f5f9; }
.node-item.active { background: #6366f1; color: white; }
.node-name { display: flex; align-items: center; gap: 4px; }
.icon { font-size: 12px; }
.btn-del {
  background: none; border: none; color: #cbd5e1; cursor: pointer;
  font-size: 14px; padding: 0 2px; opacity: 0;
}
.node-item:hover .btn-del { opacity: 1; }
.btn-del:hover { color: #ef4444; }
.node-item.active .btn-del { color: rgba(255,255,255,0.6); }
.node-item.active .btn-del:hover { color: #fff; }
</style>
