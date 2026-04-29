<template>
  <div class="flex min-h-full flex-col gap-6 p-6">
    <h2 class="text-xl font-semibold tracking-tight">设置</h2>

    <!-- AI 配置 -->
    <Card>
      <CardHeader>
        <CardTitle>AI 配置</CardTitle>
        <CardDescription>配置文件位置：~/.omnilink/config.json</CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm text-muted-foreground">AI Provider</label>
          <select
            v-model="aiConfig.provider"
            @change="onProviderChange"
            class="flex h-8 w-full rounded-lg border border-input bg-background px-3 py-1 text-sm shadow-xs transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          >
            <option value="openai">OpenAI</option>
            <option value="ollama">Ollama (本地)</option>
            <option value="fallback">离线模式（规则引擎）</option>
          </select>
        </div>

        <template v-if="aiConfig.provider === 'openai'">
          <div class="flex flex-col gap-1.5">
            <label class="text-sm text-muted-foreground">API Key</label>
            <Input v-model="aiConfig.apiKey" type="password" placeholder="sk-..." />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-sm text-muted-foreground">Base URL（可选，支持自定义 endpoint）</label>
            <Input v-model="aiConfig.baseUrl" placeholder="https://api.openai.com/v1" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-sm text-muted-foreground">模型</label>
            <Input v-model="aiConfig.model" placeholder="gpt-4o-mini" />
          </div>
        </template>

        <template v-if="aiConfig.provider === 'ollama'">
          <div class="flex flex-col gap-1.5">
            <label class="text-sm text-muted-foreground">Ollama 地址</label>
            <Input v-model="aiConfig.baseUrl" placeholder="http://localhost:11434" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-sm text-muted-foreground">模型</label>
            <Input v-model="aiConfig.model" placeholder="llama3.2" />
          </div>
        </template>

        <div class="flex items-center gap-3 pt-2">
          <Button @click="saveAiSettings" :disabled="saving">
            {{ saving ? '保存中...' : '保存 AI 配置' }}
          </Button>
          <span v-if="saved" class="text-sm text-emerald-500">已保存</span>
        </div>
      </CardContent>
    </Card>

    <!-- 数据信息 -->
    <Card>
      <CardHeader>
        <CardTitle>数据</CardTitle>
        <CardDescription>数据存储位置：~/.omnilink/omnilink.db</CardDescription>
      </CardHeader>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useApi } from '../composables/useApi';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

const api = useApi();
const saving = ref(false);
const saved = ref(false);

const aiConfig = reactive({
  provider: 'fallback' as string,
  apiKey: '',
  baseUrl: '',
  model: '',
});

onMounted(async () => {
  try {
    const config = await api.getAiConfig();
    aiConfig.provider = config.provider;
    if (config.provider === 'openai') {
      aiConfig.apiKey = config.openai.api_key;
      aiConfig.baseUrl = config.openai.base_url;
      aiConfig.model = config.openai.model;
    } else if (config.provider === 'ollama') {
      aiConfig.baseUrl = config.ollama.base_url;
      aiConfig.model = config.ollama.model;
    }
  } catch (e) {
    console.error('Failed to load AI config:', e);
  }
});

function onProviderChange() {
  aiConfig.apiKey = '';
  if (aiConfig.provider === 'ollama') {
    aiConfig.baseUrl = 'http://localhost:11434';
    aiConfig.model = 'llama3.2';
  } else if (aiConfig.provider === 'openai') {
    aiConfig.baseUrl = 'https://api.openai.com/v1';
    aiConfig.model = 'gpt-4o-mini';
  }
}

async function saveAiSettings() {
  saving.value = true;
  saved.value = false;
  try {
    await api.updateAiConfig({
      provider: aiConfig.provider,
      openaiApiKey: aiConfig.apiKey || undefined,
      openaiBaseUrl: aiConfig.baseUrl || undefined,
      openaiModel: aiConfig.model || undefined,
      ollamaBaseUrl: aiConfig.provider === 'ollama' ? aiConfig.baseUrl : undefined,
      ollamaModel: aiConfig.provider === 'ollama' ? aiConfig.model : undefined,
    });
    saved.value = true;
    setTimeout(() => { saved.value = false; }, 2000);
  } catch (e) {
    console.error('Failed to save AI config:', e);
  } finally {
    saving.value = false;
  }
}
</script>
