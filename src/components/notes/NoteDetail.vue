<template>
  <div class="flex h-full flex-col p-4">
    <div class="flex-1">
      <VditorEditor
        v-model="workspaceStore.currentContent"
        :theme="theme"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { watch, onUnmounted } from 'vue';
import VditorEditor from './VditorEditor.vue';
import { useTheme } from '../../composables/useTheme';
import { useWorkspaceStore } from '../../stores/workspace';

const workspaceStore = useWorkspaceStore();
const { theme } = useTheme();

let saveTimer: number | null = null;

// Auto-save when currentContent changes (debounced)
watch(() => workspaceStore.currentContent, () => {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    workspaceStore.saveCurrentFile();
  }, 2000);
});

onUnmounted(() => {
  if (saveTimer) {
    clearTimeout(saveTimer);
    workspaceStore.saveCurrentFile();
  }
});
</script>
