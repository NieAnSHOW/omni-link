<template>
  <div class="flex h-screen bg-background text-foreground">
    <!-- Sidebar -->
    <aside v-show="sidebarOpen" class="flex w-[260px] shrink-0 flex-col border-r border-border bg-background">
      <!-- Workspace name -->
      <div class="flex items-center gap-2 px-3 py-3">
        <FolderOpenIcon class="h-4 w-4 text-amber-500" />
        <span class="text-sm font-semibold">{{ workspaceName }}</span>
      </div>

      <!-- Create from link button -->
      <div class="px-3 pb-2">
        <Button variant="outline" size="sm" class="w-full gap-1.5" @click="showCreateFromLink = true">
          <LinkIcon class="h-3.5 w-3.5" /> 从链接创建
        </Button>
      </div>

      <!-- File tree -->
      <div class="flex-1 overflow-y-auto px-2">
        <div v-if="workspaceStore.loading" class="px-3 py-8 text-center text-sm text-muted-foreground">加载中...</div>
        <div v-else-if="workspaceStore.fileTree.length === 0" class="px-3 py-8 text-center text-sm text-muted-foreground">
          工作区为空
        </div>
        <FileTreeNode
          v-for="entry in workspaceStore.fileTree"
          :key="entry.path"
          :entry="entry"
          :current-file-path="workspaceStore.currentFilePath"
          @open-file="handleOpenFile"
        />
      </div>

      <!-- Footer -->
      <div class="border-t border-border p-3 flex justify-center">
        <Button variant="ghost" size="icon" @click="goToSettings"><SettingsIcon class="h-4 w-4" /></Button>
      </div>
    </aside>

    <!-- Right area: main content + bottom panel -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Main -->
      <main class="flex flex-1 flex-col overflow-hidden">
        <div class="flex items-center gap-3 border-b border-border px-4" style="min-height: 44px;">
          <Button variant="ghost" size="icon" @click="sidebarOpen = !sidebarOpen">
            <component :is="sidebarOpen ? PanelLeftCloseIcon : PanelLeftOpenIcon" class="h-4 w-4" />
          </Button>
          <span class="flex-1 text-sm font-medium">{{ currentTitle }}</span>
          <div class="flex items-center gap-2">
            <slot name="topbar-actions" />
            <Button variant="ghost" size="icon" @click="claudePanelOpen = !claudePanelOpen" title="Pi Agent">
              <TerminalSquareIcon class="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" @click="toggleTheme">
              <component :is="theme === 'light' ? SunIcon : MoonIcon" class="h-4 w-4" />
            </Button>
          </div>
        </div>
        <div class="flex-1 overflow-auto">
          <slot />
        </div>
      </main>

      <!-- BottomPanel: Pi Agent 交互终端 -->
      <BottomPanel ref="bottomPanelRef" v-model="claudePanelOpen" />
    </div>

    <CreateNoteDialog
      v-if="showCreateFromLink"
      @close="showCreateFromLink = false"
      @created="handleLinkNoteCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, provide, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useWorkspaceStore } from '../stores/workspace';
import { useTheme } from '../composables/useTheme';
import { Button } from '@/components/ui/button';
import {
  FolderOpenIcon,
  LinkIcon,
  SettingsIcon,
  PanelLeftCloseIcon,
  PanelLeftOpenIcon,
  TerminalSquareIcon,
  SunIcon,
  MoonIcon,
} from 'lucide-vue-next';
import CreateNoteDialog from './CreateNoteDialog.vue';
import BottomPanel from './layout/BottomPanel.vue';
import FileTreeNode from './FileTreeNode.vue';

const router = useRouter();
const workspaceStore = useWorkspaceStore();
const { theme, toggleTheme } = useTheme();

const sidebarOpen = ref(true);
const showCreateFromLink = ref(false);
const claudePanelOpen = ref(false);
const bottomPanelRef = ref<InstanceType<typeof BottomPanel> | null>(null);

provide('agentTerminal', {
  open: () => {
    claudePanelOpen.value = true;
    nextTick(() => bottomPanelRef.value?.open());
  },
  sendPrompt: (text: string) => bottomPanelRef.value?.sendPrompt(text),
  startPrintSession: async (prompt: string) => {
    claudePanelOpen.value = true;
    await nextTick();
    await bottomPanelRef.value?.startPrintSession(prompt);
  },
  isReady: () => !!bottomPanelRef.value?.sessionId,
});

const workspaceName = computed(() => {
  if (!workspaceStore.workspacePath) return 'OmniLink';
  const parts = workspaceStore.workspacePath.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || 'OmniLink';
});

const currentTitle = computed(() => {
  if (!workspaceStore.currentFilePath) return 'OmniLink';
  // Extract filename from path
  const parts = workspaceStore.currentFilePath.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || 'OmniLink';
});

onMounted(() => {
  workspaceStore.loadFileTree();
});

function handleOpenFile(path: string) {
  workspaceStore.openFile(path);
}

function goToSettings() {
  router.push('/settings');
}

async function handleLinkNoteCreated() {
  showCreateFromLink.value = false;
  await workspaceStore.loadFileTree();
}
</script>
