<template>
  <div class="tags-view">
    <div class="page-header">
      <h2>标签管理</h2>
      <button class="btn-primary" @click="showAdd = true">＋ 新建标签</button>
    </div>

    <div v-if="showAdd" class="add-form">
      <input v-model="newName" class="input" placeholder="标签名称" />
      <input v-model="newColor" class="input color-input" type="color" />
      <button class="btn-primary btn-sm" @click="handleCreate" :disabled="!newName.trim()">确定</button>
      <button class="btn-cancel" @click="showAdd = false">取消</button>
    </div>

    <input v-model="search" class="input search-input" placeholder="搜索标签..." />

    <div v-if="loading" class="empty-state">加载中...</div>
    <div v-else-if="filteredTags.length === 0" class="empty-state">暂无标签</div>
    <div v-else class="tag-list">
      <div v-for="tag in filteredTags" :key="tag.id" class="tag-row">
        <div class="tag-info">
          <span class="color-dot" :style="{ background: tag.color }"></span>
          <span class="tag-name">{{ tag.name }}</span>
          <span class="tag-count">{{ tag.content_count }} 篇内容</span>
          <span v-if="tag.tag_type === 'auto'" class="tag-badge auto">auto</span>
          <span v-else class="tag-badge manual">manual</span>
        </div>
        <div class="tag-actions">
          <button class="btn-text" @click="handleDelete(tag.id, tag.name)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useTagsStore } from '../stores/tags';

const store = useTagsStore();
const { tags, loading } = store;

const showAdd = ref(false);
const newName = ref('');
const newColor = ref('#6366f1');
const search = ref('');

const filteredTags = computed(() => {
  const q = search.value.toLowerCase().trim();
  if (!q) return tags;
  return tags.filter(t => t.name.toLowerCase().includes(q));
});

onMounted(() => store.fetchTags());

async function handleCreate() {
  if (!newName.value.trim()) return;
  await store.createTag(newName.value.trim(), newColor.value);
  newName.value = '';
  newColor.value = '#6366f1';
  showAdd.value = false;
}

async function handleDelete(id: number, name: string) {
  if (confirm(`确定删除标签「${name}」吗？`)) {
    await store.deleteTag(id);
  }
}
</script>

<style scoped>
.tags-view { width: 100%; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-header h2 { font-size: 22px; }
.btn-primary {
  padding: 8px 16px; background: #6366f1; color: white;
  border: none; border-radius: 8px; font-size: 14px; cursor: pointer;
}
.btn-sm { padding: 6px 12px; font-size: 13px; }
.btn-cancel {
  padding: 6px 12px; background: none; border: 1px solid #e2e8f0;
  border-radius: 8px; font-size: 13px; cursor: pointer; color: #64748b;
}
.add-form {
  display: flex; gap: 8px; align-items: center;
  margin-bottom: 16px; padding: 12px; background: #f8fafc; border-radius: 8px;
}
.input {
  border: 1px solid #e2e8f0; border-radius: 6px; padding: 8px 12px;
  font-size: 14px; outline: none;
}
.input:focus { border-color: #6366f1; }
.color-input { width: 40px; height: 36px; padding: 2px; cursor: pointer; }
.search-input { width: 100%; margin-bottom: 16px; box-sizing: border-box; }
.tag-list { display: flex; flex-direction: column; gap: 6px; }
.tag-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px; background: white; border: 1px solid #f1f5f9;
  border-radius: 8px;
}
.tag-row:hover { border-color: #e2e8f0; }
.tag-info { display: flex; align-items: center; gap: 8px; }
.color-dot { width: 10px; height: 10px; border-radius: 50%; }
.tag-name { font-size: 14px; font-weight: 500; }
.tag-count { font-size: 12px; color: #94a3b8; }
.tag-badge {
  font-size: 10px; padding: 1px 6px; border-radius: 4px;
  text-transform: uppercase;
}
.tag-badge.auto { background: #f0f0ff; color: #6366f1; }
.tag-badge.manual { background: #f0fdf4; color: #16a34a; }
.tag-actions { display: flex; gap: 8px; }
.btn-text {
  background: none; border: none; color: #94a3b8; font-size: 12px;
  cursor: pointer;
}
.btn-text:hover { color: #ef4444; }
.empty-state { text-align: center; color: #94a3b8; padding: 40px; }
</style>
