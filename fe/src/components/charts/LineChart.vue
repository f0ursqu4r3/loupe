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
      type: 'line' as const,
      data: props.data.rows.map((row) => row[yIdx] as number),
      smooth: true,
      symbol: 'circle',
      symbolSize: 6,
      lineStyle: {
        width: 2,
      },
      emphasis: {
        focus: 'series' as const,
      },
    }
  })

  return {
    tooltip: {
      trigger: 'axis',
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
  <BaseChart :options="chartOptions" :height="height" :loading="loading" />
</template>
