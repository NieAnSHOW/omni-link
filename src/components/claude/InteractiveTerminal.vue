<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useClaude } from '@/composables/useClaude'
import 'xterm/css/xterm.css'

const {
  checkInstalled,
  installCli,
  startSession,
  startOutput,
  writeInput,
  resizeTerminal,
  closeSession,
} = useClaude()

const terminalRef = ref<HTMLElement | null>(null)
const status = ref<'idle' | 'installing' | 'running' | 'exited'>('idle')
const sessionId = ref<string | null>(null)
const error = ref<string | null>(null)

let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let unlistenOutput: UnlistenFn | null = null
let unlistenStatus: UnlistenFn | null = null

function initTerminal() {
  if (!terminalRef.value) return

  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1a1a1a',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
      selectionBackground: '#264f78',
    },
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.open(terminalRef.value)
  nextTick(() => fitAddon?.fit())

  terminal.onData((data) => {
    if (sessionId.value) {
      writeInput(sessionId.value, data).catch(console.error)
    }
  })
}

function handleResize() {
  if (fitAddon && terminal) {
    fitAddon.fit()
    if (sessionId.value) {
      const { rows, cols } = terminal
      resizeTerminal(sessionId.value, rows, cols).catch(console.error)
    }
  }
}

async function setupEventListeners() {
  unlistenOutput = await listen<string>('claude-pty-output', (event) => {
    if (terminal && event.payload) {
      terminal.write(event.payload)
    }
  })

  unlistenStatus = await listen<{ sessionId: string; status: string }>(
    'claude-session-status',
    (event) => {
      if (event.payload.status === 'exited') {
        status.value = 'exited'
        terminal?.write('\r\n\x1b[33m--- 会话已结束 ---\x1b[0m\r\n')
      }
    },
  )
}

async function startClaude() {
  error.value = null
  const cliStatus = await checkInstalled()

  if (!cliStatus.installed) {
    status.value = 'installing'
    terminal?.write('\x1b[36m正在下载 Claude CLI...\x1b[0m\r\n')
    try {
      await installCli()
      terminal?.write('\x1b[32m下载完成！\x1b[0m\r\n')
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = msg
      terminal?.write(`\x1b[31m下载失败: ${msg}\x1b[0m\r\n`)
      status.value = 'idle'
      return
    }
  }

  status.value = 'running'
  try {
    const sid = await startSession()
    sessionId.value = sid
    await startOutput(sid)
    terminal?.write('\x1b[32mClaude Code 已启动\x1b[0m\r\n')
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    error.value = msg
    terminal?.write(`\x1b[31m启动失败: ${msg}\x1b[0m\r\n`)
    status.value = 'idle'
  }
}

async function restart() {
  if (sessionId.value) {
    await closeSession(sessionId.value).catch(console.error)
    sessionId.value = null
  }
  terminal?.clear()
  await startClaude()
}

function cleanup() {
  unlistenOutput?.()
  unlistenStatus?.()
  unlistenOutput = null
  unlistenStatus = null
  if (sessionId.value) {
    closeSession(sessionId.value).catch(console.error)
    sessionId.value = null
  }
  if (terminal) {
    terminal.dispose()
    terminal = null
  }
  window.removeEventListener('resize', handleResize)
}

onMounted(() => {
  initTerminal()
  setupEventListeners()
  window.addEventListener('resize', handleResize)
  startClaude()
})

onUnmounted(() => {
  cleanup()
})
</script>

<template>
  <div class="flex h-full flex-col bg-[#1a1a1a]">
    <!-- Terminal body -->
    <div ref="terminalRef" class="flex-1 overflow-hidden p-2" />

    <!-- Restart button (shown when exited) -->
    <div
      v-if="status === 'exited'"
      class="flex items-center justify-center gap-3 border-t border-[#3e3e3e] px-3 py-2"
    >
      <span class="text-xs text-[#888]">会话已结束</span>
      <button
        class="rounded bg-blue-600 px-3 py-1 text-xs text-white hover:bg-blue-500 transition-colors"
        @click="restart"
      >
        重新启动
      </button>
    </div>
  </div>
</template>
