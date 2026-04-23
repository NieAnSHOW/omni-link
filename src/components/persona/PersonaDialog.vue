<script setup lang="ts">
import { ref, watch } from 'vue';
import { usePersona } from '@/composables/usePersona';
import type { Persona } from '@/types/persona';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

interface Props {
  open: boolean;
  noteId: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:open': [value: boolean];
  confirm: [persona: Persona, mode: 'smart' | 'manual'];
}>();

const { scanLocalPersonas } = usePersona();

const personas = ref<Persona[]>([]);
const selectedPersona = ref<Persona | null>(null);
const loading = ref(false);
const error = ref('');

const loadPersonas = async () => {
  loading.value = true;
  error.value = '';
  try {
    personas.value = await scanLocalPersonas();
  } catch (e) {
    error.value = e instanceof Error ? e.message : '扫描人格失败';
  } finally {
    loading.value = false;
  }
};

const handleConfirm = () => {
  if (selectedPersona.value) {
    emit('confirm', selectedPersona.value, 'manual');
    emit('update:open', false);
  }
};

const handleOpenChange = (value: boolean) => {
  emit('update:open', value);
};

watch(() => props.open, (open) => {
  if (open) {
    loadPersonas();
    selectedPersona.value = null;
  }
});
</script>

<template>
  <Dialog :open="open" @update:open="handleOpenChange">
    <DialogContent class="sm:max-w-[560px]">
      <DialogHeader>
        <DialogTitle>人格撰写</DialogTitle>
      </DialogHeader>

      <Tabs default-value="manual">
        <TabsList>
          <TabsTrigger value="manual">手动选择</TabsTrigger>
          <TabsTrigger value="smart" disabled>智能选择</TabsTrigger>
        </TabsList>

        <TabsContent value="manual" class="mt-4">
          <div v-if="loading" class="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <div class="mb-3 h-8 w-8 animate-spin rounded-full border-4 border-muted border-t-primary" />
            <p class="text-sm">正在扫描本地人格...</p>
          </div>

          <div v-else-if="error" class="flex flex-col items-center justify-center py-12 text-destructive">
            <p class="mb-3">{{ error }}</p>
            <Button variant="outline" size="sm" @click="loadPersonas">重试</Button>
          </div>

          <div v-else-if="personas.length === 0" class="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <p>未找到可用的人格</p>
            <p class="mt-1 text-xs">请确保已安装 Claude Code 并配置了 perspective skills</p>
          </div>

          <div v-else class="grid grid-cols-2 gap-2.5">
            <div
              v-for="persona in personas"
              :key="persona.skillName"
              class="cursor-pointer rounded-lg border-2 p-3 transition-all hover:border-primary/50"
              :class="selectedPersona?.skillName === persona.skillName
                ? 'border-primary bg-primary/5 ring-1 ring-primary'
                : 'border-border'"
              @click="selectedPersona = persona"
            >
              <div class="mb-1 flex items-center justify-between">
                <h3 class="text-sm font-semibold">{{ persona.name }}</h3>
                <Badge v-if="persona.isBuiltin" variant="secondary" class="text-[10px]">内置</Badge>
              </div>
              <p class="mb-1 text-xs text-muted-foreground">{{ persona.category }}</p>
              <p class="line-clamp-2 text-xs text-foreground/70">{{ persona.description }}</p>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="smart" class="mt-4">
          <div class="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <div class="mb-3 text-4xl">🚧</div>
            <h3 class="mb-1 text-sm font-semibold">智能选择即将开放</h3>
            <p class="text-xs">AI 将自动分析笔记内容并推荐最合适的人格风格</p>
          </div>
        </TabsContent>
      </Tabs>

      <DialogFooter>
        <Button variant="outline" @click="handleOpenChange(false)">取消</Button>
        <Button :disabled="!selectedPersona" @click="handleConfirm">开始重构</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
