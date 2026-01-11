<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft,
  Save,
  Loader2,
  AlertCircle,
  CheckCircle,
  X,
  Play,
  Table,
  LineChart as LineChartIcon,
  BarChart3,
  Hash,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LInput, LSelect, LCard, LSpinner } from '@/components/ui'
import { VisualizationRenderer } from '@/components/charts'
import { visualizationsApi, queriesApi, runsApi } from '@/services/api'
import type {
  Visualization,
  Query,
  QueryResult,
  ChartType,
  VisualizationConfig,
  CreateVisualizationRequest,
} from '@/types'

const route = useRoute()
const router = useRouter()

// Route params
const visualizationId = computed(() => route.params.id as string | undefined)
const queryIdFromRoute = computed(() => route.query.query_id as string | undefined)
const isNew = computed(() => !visualizationId.value || visualizationId.value === 'new')

// Visualization state
const visualization = ref<Partial<Visualization>>({
  name: '',
  chart_type: 'table',
  config: {},
})

// Query and data
const query = ref<Query | null>(null)
const result = ref<QueryResult | null>(null)

// UI state
const loading = ref(false)
const saving = ref(false)
const running = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)

// Chart type options
const chartTypeOptions = [
  { value: 'table', label: 'Table', icon: Table },
  { value: 'line', label: 'Line Chart', icon: LineChartIcon },
  { value: 'bar', label: 'Bar Chart', icon: BarChart3 },
  { value: 'single_stat', label: 'Single Stat', icon: Hash },
]

// Column options from result
const columnOptions = computed(() => {
  if (!result.value) return []
  return result.value.columns.map((col) => ({
    value: col.name,
    label: `${col.name} (${col.data_type})`,
  }))
})

// Load visualization
async function loadVisualization() {
  if (isNew.value) {
    // For new visualizations, load the query from query_id param
    if (queryIdFromRoute.value) {
      try {
        query.value = await queriesApi.get(queryIdFromRoute.value)
        visualization.value.query_id = queryIdFromRoute.value
        visualization.value.name = `${query.value.name} - Visualization`
        await runQueryForPreview()
      } catch (e) {
        error.value = e instanceof Error ? e.message : 'Failed to load query'
      }
    }
    return
  }

  try {
    loading.value = true
    const viz = await visualizationsApi.get(visualizationId.value!)
    visualization.value = viz

    // Load associated query
    query.value = await queriesApi.get(viz.query_id)
    await runQueryForPreview()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load visualization'
  } finally {
    loading.value = false
  }
}

