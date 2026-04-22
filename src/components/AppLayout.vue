<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <img src="/logo-no-text.png" alt="OmniLink" class="logo" />
        <h1>OmniLink</h1>
      </div>
      <nav class="nav-section">
        <router-link to="/links" class="nav-item" :class="{ active: isLinksActive }"
          :aria-current="isLinksActive ? 'page' : undefined">
          <span class="icon">📚</span> 知识库
        </router-link>
        <router-link to="/notes" class="nav-item" active-class="active">
          <span class="icon">📝</span> 笔记
        </router-link>
        <router-link to="/tags" class="nav-item" active-class="active">
          <span class="icon">🧠</span> 智识图谱
        </router-link>
        <router-link to="/settings" class="nav-item" active-class="active">
          <span class="icon">⚙️</span> 设置
        </router-link>
      </nav>
    </aside>
    <main class="main-content">
      <router-view />
    </main>
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import Toast from './Toast.vue';

const route = useRoute();
const isLinksActive = computed(() => route.path.startsWith('/links'));
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 250px;
  background: #f8f9fa;
  border-right: 1px solid #e2e8f0;
  display: flex;
  flex-direction: column;
  padding: 16px 0;
  overflow-y: auto;
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px 16px;
  border-bottom: 1px solid #e2e8f0;
}

.logo {
  width: 28px;
  height: 28px;
}

.sidebar-header h1 {
  font-size: 18px;
  margin: 0;
}

.nav-section {
  padding: 8px;
  display: flex;
  flex-direction: column;
  row-gap: 5px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  color: #475569;
  text-decoration: none;
  font-size: 14px;
}

.nav-item:hover {
  background: #e2e8f0;
}

.nav-item.active {
  background: #6366f1;
  color: white;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 24px;
}
</style>
