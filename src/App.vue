<template>
  <div
    v-if="showSplash"
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-slate-800 opacity-100 transition-opacity duration-300 ease-out"
    ref="splashRef"
  >
    <div class="flex flex-col items-center gap-3">
      <img src="/logo-no-text.png" alt="OmniLink" class="h-20 w-20" />
      <h1 class="text-[28px] font-semibold tracking-wide text-white">Omni-Link</h1>
      <p class="text-sm tracking-widest text-slate-400">链接新思维</p>
    </div>
  </div>
  <template v-else>
    <AppLayout>
      <router-view />
    </AppLayout>
    <ScrollToTop />
    <Toaster />
  </template>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppLayout from './components/AppLayout.vue';
import ScrollToTop from './components/ScrollToTop.vue';
import { Toaster } from '@/components/ui/toast';
import { useTheme } from './composables/useTheme';

const { initTheme } = useTheme();
initTheme();

const SPLASH_DURATION = 1200;
const FADE_DURATION = 300;

const showSplash = ref(true);
const splashRef = ref<HTMLElement | null>(null);

if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

function minDelay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

onMounted(async () => {
  await minDelay(SPLASH_DURATION);
  // Trigger fade-out
  if (splashRef.value) {
    splashRef.value.style.opacity = '0';
  }
  await minDelay(FADE_DURATION);
  showSplash.value = false;
});
</script>
