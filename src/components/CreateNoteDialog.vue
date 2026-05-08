<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40" @click.self="$emit('close')">
    <div class="w-[400px] max-w-[90vw] rounded-xl border border-border bg-background p-6">
      <h3 class="mb-4 text-base font-semibold">从链接创建笔记</h3>
      <Input
        v-model="url"
        placeholder="粘贴 URL..."
        :disabled="loading"
        @keyup.enter="handleCreate"
      />
      <p v-if="error" class="mt-2 text-sm text-destructive">{{ error }}</p>
      <div class="mt-4 flex justify-end gap-2">
        <Button variant="outline" :disabled="loading" @click="$emit('close')">取消</Button>
        <Button :disabled="!url.trim() || loading" @click="handleCreate">
          {{ loading ? '解析中...' : '解析创建' }}
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { notesApi } from '../composables/useApi';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

const emit = defineEmits<{
  close: [];
  created: [];
}>();

const url = ref('');
const loading = ref(false);
const error = ref<string | null>(null);

async function handleCreate() {
  if (!url.value.trim() || loading.value) return;
  loading.value = true;
  error.value = null;
  try {
    await notesApi.createNoteFromLink(url.value.trim());
    emit('created');
    emit('close');
  } catch (e) {
    error.value = e instanceof Error ? e.message : '解析失败，请检查 URL 或重试';
  } finally {
    loading.value = false;
  }
}
</script>
