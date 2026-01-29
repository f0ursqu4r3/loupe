<script setup lang="ts">
import { computed } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LSelect, LTimelineScrubber } from '@/components/ui'
import type { TimePreset } from '@/types/canvas'

const LABEL_TO_PRESET: Partial<Record<(typeof presets)[number]['label'], TimePreset>> = {
  '1h': '1h',
  '6h': '6h',
  '1d': '24h',
  '7d': '7d',
  '1mo': '30d',
  Now: 'now',
}

const canvasStore = useCanvasStore()

// Minimum offset to consider as "now"
const MIN_MS = 60 * 1000 // 1 minute

// Define presets with evenly-spaced positions (percent) and their time values (ms)
// Ordered from left (oldest) to right (newest/now)
// values: 1y, 6m, 1m, 7d, 1d, 6h, 1h, Now
const presets = [
  { percent: 0, ms: 365 * 24 * 60 * 60 * 1000, label: '1yr' }, // 1 year
  { percent: 14.3, ms: 182 * 24 * 60 * 60 * 1000, label: '6mo' }, // 6 months (approx)
  { percent: 28.6, ms: 30 * 24 * 60 * 60 * 1000, label: '1mo' }, // 1 month
  { percent: 42.9, ms: 7 * 24 * 60 * 60 * 1000, label: '7d' }, // 7 days
  { percent: 57.2, ms: 24 * 60 * 60 * 1000, label: '1d' }, // 1 day
  { percent: 71.5, ms: 6 * 60 * 60 * 1000, label: '6h' }, // 6 hours
  { percent: 85.8, ms: 60 * 60 * 1000, label: '1h' }, // 1 hour
  { percent: 100, ms: 0, label: 'Now' }, // Now
] as const

// Convert offset (ms) to percentage using piecewise linear interpolation
function offsetToPercent(offsetMs: number): number {
  if (offsetMs <= 0) return 100
  if (offsetMs >= presets[0].ms) return 0

  // Find the two presets we're between (presets are ordered by percent ascending, ms descending)
  for (let i = 0; i < presets.length - 1; i++) {
    const left = presets[i]!
    const right = presets[i + 1]!
    if (offsetMs <= left.ms && offsetMs >= right.ms) {
      // Linear interpolation between these two presets
      const ratio = (left.ms - offsetMs) / (left.ms - right.ms)
      return left.percent + ratio * (right.percent - left.percent)
    }
  }
  return 100
}

// Convert percentage to offset (ms) using piecewise linear interpolation
function percentToOffset(percent: number): number {
  if (percent >= 100) return 0
  if (percent <= 0) return presets[0].ms

  // Find the two presets we're between
  for (let i = 0; i < presets.length - 1; i++) {
    const left = presets[i]!
    const right = presets[i + 1]!
    if (percent >= left.percent && percent <= right.percent) {
      // Linear interpolation between these two presets
      const ratio = (percent - left.percent) / (right.percent - left.percent)
      return left.ms - ratio * (left.ms - right.ms)
    }
  }
  return 0
}

// Create markers for the presets (evenly spaced, excluding 90d and Now which are at edges)
const timelineMarkers = computed(() =>
  presets
    .filter((p) => p.label !== '90d' && p.label !== 'Now')
    .map((p) => ({ position: p.percent, label: p.label, color: 'primary' as const })),
)

// Current scrubber position (computed from store)
const scrubPosition = computed({
  get: () => {
    const offset = canvasStore.activeCanvas?.timeRange?.offset ?? 0
    return offsetToPercent(offset)
  },
  set: (percent: number) => {
    if (!canvasStore.activeCanvas) return
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
  const offset = canvasStore.activeCanvas?.timeRange?.offset ?? 0
  if (offset < MIN_MS) return 'Now'

  if (offset < 60 * 60 * 1000) return `${Math.round(offset / 60000)}m`
  if (offset < 24 * 60 * 60 * 1000) return `${Math.round(offset / 3600000)}h`
  return `${Math.round(offset / 86400000)}d`
})

// Refresh interval options for the Live dropdown
const REFRESH_OPTIONS = [
  { value: 0, label: 'Off' },
  { value: 5000, label: '5s' },
  { value: 10000, label: '10s' },
  { value: 30000, label: '30s' },
  { value: 60000, label: '1m' },
  { value: 300000, label: '5m' },
]

// Check if scrubber should be disabled (any refresh interval is active)
const isScrubberDisabled = computed(() => canvasStore.isLive)

function onRefreshIntervalChange(value: string | number) {
  canvasStore.setRefreshInterval(Number(value))
}

// Handle marker clicks - snap to preset
function handleMarkerClick(marker: { position: number; label?: string }) {
  const match = presets.find((p) => p.label === marker.label)
  if (!match) return
  const namedPreset = marker.label ? LABEL_TO_PRESET[marker.label as keyof typeof LABEL_TO_PRESET] : undefined
  if (namedPreset) {
    canvasStore.setTimePreset(namedPreset)
  } else {
    canvasStore.setTimeOffset(match.ms)
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

    <!-- Refresh interval -->
    <div class="flex items-center gap-1.5 shrink-0">
      <span v-if="isScrubberDisabled" class="relative flex h-1.5 w-1.5">
        <span
          class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"
        ></span>
        <span class="relative inline-flex rounded-full h-1.5 w-1.5 bg-success"></span>
      </span>
      <LSelect
        :model-value="canvasStore.activeCanvas?.refreshInterval ?? 0"
        :options="REFRESH_OPTIONS"
        size="xs"
        @update:model-value="onRefreshIntervalChange"
      />
    </div>
  </section>
</template>
