<template>
  <Dialog :open="visible" @update:open="(val: boolean) => { if (!val) emit('close') }">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>添加知识</DialogTitle>
        <DialogDescription>粘贴一个或多个网址，每行一个</DialogDescription>
      </DialogHeader>
      <Textarea
        v-model="input"
        placeholder="粘贴一个或多个网址（每行一个）"
        rows="4"
        autofocus
        class="resize-y"
        @keydown.enter.meta="submit"
        @keydown.enter.ctrl="submit"
      />
      <DialogFooter class="flex-row justify-end gap-2">
        <Button variant="outline" @click="emit('close')">取消</Button>
        <Button :disabled="!input.trim()" @click="submit">添加</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'

defineProps<{ visible: boolean }>()
const emit = defineEmits<{ close: []; submit: [urls: string[]] }>()

const input = ref('')

function submit() {
  const urls = input.value
    .split(/[\n\r]+/)
    .map(line => line.trim())
    .filter(line => {
      try { new URL(line); return true } catch { return false }
    })
  if (urls.length > 0) {
    emit('submit', urls)
    input.value = ''
    emit('close')
  }
}
</script>
