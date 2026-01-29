<script setup lang="ts">
import { computed } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LButton, LSelect, LBadge } from '@/components/ui'
import { FilePlus } from 'lucide-vue-next'

const emit = defineEmits<{
  'new-canvas': []
}>()

const canvasStore = useCanvasStore()

const canvasOptions = computed(() =>
  canvasStore.canvases.map((c) => ({ value: c.id, label: c.name })),
)

const timeRangeLabel = computed(() => {
  const preset = canvasStore.activeCanvas?.timeRange.preset ?? '7d'
  const live = canvasStore.activeCanvas?.live ? ' (live)' : ''
  return `Last ${preset}${live}`
})

function onCanvasChange(id: string | number) {
  canvasStore.setActiveCanvas(String(id))
}

function createNewCanvas() {
  const name = prompt('Canvas name:', 'New Canvas')
  if (name) {
    canvasStore.createCanvas(name)
    emit('new-canvas')
  }
}
</script>

<template>
  <header class="flex items-center gap-6 px-4">
    <div class="flex items-center gap-2 font-semibold">
      <LSelect
        :model-value="canvasStore.activeCanvasId ?? ''"
        :options="canvasOptions"
        class="min-w-40"
        @update:model-value="onCanvasChange"
      />
      <LButton variant="secondary" @click="createNewCanvas" title="New Canvas">
        <FilePlus :size="16" />
      </LButton>
    </div>

    <div class="flex-1"></div>

    <div class="flex gap-2 items-center">
      <LBadge>Time: {{ timeRangeLabel }}</LBadge>
    </div>
  </header>
</template>
