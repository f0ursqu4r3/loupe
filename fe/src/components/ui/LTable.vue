<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch, type HTMLAttributes } from 'vue'
import { ChevronUp, ChevronDown, ChevronsUpDown } from 'lucide-vue-next'
import LCheckbox from './LCheckbox.vue'

type SortDirection = 'asc' | 'desc' | null

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
  // Sorting props
  sortable?: boolean
  sortBy?: number
  sortDirection?: SortDirection
  // Selection props
  selectable?: boolean
  selectedRows?: number[]
}

const props = withDefaults(defineProps<Props>(), {
  striped: false,
  hoverable: true,
  compact: false,
  virtualScroll: false,
  rowHeight: 40,
  maxHeight: 400,
  bufferRows: 5,
  sortable: false,
  sortBy: undefined,
  sortDirection: null,
  selectable: false,
  selectedRows: () => [],
})

const emit = defineEmits<{
  sort: [column: number, direction: SortDirection]
  'update:selectedRows': [rows: number[]]
}>()

// Local sort state
const localSortBy = ref<number | undefined>(props.sortBy)
const localSortDirection = ref<SortDirection>(props.sortDirection)

// Watch for external sort changes
watch(
  () => [props.sortBy, props.sortDirection],
  ([newSortBy, newDirection]) => {
    localSortBy.value = newSortBy as number | undefined
    localSortDirection.value = newDirection as SortDirection
  },
)

// Handle column header click for sorting
function handleSort(columnIndex: number) {
  if (!props.sortable) return

  let newDirection: SortDirection
  if (localSortBy.value === columnIndex) {
    // Cycle through: null -> asc -> desc -> null
    if (localSortDirection.value === null) newDirection = 'asc'
    else if (localSortDirection.value === 'asc') newDirection = 'desc'
    else newDirection = null
  } else {
    newDirection = 'asc'
  }

  localSortBy.value = newDirection === null ? undefined : columnIndex
  localSortDirection.value = newDirection
  emit('sort', columnIndex, newDirection)
}

// Get sort icon for column
function getSortIcon(columnIndex: number) {
  if (localSortBy.value !== columnIndex) return ChevronsUpDown
  return localSortDirection.value === 'asc' ? ChevronUp : ChevronDown
}

// Selection state
const localSelectedRows = ref<Set<number>>(new Set(props.selectedRows))

watch(
  () => props.selectedRows,
  (newSelected) => {
    localSelectedRows.value = new Set(newSelected)
  },
)

// Check if all visible rows are selected
const allSelected = computed(() => {
  if (props.rows.length === 0) return false
  return props.rows.every((_, index) => localSelectedRows.value.has(index))
})

const someSelected = computed(() => {
  return localSelectedRows.value.size > 0 && !allSelected.value
})

// Toggle all rows selection
function toggleAllRows() {
  if (allSelected.value) {
    localSelectedRows.value.clear()
  } else {
    props.rows.forEach((_, index) => localSelectedRows.value.add(index))
  }
  emit('update:selectedRows', Array.from(localSelectedRows.value))
}

// Toggle single row selection
function toggleRow(rowIndex: number) {
  if (localSelectedRows.value.has(rowIndex)) {
    localSelectedRows.value.delete(rowIndex)
  } else {
    localSelectedRows.value.add(rowIndex)
  }
  emit('update:selectedRows', Array.from(localSelectedRows.value))
}

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

  const start = Math.max(
    0,
    Math.floor(scrollTop.value / effectiveRowHeight.value) - props.bufferRows,
  )
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
            <!-- Selection checkbox column -->
            <th v-if="selectable" :class="[cellPadding, 'w-12']">
              <LCheckbox
                :size="compact ? 'xs' : 'sm'"
                :model-value="allSelected"
                :indeterminate="someSelected"
                aria-label="Select all rows"
                @update:model-value="toggleAllRows"
              />
            </th>
            <!-- Regular header columns -->
            <th
              v-for="(header, index) in headers"
              :key="index"
              :class="[
                cellPadding,
                'text-sm font-medium text-text-muted whitespace-nowrap',
                sortable && 'cursor-pointer select-none hover:text-text transition-colors',
              ]"
              @click="handleSort(index)"
            >
              <div class="flex items-center gap-2">
                <span>{{ header }}</span>
                <component
                  v-if="sortable"
                  :is="getSortIcon(index)"
                  :size="16"
                  :class="[
                    'shrink-0',
                    localSortBy === index ? 'text-primary-600' : 'text-text-subtle',
                  ]"
                />
              </div>
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
              selectable &&
                localSelectedRows.has(rowIndex) &&
                'bg-primary-50 dark:bg-primary-900/20',
            ]"
          >
            <!-- Selection checkbox -->
            <td v-if="selectable" :class="[cellPadding, 'w-12']">
              <LCheckbox
                :size="compact ? 'xs' : 'sm'"
                :model-value="localSelectedRows.has(rowIndex)"
                :aria-label="`Select row ${rowIndex + 1}`"
                @update:model-value="toggleRow(rowIndex)"
              />
            </td>
            <!-- Regular cells -->
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
              :colspan="headers.length + (selectable ? 1 : 0)"
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
              <!-- Selection checkbox column -->
              <th v-if="selectable" :class="[cellPadding, 'w-12']">
                <LCheckbox
                  :size="compact ? 'xs' : 'sm'"
                  :model-value="allSelected"
                  :indeterminate="someSelected"
                  aria-label="Select all rows"
                  @update:model-value="toggleAllRows"
                />
              </th>
              <!-- Regular header columns -->
              <th
                v-for="(header, index) in headers"
                :key="index"
                :class="[
                  cellPadding,
                  'text-sm font-medium text-text-muted whitespace-nowrap',
                  sortable && 'cursor-pointer select-none hover:text-text transition-colors',
                ]"
                @click="handleSort(index)"
              >
                <div class="flex items-center gap-2">
                  <span>{{ header }}</span>
                  <component
                    v-if="sortable"
                    :is="getSortIcon(index)"
                    :size="16"
                    :class="[
                      'shrink-0',
                      localSortBy === index ? 'text-primary-600' : 'text-text-subtle',
                    ]"
                  />
                </div>
              </th>
            </tr>
          </thead>
        </table>
      </div>

      <!-- Scrollable body -->
      <div ref="scrollContainer" class="overflow-auto" :style="{ maxHeight: maxHeightStyle }">
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
                  selectable &&
                    localSelectedRows.has(rowIndex) &&
                    'bg-primary-50 dark:bg-primary-900/20',
                ]"
                :style="{ height: `${effectiveRowHeight}px` }"
              >
                <!-- Selection checkbox -->
                <td v-if="selectable" :class="[cellPadding, 'w-12']">
                  <LCheckbox
                    :size="compact ? 'xs' : 'sm'"
                    :model-value="localSelectedRows.has(rowIndex)"
                    :aria-label="`Select row ${rowIndex + 1}`"
                    @update:model-value="toggleRow(rowIndex)"
                  />
                </td>
                <!-- Regular cells -->
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
                  :colspan="headers.length + (selectable ? 1 : 0)"
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
