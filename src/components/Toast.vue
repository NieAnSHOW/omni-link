<template>
  <TransitionGroup name="toast" tag="div" class="toast-container">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      :class="['toast-item', toast.type]"
      @click="remove(toast.id)"
    >
      {{ toast.message }}
    </div>
  </TransitionGroup>
</template>

<script setup lang="ts">
import { toasts, useToast } from '../composables/useToast';

const { remove } = useToast();
</script>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 32px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}
.toast-item {
  padding: 10px 16px;
  border-radius: 8px;
  color: white;
  font-size: 14px;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
.toast-item.success { background: #10b981; }
.toast-item.error { background: #ef4444; }

.toast-enter-active { transition: all 0.3s ease; }
.toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { opacity: 0; transform: translateY(20px); }
.toast-leave-to { opacity: 0; transform: translateY(-20px); }
</style>
