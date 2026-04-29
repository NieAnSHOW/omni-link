<template>
  <Transition
    enter-active-class="transition-opacity duration-300"
    leave-active-class="transition-opacity duration-300"
    enter-from-class="opacity-0"
    leave-to-class="opacity-0"
  >
    <Button
      v-if="visible"
      @click="scrollToTop"
      class="fixed bottom-6 right-6 z-50 h-12 w-12 rounded-full shadow-lg hover:-translate-y-0.5"
      size="icon"
      variant="default"
      aria-label="回到顶部"
    >
      ↑
    </Button>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Button } from '@/components/ui/button';

const visible = ref(false);
let scrollContainer: HTMLElement | null = null;

function handleScroll() {
  if (scrollContainer) {
    visible.value = scrollContainer.scrollTop > 300;
  }
}

function scrollToTop() {
  if (scrollContainer) {
    scrollContainer.scrollTo({ top: 0, behavior: 'smooth' });
  }
}

onMounted(() => {
  scrollContainer = document.querySelector('.main-content');
  if (scrollContainer) {
    scrollContainer.addEventListener('scroll', handleScroll);
  }
});

onUnmounted(() => {
  if (scrollContainer) {
    scrollContainer.removeEventListener('scroll', handleScroll);
  }
});
</script>
