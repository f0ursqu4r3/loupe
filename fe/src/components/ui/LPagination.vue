<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-vue-next'

interface Props {
  currentPage?: number
  totalPages?: number
  totalItems?: number
  pageSize?: number
  pageSizeOptions?: number[]
  showPageSize?: boolean
  showTotal?: boolean
  class?: HTMLAttributes['class']
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  currentPage: 1,
  totalPages: 1,
  totalItems: 0,
  pageSize: 10,
  pageSizeOptions: () => [10, 25, 50, 100],
  showPageSize: true,
  showTotal: true,
  disabled: false,
})

const emit = defineEmits<{
  'update:currentPage': [page: number]
  'update:pageSize': [size: number]
}>()

// Calculate page numbers to display
const pageNumbers = computed(() => {
  const pages: (number | string)[] = []
  const total = props.totalPages
  const current = props.currentPage

  if (total <= 7) {
    // Show all pages if 7 or fewer
    for (let i = 1; i <= total; i++) {
      pages.push(i)
    }
  } else {
    // Always show first page
    pages.push(1)

    if (current <= 3) {
      // Near the start
      pages.push(2, 3, 4, '...', total)
    } else if (current >= total - 2) {
      // Near the end
      pages.push('...', total - 3, total - 2, total - 1, total)
    } else {
      // In the middle
      pages.push('...', current - 1, current, current + 1, '...', total)
    }
  }

  return pages
})

const canGoPrevious = computed(() => props.currentPage > 1 && !props.disabled)
const canGoNext = computed(() => props.currentPage < props.totalPages && !props.disabled)

const startItem = computed(() => {
  if (props.totalItems === 0) return 0
  return (props.currentPage - 1) * props.pageSize + 1
})

const endItem = computed(() => {
  const end = props.currentPage * props.pageSize
  return Math.min(end, props.totalItems)
})

function goToPage(page: number) {
  if (props.disabled) return
  if (page < 1 || page > props.totalPages) return
  if (page === props.currentPage) return
  emit('update:currentPage', page)
}

function goToFirstPage() {
  goToPage(1)
}

function goToLastPage() {
  goToPage(props.totalPages)
}

function goToPreviousPage() {
  goToPage(props.currentPage - 1)
}

function goToNextPage() {
  goToPage(props.currentPage + 1)
}

function changePageSize(event: Event) {
  const target = event.target as HTMLSelectElement
  const newSize = parseInt(target.value)
  emit('update:pageSize', newSize)
  // Reset to first page when changing page size
  emit('update:currentPage', 1)
}
</script>

<template>
  <div
    :class="[
      'flex items-center justify-between gap-4 flex-wrap',
      props.class,
    ]"
  >
    <!-- Total items display -->
    <div v-if="showTotal" class="text-sm text-text-muted">
      <span v-if="totalItems > 0">
        Showing {{ startItem }} to {{ endItem }} of {{ totalItems }} results
      </span>
      <span v-else>
        No results
      </span>
    </div>

    <!-- Pagination controls -->
    <div class="flex items-center gap-2">
      <!-- Page size selector -->
      <div v-if="showPageSize" class="flex items-center gap-2">
        <label for="page-size" class="text-sm text-text-muted whitespace-nowrap">
          Per page:
        </label>
        <select
          id="page-size"
          :value="pageSize"
          :disabled="disabled"
          class="h-8 px-2 rounded-md border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
          @change="changePageSize"
        >
          <option v-for="size in pageSizeOptions" :key="size" :value="size">
            {{ size }}
          </option>
        </select>
      </div>

      <!-- Page navigation -->
      <div class="flex items-center gap-1">
        <!-- First page -->
        <button
          type="button"
          :disabled="!canGoPrevious"
          :aria-label="'Go to first page'"
          class="h-8 w-8 rounded-md flex items-center justify-center text-text-muted hover:text-text hover:bg-surface-sunken transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-ring"
          @click="goToFirstPage"
        >
          <ChevronsLeft :size="16" />
        </button>

        <!-- Previous page -->
        <button
          type="button"
          :disabled="!canGoPrevious"
          :aria-label="'Go to previous page'"
          class="h-8 w-8 rounded-md flex items-center justify-center text-text-muted hover:text-text hover:bg-surface-sunken transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-ring"
          @click="goToPreviousPage"
        >
          <ChevronLeft :size="16" />
        </button>

        <!-- Page numbers -->
        <template v-for="(page, index) in pageNumbers" :key="index">
          <!-- Ellipsis -->
          <span
            v-if="page === '...'"
            class="h-8 w-8 flex items-center justify-center text-text-muted"
          >
            ...
          </span>

          <!-- Page number button -->
          <button
            v-else
            type="button"
            :disabled="disabled"
            :aria-label="`Go to page ${page}`"
            :aria-current="page === currentPage ? 'page' : undefined"
            :class="[
              'h-8 min-w-[2rem] px-2 rounded-md flex items-center justify-center text-sm transition-colors focus-ring',
              page === currentPage
                ? 'bg-primary-600 text-white font-medium'
                : 'text-text-muted hover:text-text hover:bg-surface-sunken disabled:opacity-50 disabled:cursor-not-allowed',
            ]"
            @click="goToPage(page as number)"
          >
            {{ page }}
          </button>
        </template>

        <!-- Next page -->
        <button
          type="button"
          :disabled="!canGoNext"
          :aria-label="'Go to next page'"
          class="h-8 w-8 rounded-md flex items-center justify-center text-text-muted hover:text-text hover:bg-surface-sunken transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-ring"
          @click="goToNextPage"
        >
          <ChevronRight :size="16" />
        </button>

        <!-- Last page -->
        <button
          type="button"
          :disabled="!canGoNext"
          :aria-label="'Go to last page'"
          class="h-8 w-8 rounded-md flex items-center justify-center text-text-muted hover:text-text hover:bg-surface-sunken transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-ring"
          @click="goToLastPage"
        >
          <ChevronsRight :size="16" />
        </button>
      </div>
    </div>
  </div>
</template>
