<script setup lang="ts">
import { computed } from 'vue'
import type { VisualizationConfig, QueryResult } from '@/types'

const props = defineProps<{
  data: QueryResult
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

const value = computed(() => {
  if (!props.data || !props.data.rows.length) return null

  const valueColumn = props.config.value_column
  if (!valueColumn) {
    // Default to first numeric column if not specified
    return props.data.rows[0]?.[0] ?? null
  }

  const idx = props.data.columns.findIndex((c) => c.name === valueColumn)
  if (idx === -1) return null

  return props.data.rows[0]?.[idx] ?? null
})

const formattedValue = computed(() => {
  if (value.value === null) return '-'

  const val = Number(value.value)
  if (isNaN(val)) return String(value.value)

  // Format large numbers
  if (Math.abs(val) >= 1_000_000) {
    return (val / 1_000_000).toFixed(1) + 'M'
  }
  if (Math.abs(val) >= 1_000) {
    return (val / 1_000).toFixed(1) + 'K'
  }
  if (Number.isInteger(val)) {
    return val.toLocaleString()
  }
  return val.toFixed(2)
})

const thresholdColor = computed(() => {
  if (!props.config.thresholds || !value.value) return null

  const val = Number(value.value)
  if (isNaN(val)) return null

  // Sort thresholds by value descending
  const sorted = [...props.config.thresholds].sort((a, b) => b.value - a.value)

  for (const threshold of sorted) {
    if (val >= threshold.value) {
      return threshold.color
    }
  }
  return null
})
</script>

<template>
  <div
    class="flex flex-col items-center justify-center h-full min-h-32"
    :style="{ height: height || '200px' }"
  >
    <div v-if="loading" class="animate-pulse">
      <div class="h-12 w-24 bg-surface-sunken rounded" />
    </div>

    <template v-else>
      <div class="text-center">
        <div class="flex items-baseline justify-center gap-1">
          <span v-if="config.prefix" class="text-2xl text-text-muted">{{ config.prefix }}</span>
          <span
            class="text-5xl font-semibold tabular-nums"
            :style="{ color: thresholdColor || undefined }"
            :class="{ 'text-text': !thresholdColor }"
          >
            {{ formattedValue }}
          </span>
          <span v-if="config.suffix" class="text-2xl text-text-muted">{{ config.suffix }}</span>
        </div>
      </div>
    </template>
  </div>
</template>
