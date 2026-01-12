<script setup lang="ts">
import { computed } from 'vue'
import { formatDate, formatDateLike, formatDateTime } from '@/utils/dateTime'
import type { VisualizationConfig, QueryResult } from '@/types'

const props = defineProps<{
  data: QueryResult
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

// Merge column config with actual columns
const columns = computed(() => {
  if (!props.data) return []

  return props.data.columns.map((col) => {
    const configCol = props.config.columns?.find((c) => c.key === col.name)
    return {
      key: col.name,
      label: configCol?.label || col.name,
      width: configCol?.width,
      align: configCol?.align || 'left',
      format: configCol?.format,
      data_type: col.data_type,
    }
  })
})

function formatValue(value: unknown, format?: string, dataType?: string): string {
  if (value === null) return ''
  if (value === undefined) return ''

  if (format) {
    // Simple format patterns
    if (format === 'number' && typeof value === 'number') {
      return value.toLocaleString()
    }
    if (format === 'currency' && typeof value === 'number') {
      return '$' + value.toLocaleString(undefined, { minimumFractionDigits: 2 })
    }
    if (format === 'percent' && typeof value === 'number') {
      return (value * 100).toFixed(1) + '%'
    }
    if (format === 'date') {
      return formatDate(value)
    }
    if (format === 'datetime') {
      return formatDateTime(value)
    }
  }

  const formattedByType = formatDateLike(value, dataType)
  if (formattedByType !== null) {
    return formattedByType
  }

  return String(value)
}
</script>

<template>
  <div class="overflow-auto" :style="{ maxHeight: height || '400px' }">
    <!-- Loading skeleton -->
    <div v-if="loading" class="p-4 space-y-3">
      <div v-for="i in 5" :key="i" class="h-10 bg-surface-sunken rounded animate-pulse" />
    </div>

    <!-- Table -->
    <table v-else-if="data && data.rows.length > 0" class="w-full text-sm">
      <thead class="bg-surface-sunken sticky top-0 z-10">
        <tr>
          <th
            v-for="col in columns"
            :key="col.key"
            class="px-4 py-3 font-medium text-text-muted border-b border-border whitespace-nowrap"
            :class="{
              'text-left': col.align === 'left',
              'text-center': col.align === 'center',
              'text-right': col.align === 'right',
            }"
            :style="col.width ? { width: col.width + 'px' } : undefined"
          >
            <div class="flex flex-col">
              <span>{{ col.label }}</span>
              <span class="text-xs font-normal text-text-subtle">{{ col.data_type }}</span>
            </div>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(row, rowIdx) in data.rows"
          :key="rowIdx"
          class="border-b border-border hover:bg-surface-sunken/50 transition-colors"
        >
          <td
            v-for="(col, colIdx) in columns"
            :key="col.key"
            class="px-4 py-2 text-text"
            :class="{
              'text-left': col.align === 'left',
              'text-center': col.align === 'center',
              'text-right': col.align === 'right',
            }"
          >
            <span v-if="row[colIdx] === null" class="text-text-subtle italic">null</span>
            <span v-else>{{ formatValue(row[colIdx], col.format, col.data_type) }}</span>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- Empty state -->
    <div v-else class="p-8 text-center">
      <p class="text-text-muted">No data to display</p>
    </div>
  </div>
</template>
