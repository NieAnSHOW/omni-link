<template>
  <div class="rounded-lg border border-slate-200 p-4 space-y-3">
    <div class="flex justify-between items-center">
      <Skeleton class="h-5 w-16" />
      <Skeleton class="h-5 w-12" />
    </div>
    <Skeleton class="h-6 w-3/4" />
    <Skeleton class="h-4 w-full" />
    <div class="flex justify-between items-center">
      <Skeleton class="h-4 w-24" />
      <Skeleton class="h-4 w-4" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton'
</script>
