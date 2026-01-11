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
    emphasis: { focus: 'series' }
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
      symbol: 'circle',
      symbolSize: 6,
      lineStyle: { width: 2 },
      emphasis: { focus: 'series' as const },
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
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: { width: 2 },
        emphasis: { focus: 'series' as const },
      },
    ]
  }

  const showLegend = series.length > 1

  return {
    tooltip: {
      trigger: 'axis',
    },
    legend: {
      show: showLegend,
      bottom: 0,
    },
    grid: {
      left: 50,
      right: 20,
      top: 20,
      bottom: showLegend ? 40 : 20,
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
