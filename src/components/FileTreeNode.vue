<template>
  <div>
    <div
      class="flex cursor-pointer items-center gap-1.5 rounded px-2 py-1.5 text-sm transition-colors hover:bg-accent"
      :class="{ 'bg-accent': isActive }"
      @click="handleClick"
    >
      <!-- Expand/collapse chevron for directories -->
      <component
        :is="entry.is_dir ? (expanded ? ChevronDownIcon : ChevronRightIcon) : ChevronRightIcon"
        class="h-3.5 w-3.5 shrink-0 text-muted-foreground"
        :class="{ 'invisible': !entry.is_dir }"
      />
      <!-- Icon -->
      <component
        :is="entry.is_dir ? (expanded ? FolderOpenIcon : FolderIcon) : FileTextIcon"
        class="h-4 w-4 shrink-0"
        :class="entry.is_dir ? 'text-amber-500' : 'text-muted-foreground'"
      />
      <!-- Name -->
      <span class="truncate" :class="{ 'font-medium': entry.is_dir }">{{ entry.name }}</span>
    </div>
    <!-- Children -->
    <div v-if="entry.is_dir && expanded" class="ml-4">
      <FileTreeNode
        v-for="child in entry.children"
        :key="child.path"
        :entry="child"
        :current-file-path="currentFilePath"
        @open-file="$emit('open-file', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { FileEntry } from '../types/index';
import {
  ChevronRightIcon,
  ChevronDownIcon,
  FolderIcon,
  FolderOpenIcon,
  FileTextIcon,
} from 'lucide-vue-next';

const props = defineProps<{
  entry: FileEntry;
  currentFilePath: string | null;
}>();

const emit = defineEmits<{
  'open-file': [path: string];
}>();

const expanded = ref(false);

const isActive = computed(() => {
  if (props.entry.is_dir) return false;
  return props.currentFilePath === props.entry.path;
});

function handleClick() {
  if (props.entry.is_dir) {
    expanded.value = !expanded.value;
  } else {
    emit('open-file', props.entry.path);
  }
}
</script>
