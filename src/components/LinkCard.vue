<template>
  <Card
    class="cursor-pointer transition-shadow hover:shadow-md"
    :class="{ 'opacity-60 pointer-events-none': link.ai_processing_status !== 'idle' }"
    @click="$emit('click')"
  >
    <CardHeader class="pb-3">
      <div class="flex justify-between items-center">
        <Badge variant="secondary" class="text-xs uppercase">
          {{ link.platform || 'web' }}
        </Badge>
        <Badge :variant="statusVariant" class="text-xs">
          {{ statusText }}
        </Badge>
      </div>
    </CardHeader>
    <CardContent class="space-y-2">
      <CardTitle class="text-base line-clamp-3">
        {{ link.title || link.url }}
      </CardTitle>
      <p class="text-sm text-muted-foreground truncate">
        {{ link.url }}
      </p>
    </CardContent>
    <CardFooter class="flex justify-between items-center pt-3">
      <span class="text-xs text-muted-foreground">
        {{ formatDate(link.created_at) }}
      </span>
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        @click.stop="$emit('delete', link.id)"
        title="删除"
      >
        ✕
      </Button>
    </CardFooter>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Link } from '../types/index';
import { Card, CardHeader, CardContent, CardTitle, CardFooter } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

const props = defineProps<{ link: Link }>();
defineEmits<{ click: []; delete: [id: number] }>();

const statusText = computed(() => {
  if (props.link.ai_processing_status !== 'idle') {
    const aiStatusMap: Record<string, string> = {
      organizing: 'AI 整理中',
      expanding: 'AI 扩展中',
      both: 'AI 整理并扩展中'
    };
    return aiStatusMap[props.link.ai_processing_status] || 'AI 处理中';
  }
  const map: Record<string, string> = {
    pending: '待解析',
    parsing: '解析中',
    parsed: '已解析',
    failed: '失败'
  };
  return map[props.link.status] || props.link.status;
});

const statusVariant = computed(() => {
  if (props.link.ai_processing_status !== 'idle') return 'default';
  const variantMap: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
    parsed: 'default',
    pending: 'secondary',
    parsing: 'secondary',
    failed: 'destructive'
  };
  return variantMap[props.link.status] || 'outline';
});

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}
</script>
