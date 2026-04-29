<template>
  <div class="w-full">
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-xl font-semibold">智识图谱</h2>
      <Button @click="showAdd = true">
        ＋ 新建标签
      </Button>
    </div>

    <div v-if="showAdd" class="flex items-center gap-2 mb-4 p-3 bg-muted/50 rounded-lg">
      <Input v-model="newName" placeholder="标签名称" class="flex-1" />
      <input
        v-model="newColor"
        type="color"
        class="h-8 w-10 cursor-pointer rounded-md border border-input p-0.5"
      />
      <Button size="sm" :disabled="!newName.trim()" @click="handleCreate">确定</Button>
      <Button variant="outline" size="sm" @click="showAdd = false">取消</Button>
    </div>

    <Input v-model="search" placeholder="搜索标签..." class="mb-4" />

    <div v-if="loading" class="py-10 text-center text-muted-foreground">加载中...</div>
    <div v-else-if="filteredTags.length === 0" class="py-10 text-center text-muted-foreground">暂无标签</div>
    <div v-else class="flex flex-col gap-1.5">
      <div
        v-for="tag in filteredTags"
        :key="tag.id"
        class="flex items-center justify-between px-3.5 py-2.5 bg-background border border-border rounded-lg hover:border-muted-foreground/30 transition-colors"
      >
        <div class="flex items-center gap-2">
          <span
            class="size-2.5 rounded-full shrink-0"
            :style="{ background: tag.color }"
          ></span>
          <span class="text-sm font-medium">{{ tag.name }}</span>
          <span class="text-xs text-muted-foreground">{{ tag.content_count }} 篇内容</span>
          <Badge v-if="tag.tag_type === 'auto'" variant="secondary" class="text-[10px] uppercase">
            auto
          </Badge>
          <Badge v-else variant="outline" class="text-[10px] uppercase text-green-600">
            manual
          </Badge>
        </div>
        <div class="flex items-center gap-2">
          <Button variant="ghost" size="sm" class="text-muted-foreground hover:text-destructive" @click="handleDelete(tag.id, tag.name)">
            删除
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useTagsStore } from '../stores/tags';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

const store = useTagsStore();
const { tags, loading } = storeToRefs(store);

const showAdd = ref(false);
const newName = ref('');
const newColor = ref('#6366f1');
const search = ref('');

const filteredTags = computed(() => {
  const q = search.value.toLowerCase().trim();
  if (!q) return tags.value;
  return tags.value.filter((t: { name: string }) => t.name.toLowerCase().includes(q));
});

onMounted(() => store.fetchTags());

async function handleCreate() {
  if (!newName.value.trim()) return;
  await store.createTag(newName.value.trim(), newColor.value);
  newName.value = '';
  newColor.value = '#6366f1';
  showAdd.value = false;
}

async function handleDelete(id: number, name: string) {
  if (confirm(`确定删除标签「${name}」吗？`)) {
    await store.deleteTag(id);
  }
}
</script>
