<template>
  <div class="flex h-screen">
    <aside class="flex w-[250px] flex-col overflow-y-auto border-r border-border bg-background py-4">
      <div class="flex items-center gap-2 border-b border-border px-4 pb-4">
        <img src="/logo-no-text.png" alt="OmniLink" class="h-7 w-7" />
        <h1 class="m-0 text-lg font-semibold">OmniLink</h1>
      </div>
      <nav class="flex flex-col gap-1 p-2">
        <router-link
          to="/links"
          class="flex items-center gap-2 rounded-lg px-3 py-2.5 text-sm text-muted-foreground no-underline transition-colors hover:bg-accent hover:text-accent-foreground"
          :class="{ 'bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground': isLinksActive }"
          :aria-current="isLinksActive ? 'page' : undefined"
        >
          <span class="icon">📚</span> 知识库
        </router-link>
        <router-link
          to="/notes"
          class="flex items-center gap-2 rounded-lg px-3 py-2.5 text-sm text-muted-foreground no-underline transition-colors hover:bg-accent hover:text-accent-foreground"
          active-class="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground"
        >
          <span class="icon">📝</span> 笔记
        </router-link>
        <router-link
          to="/tags"
          class="flex items-center gap-2 rounded-lg px-3 py-2.5 text-sm text-muted-foreground no-underline transition-colors hover:bg-accent hover:text-accent-foreground"
          active-class="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground"
        >
          <span class="icon">🧠</span> 智识图谱
        </router-link>
        <router-link
          to="/settings"
          class="flex items-center gap-2 rounded-lg px-3 py-2.5 text-sm text-muted-foreground no-underline transition-colors hover:bg-accent hover:text-accent-foreground"
          active-class="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground"
        >
          <span class="icon">⚙️</span> 设置
        </router-link>
      </nav>
    </aside>
    <main class="main-content flex-1 overflow-y-auto px-6">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const isLinksActive = computed(() => route.path.startsWith('/links'));
</script>
