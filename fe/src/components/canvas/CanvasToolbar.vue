<script setup lang="ts">
import { computed } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LBadge } from '@/components/ui'

const canvasStore = useCanvasStore()

const timeRangeLabel = computed(() => {
  const preset = canvasStore.activeCanvas?.timeRange.preset ?? '7d'
  const live = canvasStore.activeCanvas?.live ? ' (live)' : ''
  return `Last ${preset}${live}`
})
</script>

<template>
  <div class="flex items-center gap-6 px-4">
    <LBadge>Time: {{ timeRangeLabel }}</LBadge>
  </div>
</template>
