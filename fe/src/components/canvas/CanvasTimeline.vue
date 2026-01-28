<script setup lang="ts">
import { useCanvasStore } from '@/stores/canvas'
import { LButton, LCheckbox } from '@/components/ui'
import type { TimePreset } from '@/types/canvas'

const canvasStore = useCanvasStore()

const timePresets: { value: TimePreset; label: string }[] = [
  { value: 'now', label: 'Now' },
  { value: '24h', label: '24h' },
  { value: '7d', label: '7d' },
  { value: '30d', label: '30d' },
  { value: '90d', label: '90d' },
]

function setTimePreset(preset: TimePreset) {
  canvasStore.setTimePreset(preset)
}

function toggleLive(value: boolean) {
  canvasStore.setLive(value)
}
</script>

<template>
  <section
    class="grid grid-cols-[240px_1fr_120px] items-center gap-3 px-4 border-b border-border bg-surface-sunken"
  >
    <div class="flex items-center gap-2">
      <div class="flex flex-col gap-0.5">
        <div class="font-semibold text-sm">Timeline</div>
        <div class="text-xs text-text-muted">Global time window for all queries</div>
      </div>
    </div>

    <div class="flex justify-center">
      <div class="flex gap-1.5">
        <LButton
          v-for="p in timePresets"
          :key="p.value"
          :variant="canvasStore.activeCanvas?.timeRange.preset === p.value ? 'primary' : 'secondary'"
          size="sm"
          @click="setTimePreset(p.value)"
        >
          {{ p.label }}
        </LButton>
      </div>
    </div>

    <div class="flex justify-end items-center">
      <label class="inline-flex gap-2 items-center text-xs text-text-muted cursor-pointer">
        <LCheckbox
          :model-value="canvasStore.activeCanvas?.live ?? false"
          @update:model-value="toggleLive"
        />
        Live
      </label>
    </div>
  </section>
</template>
