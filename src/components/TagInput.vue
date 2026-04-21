<template>
  <div class="tag-input">
    <div class="tags-area">
      <span
        v-for="tag in modelValue"
        :key="tag.id"
        class="tag-pill"
        :style="{ background: tag.color + '22', color: tag.color, borderColor: tag.color + '44' }"
      >
        {{ tag.name }}
        <button class="tag-remove" @click="removeTag(tag.id)">×</button>
      </span>
      <input
        ref="inputRef"
        v-model="query"
        class="tag-text-input"
        :placeholder="modelValue.length ? '' : '+ 添加标签'"
        @input="onInput"
        @keydown.enter.prevent="handleEnter"
        @keydown.backspace="handleBackspace"
        @blur="showSuggestions = false"
        @focus="onInput"
      />
    </div>
    <div v-if="showSuggestions && suggestions.length" class="suggestions">
      <div
        v-for="tag in suggestions"
        :key="tag.id"
        class="suggestion-item"
        @mousedown.prevent="addTag(tag)"
      >
        <span class="suggestion-dot" :style="{ background: tag.color }"></span>
        {{ tag.name }}
        <span class="suggestion-count">{{ tag.content_count }} 篇</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { TagWithCount } from '../types/index';

const props = defineProps<{
  modelValue: TagWithCount[];
  allTags: TagWithCount[];
}>();

const emit = defineEmits<{
  'update:modelValue': [tags: TagWithCount[]];
}>();

const query = ref('');
const showSuggestions = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

const suggestions = computed(() => {
  const q = query.value.toLowerCase().trim();
  const selectedIds = new Set(props.modelValue.map(t => t.id));
  return props.allTags.filter(t => {
    if (selectedIds.has(t.id)) return false;
    if (!q) return true;
    return t.name.toLowerCase().includes(q);
  }).slice(0, 8);
});

function addTag(tag: TagWithCount) {
  emit('update:modelValue', [...props.modelValue, tag]);
  query.value = '';
  showSuggestions.value = false;
}

function removeTag(id: number) {
  emit('update:modelValue', props.modelValue.filter(t => t.id !== id));
}

function handleEnter() {
  if (suggestions.value.length === 1) {
    addTag(suggestions.value[0]);
  } else if (query.value.trim()) {
    const name = query.value.trim();
    const existing = props.allTags.find(t => t.name.toLowerCase() === name.toLowerCase());
    if (existing) {
      addTag(existing);
    } else {
      emit('update:modelValue', [...props.modelValue, {
        id: -Date.now(),
        name,
        color: '#8b9dc3',
        tag_type: 'manual' as const,
        content_count: 0,
        created_at: new Date().toISOString(),
      }]);
      query.value = '';
      showSuggestions.value = false;
    }
  }
}

function handleBackspace() {
  if (!query.value && props.modelValue.length) {
    removeTag(props.modelValue[props.modelValue.length - 1].id);
  }
}

function onInput() {
  showSuggestions.value = true;
}
</script>

<style scoped>
.tag-input { position: relative; }
.tags-area {
  display: flex; flex-wrap: wrap; gap: 6px; align-items: center;
  padding: 6px 8px; border: 1px solid #e2e8f0; border-radius: 8px;
  min-height: 38px; background: white;
}
.tags-area:focus-within { border-color: #6366f1; }
.tag-pill {
  display: flex; align-items: center; gap: 4px;
  padding: 2px 10px; border-radius: 12px; font-size: 12px;
  border: 1px solid; white-space: nowrap;
}
.tag-remove {
  background: none; border: none; cursor: pointer;
  font-size: 14px; padding: 0; line-height: 1; opacity: 0.6;
}
.tag-remove:hover { opacity: 1; }
.tag-text-input {
  border: none; outline: none; font-size: 13px;
  flex: 1; min-width: 80px; padding: 2px 0;
}
.suggestions {
  position: absolute; top: 100%; left: 0; right: 0;
  background: white; border: 1px solid #e2e8f0; border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1); z-index: 50;
  max-height: 200px; overflow-y: auto; margin-top: 4px;
}
.suggestion-item {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; cursor: pointer; font-size: 13px;
}
.suggestion-item:hover { background: #f1f5f9; }
.suggestion-dot { width: 8px; height: 8px; border-radius: 50%; }
.suggestion-count { margin-left: auto; font-size: 11px; color: #94a3b8; }
</style>
