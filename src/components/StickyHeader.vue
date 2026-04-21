<template>
  <div ref="headerRef" class="sticky-header" :class="{ 'is-stuck': isStuck }" :style="{
    zIndex: zIndex,
    background: background,
    top: `${topOffset || 0}px`
  }">
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Props {
  zIndex: number;
  background?: string;
  topOffset?: number
}

const headerRef = ref<HTMLElement | null>(null);
const isStuck = ref(false);
let observer: IntersectionObserver | null = null;

defineProps<Props>();

onMounted(() => {
  if (headerRef.value) {
    observer = new IntersectionObserver(
      ([entry]) => {
        isStuck.value = entry.intersectionRatio < 1;
      },
      { threshold: [1], rootMargin: '-1px 0px 0px 0px' }
    );
    observer.observe(headerRef.value);
  }
});

onUnmounted(() => {
  if (observer && headerRef.value) {
    observer.unobserve(headerRef.value);
    observer.disconnect();
  }
});
</script>

<style scoped>
.sticky-header {
  position: sticky;
  transition: box-shadow 0.2s;
}
</style>