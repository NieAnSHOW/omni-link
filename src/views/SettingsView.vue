<template>
  <div class="settings-view">
    <h2>设置</h2>

    <section class="setting-section">
      <h3>AI 配置</h3>
      <p class="hint">配置文件位置：~/.omnilink/config.json</p>

      <div class="form-group">
        <label>AI Provider</label>
        <select v-model="aiConfig.provider" @change="onProviderChange">
          <option value="openai">OpenAI</option>
          <option value="ollama">Ollama (本地)</option>
          <option value="fallback">离线模式（规则引擎）</option>
        </select>
      </div>

      <template v-if="aiConfig.provider === 'openai'">
        <div class="form-group">
          <label>API Key</label>
          <input v-model="aiConfig.apiKey" type="password" placeholder="sk-..." />
        </div>
        <div class="form-group">
          <label>Base URL（可选，支持自定义 endpoint）</label>
          <input v-model="aiConfig.baseUrl" placeholder="https://api.openai.com/v1" />
        </div>
        <div class="form-group">
          <label>模型</label>
          <input v-model="aiConfig.model" placeholder="gpt-4o-mini" />
        </div>
      </template>

      <template v-if="aiConfig.provider === 'ollama'">
        <div class="form-group">
          <label>Ollama 地址</label>
          <input v-model="aiConfig.baseUrl" placeholder="http://localhost:11434" />
        </div>
        <div class="form-group">
          <label>模型</label>
          <input v-model="aiConfig.model" placeholder="llama3.2" />
        </div>
      </template>

      <button class="btn-primary" @click="saveAiSettings" :disabled="saving">
        {{ saving ? '保存中...' : '保存 AI 配置' }}
      </button>
      <span v-if="saved" class="saved-hint">已保存</span>
    </section>

    <section class="setting-section">
      <h3>数据</h3>
      <p class="hint">数据存储位置：~/.omnilink/omnilink.db</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useApi } from '../composables/useApi';

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

<style scoped>
.settings-view { width: 100%; }
h2 { font-size: 22px; margin-bottom: 24px; }
.setting-section {
  background: #f8f9fa; border-radius: 10px;
  padding: 20px; margin-bottom: 16px;
}
.setting-section h3 { font-size: 16px; margin-bottom: 16px; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 13px; color: #64748b; margin-bottom: 4px; }
.form-group input, .form-group select {
  width: 100%; padding: 8px 12px;
  border: 1px solid #d1d5db; border-radius: 6px;
  font-size: 14px;
}
.btn-primary {
  padding: 8px 16px; background: #6366f1; color: white;
  border: none; border-radius: 8px; font-size: 14px; cursor: pointer;
  margin-top: 8px;
}
.btn-primary:disabled { opacity: 0.5; }
.saved-hint { color: #10b981; font-size: 13px; margin-left: 8px; }
.hint { color: #94a3b8; font-size: 13px; margin-bottom: 12px; }
</style>
