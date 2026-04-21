<template>
  <div v-if="showSplash" class="splash-screen">
    <div class="splash-content">
      <img src="/logo-no-text.png" alt="OmniLink" class="splash-logo" />
      <h1 class="splash-title">Omni-Link</h1>
      <p class="splash-subtitle">链接新思维</p>
    </div>
  </div>
  <template v-else>
    <AppLayout />
  </template>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppLayout from './components/AppLayout.vue';

const showSplash = ref(true);

if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

function minDelay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

onMounted(async () => {
  await minDelay(1500);
  showSplash.value = false;
});
</script>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }

.splash-screen {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1e293b;
  z-index: 9999;
}

.splash-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.splash-logo {
  width: 80px;
  height: 80px;
}

.splash-title {
  font-size: 28px;
  font-weight: 600;
  color: #ffffff;
  letter-spacing: 1px;
}

.splash-subtitle {
  font-size: 14px;
  color: #94a3b8;
  letter-spacing: 2px;
}
</style>
