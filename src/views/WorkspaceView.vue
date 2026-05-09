<template>
  <div class="flex h-full items-center justify-center">
    <div class="flex flex-col items-center gap-6">
      <img src="/logo-no-text.png" alt="OmniLink" class="h-20 w-20" />
      <div class="flex flex-col items-center gap-1">
        <h1 class="text-2xl font-semibold tracking-tight">Omni-Link</h1>
        <p class="text-sm text-muted-foreground">选择或创建一个工作区来开始</p>
      </div>
      <Button
        size="lg"
        :disabled="workspaceStore.loadingTree"
        @click="workspaceStore.selectWorkspace()"
      >
        <FolderOpen class="h-4 w-4" />
        {{ workspaceStore.loadingTree ? '选择中...' : '打开文件夹' }}
      </Button>
      <p v-if="workspaceStore.error" class="text-sm text-destructive">
        {{ workspaceStore.error }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useWorkspaceStore } from '../stores/workspace';
import { Button } from '@/components/ui/button';
import { FolderOpen } from 'lucide-vue-next';

const workspaceStore = useWorkspaceStore();
</script>
