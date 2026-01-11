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
    type: 'bar'
    data: (number | null)[]
    barMaxWidth: number
    stack: string | undefined
    emphasis: { disabled: boolean }
    itemStyle: { borderRadius: [number, number, number, number] }
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
      type: 'bar' as const,
      data: xData.map((x) => dataMap.get(x) ?? null),
      barMaxWidth: 50,
      stack: props.config.stacked ? 'total' : undefined,
      emphasis: { disabled: true },
      itemStyle: {
        borderRadius: props.config.stacked
          ? ([0, 0, 0, 0] as [number, number, number, number])
          : ([4, 4, 0, 0] as [number, number, number, number]),
      },
    }))
  } else {
    // Single series - no grouping
    xData = props.data.rows.map((row) => String(row[xIdx] ?? ''))
    series = [
      {
        name: yAxisColumn || 'Value',
        type: 'bar' as const,
        data: props.data.rows.map((row) => row[yIdx] as number),
        barMaxWidth: 50,
        stack: props.config.stacked ? 'total' : undefined,
        emphasis: { disabled: true },
        itemStyle: { borderRadius: [4, 4, 0, 0] as [number, number, number, number] },
      },
    ]
  }

  // Adjust border radius for stacked bars (only top bar gets rounded corners)
  if (props.config.stacked && series.length > 0) {
    const lastSeries = series[series.length - 1]
    if (lastSeries?.itemStyle) {
      lastSeries.itemStyle.borderRadius = [4, 4, 0, 0]
    }
  }

  // Show legend: explicit config takes precedence, otherwise auto-show for multi-series
  const configShowLegend = props.config.show_legend
  const showLegend = configShowLegend !== undefined ? configShowLegend : series.length > 1

  return {
    tooltip: {
      trigger: 'axis',
      appendToBody: true,
      confine: false,
      axisPointer: {
        type: 'shadow',
      },
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
