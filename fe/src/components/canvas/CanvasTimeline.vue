<script setup lang="ts">
import { computed, watch } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LCheckbox, LTimelineScrubber } from '@/components/ui'

const canvasStore = useCanvasStore()

// Preset positions as offsets from now (in ms)
const presetOffsets = {
  '1h': 60 * 60 * 1000,
  '3h': 3 * 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '12h': 12 * 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
  '7d': 7 * 24 * 60 * 60 * 1000,
  '30d': 30 * 24 * 60 * 60 * 1000,
  '90d': 90 * 24 * 60 * 60 * 1000,
}

// Use logarithmic scale for better distribution
// Map: 0 = 1 minute ago, 100 = 90 days ago
const MIN_MS = 60 * 1000 // 1 minute
const MAX_MS = 90 * 24 * 60 * 60 * 1000 // 90 days
const LOG_MIN = Math.log(MIN_MS)
const LOG_MAX = Math.log(MAX_MS)

// Convert offset (ms) to percentage using log scale (100 = now/min, 0 = 90d)
function offsetToPercent(offsetMs: number): number {
  if (offsetMs <= MIN_MS) return 100
  if (offsetMs >= MAX_MS) return 0
  const logValue = Math.log(offsetMs)
  return 100 - ((logValue - LOG_MIN) / (LOG_MAX - LOG_MIN)) * 100
}

// Convert percentage to offset (ms) using log scale
function percentToOffset(percent: number): number {
  if (percent >= 100) return 0
  if (percent <= 0) return MAX_MS
  const logValue = LOG_MIN + ((100 - percent) / 100) * (LOG_MAX - LOG_MIN)
  return Math.exp(logValue)
}

// Create markers for the presets (with log scale positions)
const timelineMarkers = computed(() => [
  { position: offsetToPercent(presetOffsets['1h']), label: '1h', color: 'primary' as const },
  { position: offsetToPercent(presetOffsets['6h']), label: '6h', color: 'primary' as const },
  { position: offsetToPercent(presetOffsets['24h']), label: '24h', color: 'primary' as const },
  { position: offsetToPercent(presetOffsets['7d']), label: '7d', color: 'primary' as const },
  { position: offsetToPercent(presetOffsets['30d']), label: '30d', color: 'primary' as const },
])

// Current scrubber position (computed from store)
const scrubPosition = computed({
  get: () => {
    const offset = canvasStore.activeCanvas?.timeRange.offset ?? 0
    return offsetToPercent(offset)
  },
  set: (percent: number) => {
    const offsetMs = percentToOffset(percent)
    // Snap to "now" if very close
    canvasStore.setTimeOffset(offsetMs < MIN_MS ? 0 : offsetMs)
  },
})

// Format the scrubber position as a relative time label
function formatScrubValue(percent: number): string {
  if (percent >= 99) return 'Now'
  const offsetMs = percentToOffset(percent)

  if (offsetMs < 60 * 1000) return `${Math.round(offsetMs / 1000)}s ago`
  if (offsetMs < 60 * 60 * 1000) return `${Math.round(offsetMs / 60000)}m ago`
  if (offsetMs < 24 * 60 * 60 * 1000) {
    const hours = offsetMs / 3600000
    if (hours < 10) {
      return Number.isInteger(hours) ? `${Math.round(hours)}h ago` : `${hours.toFixed(1)}h ago`
    } else {
      return `${Math.round(hours)}h ago`
    }
  }
  const days = offsetMs / 86400000
  if (days < 10) {
    return Number.isInteger(days) ? `${Math.round(days)}d ago` : `${days.toFixed(1)}d ago`
  } else {
    return `${Math.round(days)}d ago`
  }
}

// Current time window label (compact)
const timeWindowLabel = computed(() => {
  const offset = canvasStore.activeCanvas?.timeRange.offset ?? 0
  if (offset < MIN_MS) return 'Now'

  if (offset < 60 * 60 * 1000) return `${Math.round(offset / 60000)}m`
  if (offset < 24 * 60 * 60 * 1000) {
    const hours = offset / 3600000
    return hours < 10 ? `${hours.toFixed(1)}h` : `${Math.round(hours)}h`
  }
  const days = offset / 86400000
  return days < 10 ? `${days.toFixed(1)}d` : `${Math.round(days)}d`
})

// Check if scrubber should be disabled
const isScrubberDisabled = computed(() => {
  return canvasStore.activeCanvas?.live ?? false
})

function toggleLive(value: boolean) {
  canvasStore.setLive(value)
}

// When going live, snap to "Now"
watch(
  () => canvasStore.activeCanvas?.live,
  (live) => {
    if (live) {
      canvasStore.setTimeOffset(0)
    }
  },
)

// Handle marker clicks - snap to preset
function handleMarkerClick(marker: { position: number; label?: string }) {
  if (marker.label && marker.label in presetOffsets) {
    const offset = presetOffsets[marker.label as keyof typeof presetOffsets]
    canvasStore.setTimeOffset(offset)
  }
}
</script>

<template>
  <section class="flex items-center gap-4 px-4 py-2 border-b border-border bg-surface-sunken">
    <!-- Label + current value -->
    <div class="flex items-center gap-3 shrink-0">
      <span class="text-sm font-medium text-text">Timeline</span>
      <span
        class="px-2 py-0.5 rounded text-xs font-medium min-w-12 text-center"
        :class="
          isScrubberDisabled ? 'bg-success/15 text-success' : 'bg-primary-500/15 text-primary-500'
        "
      >
        {{ isScrubberDisabled ? 'Live' : timeWindowLabel }}
      </span>
    </div>

    <!-- Scrubber -->
    <div class="flex-1 flex items-center gap-2 min-w-0">
      <span class="text-[10px] text-text-subtle shrink-0">90d</span>
      <LTimelineScrubber
        v-model="scrubPosition"
        :min="0"
        :max="100"
        :step="0.5"
        :markers="timelineMarkers"
        :disabled="isScrubberDisabled"
        :format-value="formatScrubValue"
        size="sm"
        marker-snap
        marker-labels
        class="flex-1"
        @markerClick="handleMarkerClick"
      />
      <span class="text-[10px] text-text-subtle shrink-0">Now</span>
    </div>

    <!-- Live toggle -->
    <label
      class="flex items-center gap-1.5 text-xs cursor-pointer shrink-0"
      :class="canvasStore.activeCanvas?.live ? 'text-success' : 'text-text-muted'"
    >
      <LCheckbox
        :model-value="canvasStore.activeCanvas?.live ?? false"
        @update:model-value="toggleLive"
      />
      <span class="flex items-center gap-1">
        <span v-if="canvasStore.activeCanvas?.live" class="relative flex h-1.5 w-1.5">
          <span
            class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"
          ></span>
          <span class="relative inline-flex rounded-full h-1.5 w-1.5 bg-success"></span>
        </span>
        Live
      </span>
    </label>
  </section>
</template>
