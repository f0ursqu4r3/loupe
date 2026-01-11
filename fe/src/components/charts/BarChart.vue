<script setup lang="ts">
import { computed } from 'vue'
import BaseChart from './BaseChart.vue'
import type { VisualizationConfig, QueryResult } from '@/types'
import type { EChartsOption } from 'echarts'

const props = defineProps<{
  data: QueryResult
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

const chartOptions = computed<EChartsOption>(() => {
  if (!props.data || !props.data.rows.length) {
    return {}
  }

  const xAxisColumn = props.config.x_axis
  const yAxisColumns = Array.isArray(props.config.y_axis)
    ? props.config.y_axis
    : props.config.y_axis
      ? [props.config.y_axis]
      : []

  // Find column indices
  const xIdx = props.data.columns.findIndex((c) => c.name === xAxisColumn)
  const yIndices = yAxisColumns.map((col) => props.data.columns.findIndex((c) => c.name === col))

  if (xIdx === -1 || yIndices.some((i) => i === -1)) {
    return {}
  }

  // Extract data - cast to string for category axis
  const xData = props.data.rows.map((row) => String(row[xIdx] ?? ''))
  const series = yAxisColumns.map((colName, i) => {
    const yIdx = yIndices[i]!
    return {
      name: colName,
      type: 'bar' as const,
      data: props.data.rows.map((row) => row[yIdx] as number),
      barMaxWidth: 50,
      stack: props.config.stacked ? 'total' : undefined,
      emphasis: {
        focus: 'series' as const,
      },
      itemStyle: {
        borderRadius: props.config.stacked
          ? ([0, 0, 0, 0] as [number, number, number, number])
          : ([4, 4, 0, 0] as [number, number, number, number]),
      },
    }
  })

  // Adjust border radius for stacked bars (only top bar gets rounded corners)
  if (props.config.stacked && series.length > 0) {
    const lastSeries = series[series.length - 1]
    if (lastSeries?.itemStyle) {
      lastSeries.itemStyle.borderRadius = [4, 4, 0, 0]
    }
  }

  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow',
      },
    },
    legend: {
      show: yAxisColumns.length > 1,
      bottom: 0,
    },
    grid: {
      left: 50,
      right: 20,
      top: 20,
      bottom: yAxisColumns.length > 1 ? 40 : 20,
      containLabel: false,
    },
    xAxis: {
      type: 'category',
      data: xData,
    },
    yAxis: {
      type: 'value',
    },
    series,
  }
})
</script>

<template>
  <BaseChart :options="chartOptions" :height="height" :loading="loading" />
</template>
