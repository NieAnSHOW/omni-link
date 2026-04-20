import { ref } from 'vue';

interface ToastItem {
  id: number;
  message: string;
  type: 'success' | 'error';
}

export const toasts = ref<ToastItem[]>([]);
let nextId = 0;

export function useToast() {
  function show(message: string, type: 'success' | 'error' = 'success') {
    const id = nextId++;
    toasts.value.push({ id, message, type });
    setTimeout(() => remove(id), 2500);
  }

  function remove(id: number) {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }

  return { toasts, show, remove };
}