// Run query to get preview data
async function runQueryForPreview() {
  if (!query.value?.id) return

  try {
    running.value = true
    result.value = null

    // Create a run
    const run = await runsApi.create({
      query_id: query.value.id,
      parameters: {},
    })

    // Poll for completion
    await pollRunStatus(run.id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to run query'
  } finally {
    running.value = false
  }
}

async function pollRunStatus(runId: string) {
  const maxAttempts = 60
  let attempts = 0

  while (attempts < maxAttempts) {
    try {
      const run = await runsApi.get(runId)

      if (run.status === 'completed') {
        result.value = await runsApi.getResult(run.id)
        return
      } else if (
        run.status === 'failed' ||
        run.status === 'cancelled' ||
        run.status === 'timeout'
      ) {
        error.value = run.error_message || `Query ${run.status}`
        return
      }

      await new Promise((resolve) => setTimeout(resolve, 500))
      attempts++
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to check query status'
      return
    }
  }

  error.value = 'Query timed out waiting for results'
}

// Save visualization
async function saveVisualization() {
  if (!visualization.value.name?.trim()) {
    error.value = 'Visualization name is required'
    return
  }

  try {
    saving.value = true
    error.value = null

    if (isNew.value) {
      const payload: CreateVisualizationRequest = {
        query_id: visualization.value.query_id!,
        name: visualization.value.name!,
        chart_type: visualization.value.chart_type!,
        config: visualization.value.config,
      }
      const created = await visualizationsApi.create(payload)
      router.replace({ name: 'visualization-editor', params: { id: created.id } })
      visualization.value = created
    } else {
      const updated = await visualizationsApi.update(visualizationId.value!, {
        name: visualization.value.name,
        chart_type: visualization.value.chart_type,
        config: visualization.value.config,
      })
      visualization.value = updated
    }

    saveSuccess.value = true
    setTimeout(() => (saveSuccess.value = false), 2000)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save visualization'
  } finally {
    saving.value = false
  }
}

// Update config helper
function updateConfig(key: keyof VisualizationConfig, value: unknown) {
  visualization.value.config = {
    ...visualization.value.config,
    [key]: value,
  }
}

onMounted(loadVisualization)

// Clear error on changes
watch(
  () => visualization.value.name,
  () => {
    error.value = null
  },
)
</script>

<template>
  <AppLayout :title="isNew ? 'New Visualization' : visualization.name || 'Visualization Editor'">
    <template #header-left>
      <LButton variant="ghost" size="sm" @click="router.push({ name: 'visualizations' })">
        <ArrowLeft class="h-4 w-4" />
        Back
      </LButton>
    </template>

    <template #header-actions>
      <LButton variant="secondary" :disabled="saving" @click="saveVisualization">
        <Loader2 v-if="saving" class="h-4 w-4 animate-spin" />
        <Save v-else class="h-4 w-4" />
        {{ saving ? 'Saving...' : 'Save' }}
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <div v-else class="flex gap-6 h-[calc(100vh-8rem)]">
      <!-- Config sidebar -->
      <div class="w-80 shrink-0 flex flex-col gap-4">
        <!-- Error banner -->
        <div
          v-if="error"
          class="flex items-center gap-3 p-3 bg-error-muted text-error rounded-lg text-sm"
        >
          <AlertCircle class="h-5 w-5 shrink-0" />
          <span class="flex-1 text-xs">{{ error }}</span>
          <button @click="error = null" class="p-1 hover:bg-error/20 rounded">
            <X class="h-4 w-4" />
          </button>
        </div>

        <!-- Success banner -->
        <div
          v-if="saveSuccess"
          class="flex items-center gap-3 p-3 bg-success-muted text-success rounded-lg text-sm"
        >
          <CheckCircle class="h-5 w-5 shrink-0" />
          <span>Saved successfully</span>
        </div>

        <!-- Basic settings -->
        <LCard padding="sm">
          <h3 class="text-sm font-medium text-text mb-3">Settings</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-muted mb-1.5">Name</label>
              <LInput v-model="visualization.name" placeholder="My Visualization" />
            </div>

            <div>
              <label class="block text-sm text-text-muted mb-1.5">Chart Type</label>
              <div class="grid grid-cols-2 gap-2">
                <button
                  v-for="opt in chartTypeOptions"
                  :key="opt.value"
                  type="button"
                  class="flex flex-col items-center gap-1.5 p-3 rounded-lg border transition-colors"
                  :class="
                    visualization.chart_type === opt.value
                      ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
                      : 'border-border hover:border-border-hover'
                  "
                  @click="visualization.chart_type = opt.value as ChartType"
                >
                  <component
                    :is="opt.icon"
                    class="h-5 w-5"
                    :class="
                      visualization.chart_type === opt.value
                        ? 'text-primary-600'
                        : 'text-text-muted'
                    "
                  />
                  <span
                    class="text-xs"
                    :class="
                      visualization.chart_type === opt.value
                        ? 'text-primary-600 font-medium'
                        : 'text-text-muted'
                    "
                  >
                    {{ opt.label }}
                  </span>
                </button>
              </div>
            </div>

            <div>
              <label class="block text-sm text-text-muted mb-1.5">Label (optional)</label>
              <LInput
                :model-value="visualization.config?.label || ''"
                @update:model-value="updateConfig('label', $event)"
                placeholder="Chart label..."
              />
            </div>
          </div>
        </LCard>

        <!-- Chart-specific config -->
        <LCard v-if="visualization.chart_type !== 'table'" padding="sm">
          <h3 class="text-sm font-medium text-text mb-3">Configuration</h3>

          <!-- Line/Bar chart config -->
          <div
            v-if="visualization.chart_type === 'line' || visualization.chart_type === 'bar'"
            class="space-y-4"
          >
            <div>
              <label class="block text-sm text-text-muted mb-1.5">X-Axis Column</label>
              <LSelect
                :model-value="visualization.config?.x_axis || ''"
                @update:model-value="updateConfig('x_axis', $event)"
                :options="columnOptions"
                placeholder="Select column..."
              />
            </div>
            <div>
              <label class="block text-sm text-text-muted mb-1.5">Y-Axis Column</label>
              <LSelect
                :model-value="visualization.config?.y_axis || ''"
                @update:model-value="updateConfig('y_axis', $event)"
                :options="columnOptions"
                placeholder="Select column..."
              />
            </div>
            <div>
              <label class="block text-sm text-text-muted mb-1.5">Series Column (optional)</label>
              <LSelect
                :model-value="visualization.config?.series_column || ''"
                @update:model-value="updateConfig('series_column', $event || undefined)"
                :options="[{ value: '', label: 'None' }, ...columnOptions]"
                placeholder="Group by column..."
              />
              <p class="text-xs text-text-muted mt-1">
                Split into multiple lines/bars by this column
              </p>
            </div>
            <div v-if="visualization.chart_type === 'bar'" class="flex items-center gap-2">
              <input
                type="checkbox"
                id="stacked"
                :checked="visualization.config?.stacked"
                @change="updateConfig('stacked', ($event.target as HTMLInputElement).checked)"
                class="rounded border-border"
              />
              <label for="stacked" class="text-sm text-text-muted">Stacked bars</label>
            </div>
          </div>

          <!-- Single stat config -->
          <div v-if="visualization.chart_type === 'single_stat'" class="space-y-4">
            <div>
              <label class="block text-sm text-text-muted mb-1.5">Value Column</label>
              <LSelect
                :model-value="visualization.config?.value_column || ''"
                @update:model-value="updateConfig('value_column', $event)"
                :options="columnOptions"
                placeholder="Select column..."
              />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm text-text-muted mb-1.5">Prefix</label>
                <LInput
                  :model-value="visualization.config?.prefix || ''"
                  @update:model-value="updateConfig('prefix', $event)"
                  placeholder="$"
                />
              </div>
              <div>
                <label class="block text-sm text-text-muted mb-1.5">Suffix</label>
                <LInput
                  :model-value="visualization.config?.suffix || ''"
                  @update:model-value="updateConfig('suffix', $event)"
                  placeholder="%"
                />
              </div>
            </div>
          </div>
        </LCard>

        <!-- Query info -->
        <LCard v-if="query" padding="sm">
          <h3 class="text-sm font-medium text-text mb-2">Source Query</h3>
          <p class="text-sm text-text-muted truncate">{{ query.name }}</p>
          <LButton
            variant="ghost"
            size="sm"
            class="mt-2 -ml-2"
            @click="router.push({ name: 'query-editor', params: { id: query.id } })"
          >
            Edit Query
          </LButton>
        </LCard>
      </div>

      <!-- Preview area -->
      <div class="flex-1 flex flex-col min-w-0">
        <LCard padding="none" class="flex-1 flex flex-col">
          <div class="p-3 border-b border-border flex items-center justify-between">
            <span class="text-sm font-medium text-text">Preview</span>
            <div class="flex items-center gap-3">
              <LButton
                variant="ghost"
                size="sm"
                :disabled="running || !query"
                @click="runQueryForPreview"
              >
                <Loader2 v-if="running" class="h-4 w-4 animate-spin" />
                <Play v-else class="h-4 w-4" />
                {{ running ? 'Loading...' : 'Refresh' }}
              </LButton>
              <span v-if="result" class="text-xs text-text-muted">
                {{ result.rows.length.toLocaleString() }} rows
              </span>
            </div>
          </div>

          <div class="flex-1 p-4 overflow-auto">
            <VisualizationRenderer
              :chart-type="visualization.chart_type || 'table'"
              :data="result"
              :config="visualization.config || {}"
              :loading="running"
              height="100%"
            />
          </div>
        </LCard>
      </div>
    </div>
  </AppLayout>
</template>
