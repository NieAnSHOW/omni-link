<script setup lang="ts">
import { ref, computed } from 'vue'
import InteractiveTerminal from '../claude/InteractiveTerminal.vue'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const panelHeight = ref(280)
const MIN_HEIGHT = 120
const MAX_HEIGHT = 600
const DEFAULT_HEIGHT = 280

const isDragging = ref(false)
let startY = 0
let startHeight = 0

function onDragStart(e: MouseEvent) {
  isDragging.value = true
  startY = e.clientY
  startHeight = panelHeight.value
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
  e.preventDefault()
}

function onDragMove(e: MouseEvent) {
  const delta = startY - e.clientY
  panelHeight.value = Math.max(MIN_HEIGHT, Math.min(MAX_HEIGHT, startHeight + delta))
}

function onDragEnd() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
}

function onDoubleClick() {
  if (isOpen.value && panelHeight.value === DEFAULT_HEIGHT) {
    isOpen.value = false
  } else {
    isOpen.value = true
    panelHeight.value = DEFAULT_HEIGHT
  }
}
</script>

<template>
  <div class="flex flex-col" :class="{ 'select-none': isDragging }">
    <!-- Drag handle / divider -->
    <div
      class="group flex cursor-row-resize items-center justify-center border-t border-border hover:bg-accent/50 transition-colors"
      style="height: 6px;"
      @mousedown="onDragStart"
      @dblclick="onDoubleClick"
    >
      <div class="h-[2px] w-8 rounded-full bg-border group-hover:bg-foreground/30 transition-colors" />
    </div>

    <!-- Status bar (collapsed) -->
    <div
      v-if="!isOpen"
      class="flex items-center justify-between border-t border-border px-4 py-1.5 text-xs text-muted-foreground bg-background cursor-pointer hover:bg-accent/30"
      @click="isOpen = true"
    >
      <span>Claude Code 就绪</span>
      <span>点击展开</span>
    </div>

    <!-- Panel (expanded) -->
    <div
      v-if="isOpen"
      :style="{ height: `${panelHeight}px` }"
      class="flex flex-col overflow-hidden bg-background"
    >
      <!-- Panel header -->
      <div class="flex items-center justify-between border-b border-border px-3 py-1">
        <span class="text-xs font-medium">Claude Code</span>
        <button
          class="text-xs text-muted-foreground hover:text-foreground px-1"
          @click="isOpen = false"
        >
          x
        </button>
      </div>

      <!-- Terminal content -->
      <div class="flex-1 overflow-hidden">
        <InteractiveTerminal />
      </div>
    </div>
  </div>
</template>
