<script setup lang="ts">
import { computed, ref, onUnmounted, watch } from 'vue'
import { formatDate, formatDateLike, formatDateTime } from '@/utils/dateTime'
import type { VisualizationConfig, QueryResult } from '@/types'

const ROW_HEIGHT = 36
const HEADER_HEIGHT = 52
const BUFFER_ROWS = 10
const VIRTUAL_THRESHOLD = 100

const props = defineProps<{
  data: QueryResult
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

// Virtual scroll state
const scrollContainer = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const measuredHeight = ref(400)

// Should use virtual scrolling
const useVirtualScroll = computed(() => (props.data?.rows?.length ?? 0) > VIRTUAL_THRESHOLD)

// Use measured height from actual container
const containerHeight = computed(() => measuredHeight.value)

// Measure container height
function updateContainerHeight() {
  if (scrollContainer.value) {
    const height = scrollContainer.value.clientHeight
    if (height > 0) {
      measuredHeight.value = height
    }
  }
}

// Total content height (rows only, header is sticky)
const totalRowsHeight = computed(() => (props.data?.rows?.length ?? 0) * ROW_HEIGHT)

// Calculate visible range
const visibleRange = computed(() => {
  if (!useVirtualScroll.value) {
    return { start: 0, end: props.data?.rows?.length ?? 0 }
  }

  // Account for header in scroll position
  const adjustedScroll = Math.max(0, scrollTop.value)
  const start = Math.max(0, Math.floor(adjustedScroll / ROW_HEIGHT) - BUFFER_ROWS)
  const visibleCount = Math.ceil(containerHeight.value / ROW_HEIGHT)
  const end = Math.min(props.data?.rows?.length ?? 0, start + visibleCount + BUFFER_ROWS * 2)

  return { start, end }
})

// Visible rows with indices
const visibleRows = computed(() => {
  if (!props.data?.rows) return []
  const { start, end } = visibleRange.value
  return props.data.rows.slice(start, end).map((row, i) => ({
    row,
    index: start + i,
  }))
})

// Offset for positioning
const offsetY = computed(() => visibleRange.value.start * ROW_HEIGHT)

// Columns with config
const columns = computed(() => {
  if (!props.data?.columns?.length) return []

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

// Scroll handling
let rafId: number | null = null

function handleScroll(e: Event) {
  if (rafId !== null) return
  rafId = requestAnimationFrame(() => {
    const target = e.target as HTMLElement
    scrollTop.value = target.scrollTop
    rafId = null
  })
}

watch(
  () => props.data?.rows?.length,
  (newLen, oldLen) => {
    if (newLen !== oldLen && scrollContainer.value) {
      scrollContainer.value.scrollTop = 0
      scrollTop.value = 0
    }
  },
)

// ResizeObserver for container height changes
let resizeObserver: ResizeObserver | null = null
let currentContainer: HTMLElement | null = null

function setupScrollContainer(el: HTMLElement | null) {
  // Cleanup previous
  if (currentContainer) {
    currentContainer.removeEventListener('scroll', handleScroll)
    resizeObserver?.disconnect()
  }

  currentContainer = el

  if (el) {
    el.addEventListener('scroll', handleScroll, { passive: true })
    updateContainerHeight()

    resizeObserver = new ResizeObserver(() => {
      updateContainerHeight()
    })
    resizeObserver.observe(el)
  }
}

// Watch for scrollContainer ref changes (happens when v-if toggles)
watch(scrollContainer, (el) => {
  setupScrollContainer(el)
}, { immediate: true })

onUnmounted(() => {
  if (rafId !== null) cancelAnimationFrame(rafId)
  setupScrollContainer(null)
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <!-- Loading -->
    <div v-if="loading" class="p-4 space-y-3">
      <div v-for="i in 5" :key="i" class="h-10 bg-surface-sunken rounded animate-pulse" />
    </div>

    <!-- Table -->
    <div
      v-else-if="data?.rows?.length"
      ref="scrollContainer"
      class="flex-1 overflow-auto min-h-0"
    >
      <table class="w-full min-w-max text-sm">
        <!-- Sticky header -->
        <thead class="bg-surface-sunken sticky top-0 z-10">
          <tr>
            <th
              v-for="col in columns"
              :key="col.key"
              class="px-4 py-2 font-medium text-text-muted border-b border-border whitespace-nowrap"
              :class="{
                'text-left': col.align === 'left',
                'text-center': col.align === 'center',
                'text-right': col.align === 'right',
              }"
              :style="col.width ? { width: col.width + 'px', minWidth: col.width + 'px' } : undefined"
            >
              <div class="flex flex-col">
                <span>{{ col.label }}</span>
                <span class="text-[10px] font-normal text-text-subtle">{{ col.data_type }}</span>
              </div>
            </th>
          </tr>
        </thead>

        <!-- Virtual scroll body -->
        <tbody v-if="useVirtualScroll">
          <!-- Top spacer -->
          <tr v-if="offsetY > 0" aria-hidden="true">
            <td :colspan="columns.length" :style="{ height: `${offsetY}px`, padding: 0 }"></td>
          </tr>
          <!-- Visible rows -->
          <tr
            v-for="{ row, index: rowIdx } in visibleRows"
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
              :style="{ height: `${ROW_HEIGHT}px` }"
            >
              <div class="truncate">
                <span v-if="row[colIdx] === null" class="text-text-subtle italic">null</span>
                <span v-else>{{ formatValue(row[colIdx], col.format, col.data_type) }}</span>
              </div>
            </td>
          </tr>
          <!-- Bottom spacer -->
          <tr v-if="totalRowsHeight - offsetY - (visibleRows.length * ROW_HEIGHT) > 0" aria-hidden="true">
            <td
              :colspan="columns.length"
              :style="{ height: `${totalRowsHeight - offsetY - (visibleRows.length * ROW_HEIGHT)}px`, padding: 0 }"
            ></td>
          </tr>
        </tbody>

        <!-- Regular body (small datasets) -->
        <tbody v-else>
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
    </div>

    <!-- Empty state -->
    <div v-else class="p-8 text-center flex-1 flex items-center justify-center">
      <p class="text-text-muted">No data to display</p>
    </div>
  </div>
</template>
