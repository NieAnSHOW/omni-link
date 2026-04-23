<template>
  <div class="mt-4 flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <input
        v-model="editTitle"
        class="flex h-9 w-full rounded-lg border border-input bg-transparent px-3 text-lg font-semibold outline-none ring-offset-background placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/50"
        placeholder="标题"
      />
      <div class="ml-3 flex shrink-0 gap-2">
        <Button @click="handleSave" :disabled="saving">
          {{ saving ? '保存中...' : '保存' }}
        </Button>
        <Button variant="outline" @click="emit('cancel')">取消</Button>
      </div>
    </div>
    <MdEditor v-model="editBody" :language="'zh-CN'" :style="{ height: '60vh' }" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import { MdEditor } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'

import Button from '@/components/ui/button/Button.vue'

const props = defineProps<{
  initialTitle: string | null
  initialBody: string
  saving?: boolean
}>()

const emit = defineEmits<{
  save: [title: string | null, bodyText: string]
  cancel: []
}>()

const editTitle = ref(props.initialTitle ?? '')
const editBody = ref(props.initialBody)

function handleSave() {
  emit('save', editTitle.value || null, editBody.value)
}
</script>
