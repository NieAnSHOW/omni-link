<template>
  <div
    ref="headerRef"
    class="sticky-header"
    :class="{ 'is-stuck': isStuck }"
    :style="{
      zIndex: zIndex,
      background: background,
      top: 0
    }"
  >
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Props {
  zIndex: number;
  background?: string;
}

const props = withDefaults(defineProps<Props>(), {
  background: '#ffffff'
});

const headerRef = ref<HTMLElement | null>(null);
const isStuck = ref(false);
let observer: IntersectionObserver | null = null;

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

.sticky-header.is-stuck {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
</style>