<template>
  <div v-if="visible" class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <h3>添加链接</h3>
      <textarea
        v-model="input"
        placeholder="粘贴一个或多个链接（每行一个）"
        rows="4"
        autofocus
      />
      <div class="actions">
        <button class="btn-secondary" @click="$emit('close')">取消</button>
        <button class="btn-primary" @click="submit" :disabled="!input.trim()">添加</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: []; submit: [urls: string[]] }>();

const input = ref('');

function submit() {
  const urls = input.value
    .split(/[\n\r]+/)
    .map(line => line.trim())
    .filter(line => {
      try { new URL(line); return true; } catch { return false; }
    });
  if (urls.length > 0) {
    emit('submit', urls);
    input.value = '';
    emit('close');
  }
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
}
.dialog {
  background: white; border-radius: 12px;
  padding: 24px; width: 480px;
  box-shadow: 0 20px 60px rgba(0,0,0,0.15);
}
.dialog h3 { margin-bottom: 16px; }
textarea {
  width: 100%; padding: 12px;
  border: 1px solid #d1d5db; border-radius: 8px;
  font-size: 14px; resize: vertical;
  font-family: inherit;
}
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
.btn-primary, .btn-secondary {
  padding: 8px 16px; border-radius: 6px;
  font-size: 14px; cursor: pointer; border: none;
}
.btn-primary { background: #6366f1; color: white; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-secondary { background: #f1f5f9; color: #475569; }
</style>
