<template>
  <div class="flex h-screen bg-background text-foreground">
    <!-- Sidebar -->
    <aside v-show="sidebarOpen" class="flex w-[260px] shrink-0 flex-col border-r border-border bg-background">
      <!-- Search -->
      <div class="p-3">
        <Input v-model="searchQuery" placeholder="搜索笔记..." />
      </div>

      <!-- New note button with dropdown -->
      <div class="relative px-3 pb-2">
        <div class="flex">
          <Button class="flex-1 rounded-r-none" @click="handleCreateBlank">
            + 新建笔记
          </Button>
          <Button variant="default" class="rounded-l-none border-l border-white/20 px-2" @click="showNewMenu = !showNewMenu">
            ▾
          </Button>
        </div>
        <div v-if="showNewMenu" class="absolute left-3 right-3 top-full z-10 overflow-hidden rounded-md border border-border bg-background shadow-md">
          <button class="block w-full px-3 py-2.5 text-left text-sm text-foreground hover:bg-accent" @click="handleCreateBlank">空白笔记</button>
          <button class="block w-full px-3 py-2.5 text-left text-sm text-foreground hover:bg-accent" @click="handleCreateFromLink">从链接创建</button>
        </div>
      </div>

      <!-- Notes list -->
      <div class="flex-1 overflow-y-auto">
        <div v-if="notesStore.loading" class="px-3 py-8 text-center text-sm text-muted-foreground">加载中...</div>
        <div v-else-if="filteredNotes.length === 0" class="px-3 py-8 text-center text-sm text-muted-foreground">
          {{ searchQuery ? '未找到匹配的笔记' : '暂无笔记' }}
        </div>
        <div
          v-for="note in filteredNotes"
          :key="note.id"
          class="cursor-pointer border-l-[3px] border-l-transparent px-4 py-3 transition-colors hover:bg-accent"
          :class="{ 'border-l-primary bg-accent': note.id === currentNoteId }"
          @click="goToNote(note.id)"
        >
          <div class="truncate text-sm font-medium">{{ note.title || '未命名笔记' }}</div>
          <div class="mt-1 text-xs text-muted-foreground">{{ formatTime(note.updated_at) }}</div>
        </div>
      </div>

      <!-- Footer -->
      <div class="border-t border-border p-3 flex justify-center">
        <Button variant="ghost" size="icon" @click="goToSettings">⚙️</Button>
      </div>
    </aside>

    <!-- Right area: main content + bottom panel -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Main -->
      <main class="flex flex-1 flex-col overflow-hidden">
        <div class="flex items-center gap-3 border-b border-border px-4" style="min-height: 44px;">
          <Button variant="ghost" size="icon" @click="sidebarOpen = !sidebarOpen">≡</Button>
          <span class="flex-1 text-sm font-medium">{{ currentTitle }}</span>
          <div class="flex items-center gap-2">
            <slot name="topbar-actions" />
            <Button variant="ghost" size="icon" @click="claudePanelOpen = !claudePanelOpen" title="Claude Code">
              ⌨
            </Button>
            <Button variant="ghost" size="icon" @click="toggleTheme">
              {{ theme === 'light' ? '☀️' : '🌙' }}
            </Button>
          </div>
        </div>
        <div class="flex-1 overflow-hidden">
          <slot />
        </div>
      </main>

      <!-- BottomPanel: Claude Code 交互终端 -->
      <BottomPanel v-model="claudePanelOpen" />
    </div>

    <CreateNoteDialog
      v-if="showCreateFromLink"
      @close="showCreateFromLink = false"
      @created="handleLinkNoteCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useNotesStore } from '../stores/notes';
import { useTheme } from '../composables/useTheme';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import CreateNoteDialog from './CreateNoteDialog.vue';
import BottomPanel from './layout/BottomPanel.vue';

const router = useRouter();
const route = useRoute();
const notesStore = useNotesStore();
const { theme, toggleTheme } = useTheme();

const sidebarOpen = ref(true);
const searchQuery = ref('');
const showNewMenu = ref(false);
const showCreateFromLink = ref(false);
const claudePanelOpen = ref(false);

const currentNoteId = computed(() => {
  const id = route.params.id;
  return id ? Number(id) : null;
});

const currentTitle = computed(() => {
  if (!currentNoteId.value) return 'OmniLink';
  const note = notesStore.notes.find(n => n.id === currentNoteId.value);
  return note?.title || 'OmniLink';
});

const filteredNotes = computed(() => {
  if (!searchQuery.value.trim()) return notesStore.notes;
  const q = searchQuery.value.toLowerCase();
  return notesStore.notes.filter(n =>
    n.title.toLowerCase().includes(q)
  );
});

onMounted(() => {
  notesStore.fetchNotes();
});

function goToNote(id: number) {
  router.push(`/notes/${id}`);
}

function goToSettings() {
  router.push('/settings');
}

async function handleCreateBlank() {
  showNewMenu.value = false;
  try {
    const note = await notesStore.createNote('未命名笔记');
    router.push(`/notes/${note.id}`);
  } catch (e) {
    console.error('Failed to create note:', e);
  }
}

function handleCreateFromLink() {
  showNewMenu.value = false;
  showCreateFromLink.value = true;
}

async function handleLinkNoteCreated(noteId: number) {
  showCreateFromLink.value = false;
  await notesStore.fetchNotes();
  router.push(`/notes/${noteId}`);
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return '刚刚';
  if (diffMins < 60) return `${diffMins} 分钟前`;
  if (diffHours < 24) return `${diffHours} 小时前`;
  if (diffDays < 7) return `${diffDays} 天前`;
  return date.toLocaleDateString('zh-CN');
}
</script>
