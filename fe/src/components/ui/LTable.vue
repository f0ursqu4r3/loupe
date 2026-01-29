<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch, type HTMLAttributes } from 'vue'

interface Props {
  headers: string[]
  rows: unknown[][]
  striped?: boolean
  hoverable?: boolean
  compact?: boolean
  class?: HTMLAttributes['class']
  // Virtual scrolling props
  virtualScroll?: boolean
  rowHeight?: number
  maxHeight?: number | string
  bufferRows?: number
}

const props = withDefaults(defineProps<Props>(), {
  striped: false,
  hoverable: true,
  compact: false,
  virtualScroll: false,
  rowHeight: 40,
  maxHeight: 400,
  bufferRows: 5,
})

const tableClasses = computed(() => ['w-full text-left', props.class])
const cellPadding = computed(() => (props.compact ? 'px-3 py-2' : 'px-4 py-3'))

// Virtual scroll state
const scrollContainer = ref<HTMLElement | null>(null)
const scrollTop = ref(0)

// Compute row height based on compact mode if not explicitly set
const effectiveRowHeight = computed(() => (props.compact ? 36 : props.rowHeight))

// Format maxHeight for CSS
const maxHeightStyle = computed(() => {
  if (typeof props.maxHeight === 'number') return `${props.maxHeight}px`
  return props.maxHeight
})

// Total height of all rows (for scroll spacer)
const totalHeight = computed(() => props.rows.length * effectiveRowHeight.value)

// Calculate visible range
const visibleRange = computed(() => {
  if (!props.virtualScroll) {
    return { start: 0, end: props.rows.length }
  }

  const containerHeight =
    typeof props.maxHeight === 'number' ? props.maxHeight : parseInt(props.maxHeight) || 400

  const start = Math.max(0, Math.floor(scrollTop.value / effectiveRowHeight.value) - props.bufferRows)
  const visibleCount = Math.ceil(containerHeight / effectiveRowHeight.value)
  const end = Math.min(props.rows.length, start + visibleCount + props.bufferRows * 2)

  return { start, end }
})

// Get visible rows with their original indices
const visibleRows = computed(() => {
  const { start, end } = visibleRange.value
  return props.rows.slice(start, end).map((row, i) => ({
    row,
    index: start + i,
  }))
})

// Offset for positioning visible rows
const offsetY = computed(() => visibleRange.value.start * effectiveRowHeight.value)

// Handle scroll events
let rafId: number | null = null

function handleScroll(e: Event) {
  if (rafId !== null) return

  rafId = requestAnimationFrame(() => {
    const target = e.target as HTMLElement
    scrollTop.value = target.scrollTop
    rafId = null
  })
}

// Reset scroll position when rows change significantly
watch(
  () => props.rows.length,
  (newLen, oldLen) => {
    if (Math.abs(newLen - oldLen) > 100 && scrollContainer.value) {
      scrollContainer.value.scrollTop = 0
      scrollTop.value = 0
    }
  },
)

onMounted(() => {
  if (props.virtualScroll && scrollContainer.value) {
    scrollContainer.value.addEventListener('scroll', handleScroll, { passive: true })
  }
})

onUnmounted(() => {
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  if (scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', handleScroll)
  }
})
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-border">
    <!-- Non-virtual scroll mode (original behavior) -->
    <div v-if="!virtualScroll" class="overflow-x-auto">
      <table :class="tableClasses">
        <thead class="bg-surface-sunken border-b border-border">
          <tr>
            <th
              v-for="(header, index) in headers"
              :key="index"
              :class="[cellPadding, 'text-sm font-medium text-text-muted whitespace-nowrap']"
            >
              {{ header }}
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-border">
          <tr
            v-for="(row, rowIndex) in rows"
            :key="rowIndex"
            :class="[
              striped && rowIndex % 2 === 1 && 'bg-surface-sunken/50',
              hoverable && 'hover:bg-surface-sunken/70 transition-colors',
            ]"
          >
            <td
              v-for="(cell, cellIndex) in row"
              :key="cellIndex"
              :class="[cellPadding, 'text-sm text-text']"
            >
              <slot :name="`cell-${cellIndex}`" :value="cell" :row="row" :rowIndex="rowIndex">
                {{ cell }}
              </slot>
            </td>
          </tr>
          <tr v-if="rows.length === 0">
            <td
              :colspan="headers.length"
              :class="[cellPadding, 'text-sm text-text-muted text-center']"
            >
              <slot name="empty">No data available</slot>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Virtual scroll mode -->
    <template v-else>
      <!-- Fixed header -->
      <div class="overflow-hidden bg-surface-sunken border-b border-border">
        <table :class="tableClasses">
          <thead>
            <tr>
              <th
                v-for="(header, index) in headers"
                :key="index"
                :class="[cellPadding, 'text-sm font-medium text-text-muted whitespace-nowrap']"
              >
                {{ header }}
              </th>
            </tr>
          </thead>
        </table>
      </div>

      <!-- Scrollable body -->
      <div
        ref="scrollContainer"
        class="overflow-auto"
        :style="{ maxHeight: maxHeightStyle }"
      >
        <!-- Spacer to maintain scroll height -->
        <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
          <!-- Visible rows container -->
          <table
            :class="tableClasses"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              transform: `translateY(${offsetY}px)`,
            }"
          >
            <tbody>
              <tr
                v-for="{ row, index: rowIndex } in visibleRows"
                :key="rowIndex"
                :class="[
                  striped && rowIndex % 2 === 1 && 'bg-surface-sunken/50',
                  hoverable && 'hover:bg-surface-sunken/70 transition-colors',
                ]"
                :style="{ height: `${effectiveRowHeight}px` }"
              >
                <td
                  v-for="(cell, cellIndex) in row"
                  :key="cellIndex"
                  :class="[cellPadding, 'text-sm text-text truncate']"
                >
                  <slot :name="`cell-${cellIndex}`" :value="cell" :row="row" :rowIndex="rowIndex">
                    {{ cell }}
                  </slot>
                </td>
              </tr>
              <tr v-if="rows.length === 0">
                <td
                  :colspan="headers.length"
                  :class="[cellPadding, 'text-sm text-text-muted text-center']"
                >
                  <slot name="empty">No data available</slot>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>
