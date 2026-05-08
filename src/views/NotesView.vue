<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center justify-end gap-2 border-b border-border px-4 py-2">
      <Button
        v-if="workspaceStore.currentFilePath"
        variant="outline"
        size="sm"
        @click="showPersonaDialog = true"
      >
        <SparklesIcon class="h-4 w-4" /> 人格
      </Button>
    </div>

    <div class="flex-1 overflow-hidden">
      <NoteDetail
        v-if="workspaceStore.currentFilePath"
      />
      <div v-else class="flex h-full items-center justify-center text-muted-foreground">
        <p>从左侧文件树选择一个文件</p>
      </div>
    </div>

    <PersonaDialog
      v-if="workspaceStore.currentFilePath"
      :open="showPersonaDialog"
      :note-id="0"
      @update:open="showPersonaDialog = $event"
      @confirm="handlePersonaConfirm"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue';
import { useWorkspaceStore } from '../stores/workspace';
import NoteDetail from '../components/notes/NoteDetail.vue';
import PersonaDialog from '../components/persona/PersonaDialog.vue';
import { Button } from '@/components/ui/button';
import { SparklesIcon } from 'lucide-vue-next';
import type { Persona } from '@/types/persona';

const workspaceStore = useWorkspaceStore();

const agentTerminal = inject<{
  open: () => void;
  sendPrompt: (text: string) => Promise<void>;
  startPrintSession: (prompt: string) => Promise<void>;
  isReady: () => boolean;
}>('agentTerminal')!;

const showPersonaDialog = ref(false);

async function handlePersonaConfirm(persona: Persona, _mode: 'smart' | 'manual') {
  if (!workspaceStore.currentFilePath) return;
  try {
    const prompt = `先用 Read 工具读取 ${workspaceStore.currentFilePath} 的内容，然后以 ${persona.skill_name} 的视角重写全文，最后用 Write 工具将重写后的内容覆盖写回 ${workspaceStore.currentFilePath}。不要输出任何额外解释，直接完成文件操作。`;
    await agentTerminal.startPrintSession(prompt);
  } catch (error) {
    console.error('启动人格重构失败:', error);
  }
}
</script>
