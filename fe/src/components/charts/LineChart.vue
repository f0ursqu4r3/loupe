<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import BaseChart from './BaseChart.vue'
import type { VisualizationConfig, QueryResult } from '@/types'
import type { EChartsOption } from 'echarts'

const props = defineProps<{
  data: QueryResult
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

// Sparkline detection based on container size
const containerRef = ref<HTMLDivElement | null>(null)
const containerWidth = ref(400)
const containerHeight = ref(300)

const isSparkline = computed(() => {
  // Sparkline mode when container is small (less than 200px height or 250px width)
  return containerHeight.value < 200 || containerWidth.value < 250
})

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (containerRef.value) {
    const updateSize = () => {
      if (containerRef.value) {
        containerWidth.value = containerRef.value.offsetWidth
        containerHeight.value = containerRef.value.offsetHeight
      }
    }
    updateSize()
    resizeObserver = new ResizeObserver(() => {
      updateSize()
    })
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

const chartOptions = computed<EChartsOption>(() => {
  if (!props.data || !props.data.rows.length) {
    return {}
  }

  const xAxisColumn = props.config.x_axis
  const yAxisColumn = props.config.y_axis
  const seriesColumn = props.config.series_column

  // Find column indices
  const xIdx = props.data.columns.findIndex((c) => c.name === xAxisColumn)
  const yIdx = props.data.columns.findIndex((c) => c.name === yAxisColumn)
  const seriesIdx = seriesColumn ? props.data.columns.findIndex((c) => c.name === seriesColumn) : -1

  if (xIdx === -1 || yIdx === -1) {
    return {}
  }

  let xData: string[]
  let series: Array<{
    name: string
    type: 'line'
    data: (number | null)[]
    smooth: boolean
    symbol: string
    symbolSize: number
    lineStyle: { width: number }
    emphasis: { disabled: boolean }
  }>

  if (seriesIdx !== -1) {
    // Group data by series column
    const seriesMap = new Map<string, Map<string, number>>()
    const xValues = new Set<string>()

    for (const row of props.data.rows) {
      const seriesName = String(row[seriesIdx] ?? 'Unknown')
      const xValue = String(row[xIdx] ?? '')
      const yValue = row[yIdx] as number

      xValues.add(xValue)
      if (!seriesMap.has(seriesName)) {
        seriesMap.set(seriesName, new Map())
      }
      seriesMap.get(seriesName)!.set(xValue, yValue)
    }

    xData = Array.from(xValues).sort()
    series = Array.from(seriesMap.entries()).map(([name, dataMap]) => ({
      name,
      type: 'line' as const,
      data: xData.map((x) => dataMap.get(x) ?? null),
      smooth: true,
      symbol: isSparkline.value ? 'none' : 'circle',
      symbolSize: isSparkline.value ? 0 : 6,
      lineStyle: { width: isSparkline.value ? 1.5 : 2 },
      emphasis: {
        disabled: true,
      },
    }))
  } else {
    // Single series - no grouping
    xData = props.data.rows.map((row) => String(row[xIdx] ?? ''))
    series = [
      {
        name: yAxisColumn || 'Value',
        type: 'line' as const,
        data: props.data.rows.map((row) => row[yIdx] as number),
        smooth: true,
        symbol: isSparkline.value ? 'none' : 'circle',
        symbolSize: isSparkline.value ? 0 : 6,
        lineStyle: { width: isSparkline.value ? 1.5 : 2 },
        emphasis: {
          disabled: true,
        },
      },
    ]
  }

  const showLegend = series.length > 1 && !isSparkline.value

  // Sparkline mode: minimal chrome, just the line
  if (isSparkline.value) {
    return {
      tooltip: {
        trigger: 'axis',
        appendToBody: true,
        confine: false,
      },
      grid: {
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
        containLabel: false,
      },
      xAxis: {
        type: 'category',
        data: xData,
        boundaryGap: false,
        show: false,
      },
      yAxis: {
        type: 'value',
        show: false,
      },
      series,
    }
  }

  return {
    tooltip: {
      trigger: 'axis',
      appendToBody: true,
      confine: false,
    },
    legend: {
      show: showLegend,
      type: 'scroll',
      bottom: 0,
      left: 'center',
      width: '90%',
    },
    grid: {
      left: 50,
      right: 20,
      top: 20,
      bottom: showLegend ? 50 : 20,
      containLabel: false,
    },
    xAxis: {
      type: 'category',
      data: xData,
      boundaryGap: false,
    },
    yAxis: {
      type: 'value',
    },
    series,
  }
})
</script>

<template>
  <div ref="containerRef" class="h-full w-full">
    <BaseChart :options="chartOptions" :height="height || '100%'" :loading="loading" />
  </div>
</template>
