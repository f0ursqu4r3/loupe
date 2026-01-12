<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import BaseChart from './BaseChart.vue'
import type { VisualizationConfig, QueryResult } from '@/types'
import type { EChartsOption } from 'echarts'
import { formatDateLike } from '@/utils/dateTime'

const props = defineProps<{
  data: QueryResult
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

// Track container size for sparkline mode
const chartRef = ref<InstanceType<typeof BaseChart> | null>(null)
const isSmallContainer = ref(false)
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (chartRef.value?.$el) {
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (entry) {
        const { width, height } = entry.contentRect
        isSmallContainer.value = height < 200 || width < 250
      }
    })
    resizeObserver.observe(chartRef.value.$el)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

const chartOptions = computed<EChartsOption>(() => {
  if (!props.data || !props.data.rows.length) {
    return {}
  }

  const labelColumn = props.config.label_column
  const valueColumn = props.config.value_column

  // Find column indices
  const labelIdx = labelColumn ? props.data.columns.findIndex((c) => c.name === labelColumn) : 0
  const valueIdx = valueColumn
    ? props.data.columns.findIndex((c) => c.name === valueColumn)
    : props.data.columns.length > 1
      ? 1
      : 0

  if (labelIdx === -1 || valueIdx === -1) {
    return {}
  }

  const labelType = props.data.columns[labelIdx]?.data_type
  const formatLabelValue = (value: unknown) => {
    if (value === null || value === undefined) return 'Unknown'
    const formatted = formatDateLike(value, labelType)
    return formatted !== null ? formatted : String(value ?? 'Unknown')
  }

  // Build pie data
  const pieData = props.data.rows.map((row) => ({
    name: formatLabelValue(row[labelIdx]),
    value: Number(row[valueIdx]) || 0,
  }))

  // Donut mode - inner radius
  const isDonut = props.config.donut ?? false
  const innerRadius = isDonut ? '50%' : '0%'

  // Show legend: explicit config takes precedence, otherwise auto-show when not in sparkline mode
  const configShowLegend = props.config.show_legend
  const showLegend =
    !isSmallContainer.value && (configShowLegend !== undefined ? configShowLegend : true)
  const showLabel = !isSmallContainer.value

  return {
    tooltip: {
      trigger: 'item',
      appendToBody: true,
      confine: false,
      formatter: '{b}: {c} ({d}%)',
    },
    legend: showLegend
      ? {
          type: 'scroll',
          orient: 'vertical',
          right: 10,
          top: 20,
          bottom: 20,
          textStyle: {
            color: 'var(--color-text-muted)',
            fontSize: 12,
          },
          pageTextStyle: {
            color: 'var(--color-text-muted)',
          },
        }
      : undefined,
    series: [
      {
        name: props.config.label || 'Data',
        type: 'pie',
        radius: [innerRadius, isSmallContainer.value ? '90%' : '70%'],
        center: showLegend ? ['40%', '50%'] : ['50%', '50%'],
        avoidLabelOverlap: true,
        itemStyle: {
          borderRadius: 4,
          borderColor: 'var(--color-bg-surface)',
          borderWidth: 2,
        },
        label: showLabel
          ? {
              show: true,
              formatter: '{b}',
              color: 'var(--color-text-muted)',
              fontSize: 11,
            }
          : {
              show: false,
            },
        emphasis: {
          label: {
            show: true,
            fontSize: 14,
            fontWeight: 'bold',
          },
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.2)',
          },
        },
        labelLine: {
          show: showLabel,
        },
        data: pieData,
      },
    ],
  }
})
</script>

<template>
  <BaseChart ref="chartRef" :options="chartOptions" :height="height || '100%'" :loading="loading" />
</template>
