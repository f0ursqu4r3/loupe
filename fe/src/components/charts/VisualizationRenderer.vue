<script setup lang="ts">
import { computed } from 'vue'
import LineChart from './LineChart.vue'
import BarChart from './BarChart.vue'
import PieChart from './PieChart.vue'
import SingleStat from './SingleStat.vue'
import DataTable from './DataTable.vue'
import type { ChartType, VisualizationConfig, QueryResult } from '@/types'

const props = defineProps<{
  chartType: ChartType
  data: QueryResult | null
  config: VisualizationConfig
  height?: string
  loading?: boolean
}>()

const chartComponent = computed(() => {
  switch (props.chartType) {
    case 'line':
      return LineChart
    case 'bar':
      return BarChart
    case 'pie':
      return PieChart
    case 'single_stat':
      return SingleStat
    case 'table':
    default:
      return DataTable
  }
})
</script>

<template>
  <div class="w-full h-full flex flex-col">
    <div v-if="config.label" class="text-sm font-medium text-text mb-2 px-1">
      {{ config.label }}
    </div>
    <component
      :is="chartComponent"
      :data="data || { columns: [], rows: [], row_count: 0, execution_time_ms: 0 }"
      :config="config"
      :height="height"
      :loading="loading"
      class="flex-1 min-h-0"
    />
  </div>
</template>
