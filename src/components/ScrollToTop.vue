<template>
  <transition name="fade">
    <button v-if="visible" class="scroll-to-top" @click="scrollToTop" aria-label="回到顶部">
      ↑
    </button>
  </transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

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

<style scoped>
.scroll-to-top {
  position: fixed;
  right: 24px;
  bottom: 24px;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: #6366f1;
  color: white;
  border: none;
  font-size: 20px;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
  z-index: 100;
  transition: transform 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.scroll-to-top:hover {
  transform: translateY(-2px);
  background: #4f46e5;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
