<template>
  <div class="relative">
    <div class="flex flex-wrap items-center gap-1.5 rounded-md border border-input bg-background px-2 py-1 transition-colors focus-within:border-ring focus-within:ring-1 focus-within:ring-ring/50">
      <Badge
        v-for="tag in modelValue"
        :key="tag.id"
        variant="outline"
        class="gap-1"
        :style="{ color: tag.color, borderColor: tag.color + '44', backgroundColor: tag.color + '11' }"
      >
        {{ tag.name }}
        <button
          class="ml-0.5 h-3.5 w-3.5 inline-flex items-center justify-center rounded-full opacity-60 transition-opacity hover:opacity-100"
          @click="removeTag(tag.id)"
        >
          <span class="text-xs leading-none">&times;</span>
        </button>
      </Badge>
      <Input
        ref="inputRef"
        v-model="query"
        class="h-6 min-w-[80px] flex-1 border-0 bg-transparent px-0 py-0 text-sm shadow-none ring-0 focus-visible:ring-0"
        :placeholder="modelValue.length ? '' : '+ 添加标签'"
        @input="onInput"
        @keydown.enter.prevent="handleEnter"
        @keydown.backspace="handleBackspace"
        @keydown.comma.prevent="handleEnter"
        @blur="showSuggestions = false"
        @focus="onInput"
      />
    </div>
    <div
      v-if="showSuggestions && suggestions.length"
      class="absolute top-full left-0 right-0 z-50 mt-1 max-h-[200px] overflow-y-auto rounded-lg border border-border bg-background shadow-md"
    >
      <button
        v-for="tag in suggestions"
        :key="tag.id"
        class="flex w-full items-center gap-2 px-3 py-2 text-sm text-left transition-colors hover:bg-muted"
        @mousedown.prevent="addTag(tag)"
      >
        <span class="h-2 w-2 shrink-0 rounded-full" :style="{ background: tag.color }" />
        <span>{{ tag.name }}</span>
        <span class="ml-auto text-xs text-muted-foreground">{{ tag.content_count }} 篇</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { TagWithCount } from '../types/index'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  modelValue: TagWithCount[]
  allTags: TagWithCount[]
}>()

const emit = defineEmits<{
  'update:modelValue': [tags: TagWithCount[]]
}>()

const query = ref('')
const showSuggestions = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)

const suggestions = computed(() => {
  const q = query.value.toLowerCase().trim()
  const selectedIds = new Set(props.modelValue.map(t => t.id))
  return props.allTags.filter(t => {
    if (selectedIds.has(t.id)) return false
    if (!q) return true
    return t.name.toLowerCase().includes(q)
  }).slice(0, 8)
})

function addTag(tag: TagWithCount) {
  emit('update:modelValue', [...props.modelValue, tag])
  query.value = ''
  showSuggestions.value = false
}

function removeTag(id: number) {
  emit('update:modelValue', props.modelValue.filter(t => t.id !== id))
}

function handleEnter() {
  if (suggestions.value.length === 1) {
    addTag(suggestions.value[0])
  } else if (query.value.trim()) {
    const name = query.value.trim()
    const existing = props.allTags.find(t => t.name.toLowerCase() === name.toLowerCase())
    if (existing) {
      addTag(existing)
    } else {
      emit('update:modelValue', [...props.modelValue, {
        id: -Date.now(),
        name,
        color: '#8b9dc3',
        tag_type: 'manual' as const,
        content_count: 0,
        created_at: new Date().toISOString(),
      }])
      query.value = ''
      showSuggestions.value = false
    }
  }
}

function handleBackspace() {
  if (!query.value && props.modelValue.length) {
    removeTag(props.modelValue[props.modelValue.length - 1].id)
  }
}

function onInput() {
  showSuggestions.value = true
}
</script>
