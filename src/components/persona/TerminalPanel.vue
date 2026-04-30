<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { usePersona } from '@/composables/usePersona';
import { useClaude } from '@/composables/useClaude';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { XIcon } from 'lucide-vue-next';
import 'xterm/css/xterm.css';

interface Props {
  sessionId: string;
  noteId: number;
  personaName: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
  completed: [];
  failed: [error: string];
}>();

const { updateSessionStatus } = usePersona();
const { resizeTerminal, closeSession } = useClaude();

const terminalRef = ref<HTMLElement | null>(null);
const status = ref<'running' | 'completed' | 'failed'>('running');

let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlistenOutput: UnlistenFn | null = null;
let unlistenStatus: UnlistenFn | null = null;

const statusText = computed(() => {
  switch (status.value) {
    case 'running': return '执行中';
    case 'completed': return '已完成';
    case 'failed': return '失败';
  }
});

const initTerminal = () => {
  if (!terminalRef.value) return;

  terminal = new Terminal({
    cursorBlink: false,
    disableStdin: true,
    fontSize: 13,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
    },
  });

  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.open(terminalRef.value);
  nextTick(() => fitAddon?.fit());
};

const handleResize = () => {
  if (fitAddon && terminal) {
    fitAddon.fit();
    const { rows, cols } = terminal;
    resizeTerminal(props.sessionId, rows, cols).catch(console.error);
  }
};

const setupEventListeners = async () => {
  // 使用 session-specific 事件（新 Claude 系统）
  unlistenOutput = await listen<string>(
    `claude-pty-output-${props.sessionId}`,
    (event) => {
      if (terminal && event.payload) {
        terminal.write(event.payload);
      }
    }
  );

  unlistenStatus = await listen<{ sessionId: string; status: string }>(
    'claude-session-status',
    async (event) => {
      if (event.payload.sessionId === props.sessionId) {
        if (event.payload.status === 'exited') {
          status.value = 'completed';
          try {
            await updateSessionStatus(props.sessionId, 'completed');
          } catch (e) {
            console.error('updateSessionStatus failed:', e);
          }
          emit('completed');
        }
      }
    }
  );
  // 不需要调用 startReading — start_claude_rewrite 命令已启动 output loop
};

const handleClose = async () => {
  await closeSession(props.sessionId).catch(console.error);
  cleanup();
  emit('close');
};

const cleanup = () => {
  unlistenOutput?.();
  unlistenStatus?.();
  unlistenOutput = null;
  unlistenStatus = null;
  if (terminal) {
    terminal.dispose();
    terminal = null;
  }
  window.removeEventListener('resize', handleResize);
};

onMounted(() => {
  initTerminal();
  setupEventListeners();
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  cleanup();
});
</script>

<template>
  <div class="flex h-full flex-col bg-[#1e1e1e]">
    <!-- Header bar -->
    <div class="flex items-center justify-between border-b border-[#3e3e3e] bg-[#2d2d2d] px-4 py-1.5">
      <div class="flex items-center gap-2">
        <span
          class="inline-block h-2 w-2 rounded-full"
          :class="status === 'running' ? 'animate-pulse bg-yellow-400' : status === 'completed' ? 'bg-green-400' : 'bg-red-400'"
        />
        <span class="text-[13px] font-medium text-[#d4d4d4]">{{ personaName }}</span>
        <Badge variant="secondary" class="text-[11px] text-[#a0a0a0]">
          {{ statusText }}
        </Badge>
      </div>
      <Button
        variant="ghost"
        size="icon-xs"
        class="text-[#a0a0a0] hover:text-[#d4d4d4]"
        @click="handleClose"
      >
        <XIcon class="h-3.5 w-3.5" />
      </Button>
    </div>

    <!-- Terminal body -->
    <div ref="terminalRef" class="flex-1 overflow-hidden p-2" />
  </div>
</template>
