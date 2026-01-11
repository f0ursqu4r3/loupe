<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, BarChart3, Table, LineChart, Hash } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner } from '@/components/ui'
import { visualizationsApi } from '@/services/api'
import type { Visualization, ChartType } from '@/types'

const visualizations = ref<Visualization[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadVisualizations() {
  try {
    loading.value = true
    error.value = null
    visualizations.value = await visualizationsApi.list()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load visualizations'
  } finally {
    loading.value = false
  }
}

onMounted(loadVisualizations)

function formatDate(dateString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(new Date(dateString))
}

const chartTypeIcons: Record<ChartType, typeof BarChart3> = {
  table: Table,
  bar: BarChart3,
  line: LineChart,
  single_stat: Hash,
}

const chartTypeLabels: Record<ChartType, string> = {
  table: 'Table',
  bar: 'Bar Chart',
  line: 'Line Chart',
  single_stat: 'Single Stat',
}
</script>

<template>
  <AppLayout title="Visualizations">
    <template #header-actions>
      <LButton>
        <Plus class="h-4 w-4" />
        New Visualization
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <!-- Empty state -->
    <LEmptyState
      v-else-if="visualizations.length === 0"
      title="No visualizations yet"
      description="Create a visualization from one of your queries."
    >
      <template #icon>
        <BarChart3 class="h-8 w-8 text-text-subtle" />
      </template>
      <template #action>
        <LButton>
          <Plus class="h-4 w-4" />
          Create Visualization
        </LButton>
      </template>
    </LEmptyState>

    <!-- Visualizations grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <LCard
        v-for="viz in visualizations"
        :key="viz.id"
        class="group hover:border-primary-500/50 transition-colors cursor-pointer"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
              <component :is="chartTypeIcons[viz.chart_type]" class="h-5 w-5 text-primary-600" />
            </div>
            <div>
              <h3 class="font-medium text-text group-hover:text-primary-600 transition-colors">
                {{ viz.name }}
              </h3>
              <span class="text-xs text-text-muted">{{ chartTypeLabels[viz.chart_type] }}</span>
            </div>
          </div>
        </div>

        <!-- Preview placeholder -->
        <div class="h-32 bg-surface-sunken rounded-lg flex items-center justify-center mb-3">
          <component :is="chartTypeIcons[viz.chart_type]" class="h-12 w-12 text-text-subtle/30" />
        </div>

        <div class="text-xs text-text-subtle">
          Updated {{ formatDate(viz.updated_at) }}
        </div>
      </LCard>
    </div>
  </AppLayout>
</template>
