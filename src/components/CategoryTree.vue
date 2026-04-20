<template>
  <div class="category-tree">
    <div class="tree-header">
      <span class="label">分类</span>
      <button class="btn-add" @click="showAdd = !showAdd" title="新建分类">＋</button>
    </div>
    <div v-if="showAdd" class="add-form">
      <input
        v-model="newName"
        class="add-input"
        placeholder="分类名称"
        @keyup.enter="handleAdd"
      />
      <button class="btn-confirm" @click="handleAdd">确定</button>
    </div>
    <div class="tree-content">
      <div
        class="tree-item all"
        :class="{ active: selectedId === null }"
        @click="$emit('select', null)"
      >
        📁 全部
      </div>
      <CategoryNodeItem
        v-for="node in categories"
        :key="node.id"
        :node="node"
        :depth="0"
        :selected-id="selectedId"
        @select="$emit('select', $event)"
        @delete="handleDelete"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { CategoryNode } from '../types/index';
import CategoryNodeItem from './CategoryNodeItem.vue';

defineProps<{
  categories: CategoryNode[];
  selectedId: number | null;
}>();

const emit = defineEmits<{
  select: [id: number | null];
  refresh: [];
}>();

const showAdd = ref(false);
const newName = ref('');

async function handleAdd() {
  if (!newName.value.trim()) return;
  const { useCategoriesStore } = await import('../stores/categories');
  const store = useCategoriesStore();
  await store.createCategory(newName.value.trim());
  newName.value = '';
  showAdd.value = false;
  emit('refresh');
}

async function handleDelete(id: number) {
  const { useCategoriesStore } = await import('../stores/categories');
  const store = useCategoriesStore();
  await store.deleteCategory(id);
  emit('refresh');
}
</script>

<style scoped>
.category-tree { padding: 8px 0; }
.tree-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 0 12px 8px; border-bottom: 1px solid #e2e8f0; margin-bottom: 4px;
}
.label { font-size: 11px; color: #94a3b8; text-transform: uppercase; letter-spacing: 1px; }
.btn-add {
  background: none; border: none; font-size: 16px; color: #94a3b8;
  cursor: pointer; padding: 0 4px;
}
.btn-add:hover { color: #6366f1; }
.add-form {
  display: flex; gap: 4px; padding: 4px 12px 8px;
}
.add-input {
  flex: 1; border: 1px solid #e2e8f0; border-radius: 4px;
  padding: 4px 8px; font-size: 12px; outline: none;
}
.add-input:focus { border-color: #6366f1; }
.btn-confirm {
  padding: 4px 8px; font-size: 12px; background: #6366f1; color: white;
  border: none; border-radius: 4px; cursor: pointer;
}
.tree-content { font-size: 13px; }
.tree-item {
  padding: 6px 12px; cursor: pointer; border-radius: 6px; margin: 1px 4px;
  display: flex; align-items: center; justify-content: space-between;
}
.tree-item:hover { background: #f1f5f9; }
.tree-item.active { background: #6366f1; color: white; }
</style>
