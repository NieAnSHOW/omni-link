<template>
  <div class="flex flex-col h-full bg-white">
    <div class="flex items-center gap-2 px-4 py-3 border-b border-gray-200 bg-gray-50">
      <input
        v-model="localTitle"
        type="text"
        class="flex-1 px-3 py-1.5 text-sm font-medium border border-gray-200 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 bg-white"
        placeholder="标题"
      />
      <button
        class="px-3 py-1.5 text-sm text-gray-500 border border-gray-200 rounded-md hover:bg-gray-100 transition-colors"
        @click="$emit('cancel')"
      >
        取消
      </button>
      <button
        class="px-3 py-1.5 text-sm text-white bg-indigo-500 rounded-md hover:bg-indigo-600 transition-colors"
        @click="handleSave"
      >
        保存
      </button>
    </div>

    <div class="flex-1 overflow-hidden">
      <VditorEditor
        v-model="localContent"
        :theme="theme"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import VditorEditor from './VditorEditor.vue'
import { useTheme } from '@/composables/useTheme'

const { theme } = useTheme()

interface Props {
  title?: string
  content?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  content: '',
})

const emit = defineEmits<{
  save: [title: string, content: string]
  cancel: []
}>()

const localTitle = ref(props.title)
const localContent = ref(props.content)

watch(
  () => props.title,
  (val) => {
    if (val !== localTitle.value) {
      localTitle.value = val
    }
  },
)

watch(
  () => props.content,
  (val) => {
    if (val !== localContent.value) {
      localContent.value = val
    }
  },
)

function handleSave() {
  emit('save', localTitle.value, localContent.value)
}
</script>
