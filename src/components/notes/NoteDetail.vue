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
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import VditorEditor from './VditorEditor.vue';
import { useTheme } from '../../composables/useTheme';
import { useWorkspaceStore } from '../../stores/workspace';

const workspaceStore = useWorkspaceStore();
const { theme } = useTheme();

let saveTimer: number | null = null;
let unlistenSessionStatus: UnlistenFn | null = null;

// Auto-save when currentContent changes (debounced)
watch(() => workspaceStore.currentContent, () => {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    workspaceStore.saveCurrentFile();
  }, 2000);
});

// Reload file when pi session exits (persona rewrite finished)
listen<{ sessionId: string; status: string }>('pi-session-status', (event) => {
  if (event.payload.status === 'exited' && workspaceStore.currentFilePath) {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    workspaceStore.reloadCurrentFile();
  }
}).then(fn => { unlistenSessionStatus = fn; });

onUnmounted(() => {
  if (saveTimer) {
    clearTimeout(saveTimer);
    workspaceStore.saveCurrentFile();
  }
  unlistenSessionStatus?.();
});
</script>
