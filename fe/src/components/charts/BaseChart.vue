<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import * as echarts from 'echarts/core'
import { LineChart, BarChart, PieChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import type { EChartsOption } from 'echarts'
import { useColorMode } from '@/composables/useColorMode'

// Register ECharts components
echarts.use([
  LineChart,
  BarChart,
  PieChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  CanvasRenderer,
])

const props = defineProps<{
  options: EChartsOption
  height?: string
  loading?: boolean
  error?: string | null
  empty?: boolean
}>()

const { isDark } = useColorMode()
const chartRef = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

// Theme colors that match our design system
const lightTheme = {
  backgroundColor: 'transparent',
  textStyle: {
    color: 'oklch(0.35 0.02 260)', // text
  },
  legend: {
    textStyle: {
      color: 'oklch(0.45 0.02 260)', // text-muted
    },
  },
  tooltip: {
    backgroundColor: 'oklch(0.98 0.005 260)', // surface
    borderColor: 'oklch(0.90 0.01 260)', // border
    borderWidth: 1,
    padding: 12,
    textStyle: {
      color: 'oklch(0.35 0.02 260)', // text
      fontSize: 13,
    },
    axisPointer: {
      type: 'cross',
      lineStyle: {
        color: 'oklch(0.90 0.01 260)',
        type: 'dashed',
      },
    },
  },
  xAxis: {
    axisLine: {
      lineStyle: { color: 'oklch(0.90 0.01 260)' },
    },
    splitLine: {
      lineStyle: { color: 'oklch(0.94 0.005 260)' },
    },
    axisLabel: {
      color: 'oklch(0.55 0.02 260)', // text-subtle
    },
  },
  yAxis: {
    axisLine: {
      lineStyle: { color: 'oklch(0.90 0.01 260)' },
    },
    splitLine: {
      lineStyle: { color: 'oklch(0.94 0.005 260)' },
    },
    axisLabel: {
      color: 'oklch(0.55 0.02 260)',
    },
  },
}

const darkTheme = {
  backgroundColor: 'transparent',
  textStyle: {
    color: 'oklch(0.92 0.01 260)', // text
  },
  legend: {
    textStyle: {
      color: 'oklch(0.75 0.02 260)', // text-muted
    },
  },
  tooltip: {
    backgroundColor: 'oklch(0.20 0.015 260)', // surface
    borderColor: 'oklch(0.30 0.02 260)', // border
    borderWidth: 1,
    padding: 12,
    textStyle: {
      color: 'oklch(0.92 0.01 260)', // text
      fontSize: 13,
    },
    axisPointer: {
      type: 'cross',
      lineStyle: {
        color: 'oklch(0.30 0.02 260)',
        type: 'dashed',
      },
    },
  },
  xAxis: {
    axisLine: {
      lineStyle: { color: 'oklch(0.30 0.02 260)' },
    },
    splitLine: {
      lineStyle: { color: 'oklch(0.25 0.015 260)' },
    },
    axisLabel: {
      color: 'oklch(0.65 0.02 260)',
    },
  },
  yAxis: {
    axisLine: {
      lineStyle: { color: 'oklch(0.30 0.02 260)' },
    },
    splitLine: {
      lineStyle: { color: 'oklch(0.25 0.015 260)' },
    },
    axisLabel: {
      color: 'oklch(0.65 0.02 260)',
    },
  },
}

const currentTheme = computed(() => (isDark.value ? darkTheme : lightTheme))

// Color palette for series
const colors = [
  'oklch(0.62 0.18 260)', // primary
  'oklch(0.62 0.18 140)', // green
  'oklch(0.62 0.18 30)', // orange
  'oklch(0.62 0.18 320)', // purple
  'oklch(0.62 0.18 200)', // cyan
  'oklch(0.62 0.18 80)', // yellow
]

function initChart() {
  if (!chartRef.value) return

  chart = echarts.init(chartRef.value)
  updateChart()
}

function updateChart() {
  if (!chart) return

  const mergedOptions: EChartsOption = {
    ...currentTheme.value,
    color: colors,
    ...props.options,
    // Merge axis styles
    xAxis: {
      ...(currentTheme.value.xAxis as object),
      ...(props.options.xAxis as object),
    },
    yAxis: {
      ...(currentTheme.value.yAxis as object),
      ...(props.options.yAxis as object),
    },
  }

  // Use notMerge=true to fully replace options including legend visibility
  chart.setOption(mergedOptions, { notMerge: true, lazyUpdate: false })

  if (props.loading) {
    chart.showLoading({
      text: '',
      color: 'oklch(0.62 0.18 260)',
      maskColor: isDark.value ? 'rgba(0,0,0,0.3)' : 'rgba(255,255,255,0.6)',
    })
  } else {
    chart.hideLoading()
  }
}

function handleResize() {
  chart?.resize()
}

// Debounced resize for ResizeObserver to avoid loop warnings
let resizeTimeout: ReturnType<typeof setTimeout> | null = null
function handleResizeDebounced() {
  if (resizeTimeout) clearTimeout(resizeTimeout)
  resizeTimeout = setTimeout(() => {
    handleResize()
  }, 0)
}

// Use ResizeObserver to detect container size changes
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  initChart()
  window.addEventListener('resize', handleResize)

  // Observe container for size changes (e.g., when grid item resizes)
  if (chartRef.value) {
    resizeObserver = new ResizeObserver(() => {
      handleResizeDebounced()
    })
    resizeObserver.observe(chartRef.value)
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  if (resizeTimeout) clearTimeout(resizeTimeout)
  resizeObserver?.disconnect()
  chart?.dispose()
})

watch(
  [() => props.options, () => props.loading, isDark],
  () => {
    updateChart()
  },
  { deep: true },
)

// Expose resize for parent components
defineExpose({
  resize: handleResize,
})
</script>

<template>
  <div class="relative w-full" :style="{ height: height || '300px' }">
    <!-- Chart container -->
    <div ref="chartRef" class="w-full h-full" />

    <!-- Empty state overlay -->
    <div
      v-if="empty && !loading && !error"
      class="absolute inset-0 flex flex-col items-center justify-center bg-surface/80 backdrop-blur-sm"
    >
      <svg
        class="w-16 h-16 text-text-subtle mb-3"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
        />
      </svg>
      <p class="text-sm text-text-muted">No data to display</p>
    </div>

    <!-- Error state overlay -->
    <div
      v-if="error && !loading"
      class="absolute inset-0 flex flex-col items-center justify-center bg-surface/80 backdrop-blur-sm"
    >
      <svg
        class="w-16 h-16 text-error mb-3"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        />
      </svg>
      <p class="text-sm font-medium text-error mb-1">Failed to load chart</p>
      <p class="text-xs text-text-muted">{{ error }}</p>
    </div>
  </div>
</template>
