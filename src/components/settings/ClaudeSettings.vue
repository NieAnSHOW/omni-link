<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useClaude } from '@/composables/useClaude'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

const claude = useClaude()
const saving = ref(false)
const saved = ref(false)
const cliStatus = ref<{ installed: boolean; version: string | null }>({ installed: false, version: null })
const installing = ref(false)

const config = reactive({
  provider: 'anthropic',
  apiKey: '',
  baseUrl: 'https://api.anthropic.com',
  model: 'claude-sonnet-4-20250514',
})

onMounted(async () => {
  try {
    const status = await claude.checkInstalled()
    cliStatus.value = status
  } catch (e) {
    console.error('Failed to check CLI status:', e)
  }

  try {
    const cfg = await claude.getConfig()
    config.provider = cfg.provider
    config.apiKey = cfg.api_key
    config.baseUrl = cfg.base_url
    config.model = cfg.model
  } catch (e) {
    console.error('Failed to load Claude config:', e)
  }
})

function onProviderChange() {
  if (config.provider === 'anthropic') {
    config.baseUrl = 'https://api.anthropic.com'
    config.model = 'claude-sonnet-4-20250514'
  }
}

async function handleInstallCli() {
  installing.value = true
  try {
    await claude.installCli()
    const status = await claude.checkInstalled()
    cliStatus.value = status
  } catch (e: unknown) {
    console.error('Install failed:', e)
  } finally {
    installing.value = false
  }
}

async function saveConfig() {
  saving.value = true
  saved.value = false
  try {
    await claude.updateConfig({
      provider: config.provider,
      apiKey: config.apiKey || undefined,
      baseUrl: config.baseUrl || undefined,
      model: config.model || undefined,
    })
    saved.value = true
    setTimeout(() => { saved.value = false }, 2000)
  } catch (e) {
    console.error('Failed to save Claude config:', e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Claude Code</CardTitle>
      <CardDescription>内置 Claude Code 终端配置</CardDescription>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
      <!-- CLI Status -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">CLI 状态</label>
        <div class="flex items-center gap-3">
          <span v-if="cliStatus.installed" class="text-sm text-emerald-500">
            已安装 {{ cliStatus.version }}
          </span>
          <span v-else class="text-sm text-muted-foreground">未安装</span>
          <Button
            v-if="!cliStatus.installed"
            size="sm"
            :disabled="installing"
            @click="handleInstallCli"
          >
            {{ installing ? '安装中...' : '安装 Claude CLI' }}
          </Button>
        </div>
      </div>

      <!-- Provider -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">API Provider</label>
        <select
          v-model="config.provider"
          class="flex h-8 w-full rounded-lg border border-input bg-background px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          @change="onProviderChange"
        >
          <option value="anthropic">Anthropic</option>
          <option value="openai-compatible">OpenAI Compatible</option>
          <option value="custom">自定义</option>
        </select>
      </div>

      <!-- API Key -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">API Key</label>
        <Input v-model="config.apiKey" type="password" placeholder="sk-ant-..." />
      </div>

      <!-- Base URL -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">请求地址</label>
        <Input v-model="config.baseUrl" placeholder="https://api.anthropic.com" />
      </div>

      <!-- Model -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">模型</label>
        <Input v-model="config.model" placeholder="claude-sonnet-4-20250514" />
      </div>

      <!-- Save -->
      <div class="flex items-center gap-3 pt-2">
        <Button :disabled="saving" @click="saveConfig">
          {{ saving ? '保存中...' : '保存 Claude 配置' }}
        </Button>
        <span v-if="saved" class="text-sm text-emerald-500">已保存</span>
      </div>
    </CardContent>
  </Card>
</template>
