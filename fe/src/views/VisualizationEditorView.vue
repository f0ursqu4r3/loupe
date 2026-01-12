<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
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
  PieChart as PieChartIcon,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LInput, LSelect, LCard, LSpinner, LTagsInput } from '@/components/ui'
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

function createEmptyVisualization(): Partial<Visualization> {
  return {
    name: '',
    chart_type: 'table',
    config: {},
    tags: [],
  }
}

// Visualization state
const visualization = ref<Partial<Visualization>>(createEmptyVisualization())

// Query and data
const query = ref<Query | null>(null)
const allQueries = ref<Query[]>([])
const result = ref<QueryResult | null>(null)

// UI state
const loading = ref(false)
const saving = ref(false)
const running = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)
const previewPollToken = ref(0)
const isActive = ref(true)

function resetVisualizationState() {
  visualization.value = createEmptyVisualization()
  query.value = null
  result.value = null
  error.value = null
  saveSuccess.value = false
  running.value = false
  loading.value = false
  saving.value = false
  previewPollToken.value += 1
}

// Chart type options
const chartTypeOptions = [
  { value: 'table', label: 'Table', icon: Table },
  { value: 'line', label: 'Line Chart', icon: LineChartIcon },
  { value: 'bar', label: 'Bar Chart', icon: BarChart3 },
  { value: 'pie', label: 'Pie Chart', icon: PieChartIcon },
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

// Query options for selector
const queryOptions = computed(() =>
  allQueries.value.map((q) => ({
    value: q.id,
    label: q.name,
  })),
)

// Load all queries for the selector
async function loadQueries() {
  try {
    allQueries.value = await queriesApi.list()
  } catch (e) {
    console.error('Failed to load queries:', e)
  }
}

// Handle query change
async function handleQueryChange(queryId: string) {
  if (!queryId || queryId === visualization.value.query_id) return

  visualization.value.query_id = queryId

  // Load the new query and refresh preview
  try {
    query.value = await queriesApi.get(queryId)
    // Clear column-specific config since columns may differ
    visualization.value.config = {
      ...visualization.value.config,
      x_axis: undefined,
      y_axis: undefined,
      series_column: undefined,
      value_column: undefined,
      label_column: undefined,
    }
    await runQueryForPreview()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load query'
  }
}

// Load visualization
async function loadVisualization() {
  const activeVisualizationId = visualizationId.value
  const activeQueryId = queryIdFromRoute.value

  // Load all queries for the selector
  await loadQueries()

  if (
    activeVisualizationId !== visualizationId.value ||
    activeQueryId !== queryIdFromRoute.value
  ) {
    return
  }

  if (isNew.value) {
    // For new visualizations, load the query from query_id param
    if (activeQueryId) {
      try {
        query.value = await queriesApi.get(activeQueryId)
        if (
          activeVisualizationId !== visualizationId.value ||
          activeQueryId !== queryIdFromRoute.value
        ) {
          return
        }
        visualization.value.query_id = activeQueryId
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
    const viz = await visualizationsApi.get(activeVisualizationId!)
    if (activeVisualizationId !== visualizationId.value) return
    visualization.value = viz

    // Load associated query
    query.value = await queriesApi.get(viz.query_id)
    await runQueryForPreview()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load visualization'
  } finally {
    if (activeVisualizationId === visualizationId.value) {
      loading.value = false
    }
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
    const pollToken = ++previewPollToken.value
    await pollRunStatus(run.id, pollToken)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to run query'
  } finally {
    running.value = false
  }
}

async function pollRunStatus(runId: string, pollToken: number) {
  const maxAttempts = 60
  let attempts = 0

  while (attempts < maxAttempts) {
    if (!isActive.value || pollToken !== previewPollToken.value) return
    try {
      const run = await runsApi.get(runId)
      if (!isActive.value || pollToken !== previewPollToken.value) return

      if (run.status === 'completed') {
        const data = await runsApi.getResult(run.id)
        if (!isActive.value || pollToken !== previewPollToken.value) return
        result.value = data
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

  if (!isActive.value || pollToken !== previewPollToken.value) return
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
        tags: visualization.value.tags,
      }
      const created = await visualizationsApi.create(payload)
      router.replace({ name: 'visualization-editor', params: { id: created.id } })
      visualization.value = created
    } else {
      const updated = await visualizationsApi.update(visualizationId.value!, {
        query_id: visualization.value.query_id,
        name: visualization.value.name,
        chart_type: visualization.value.chart_type,
        config: visualization.value.config,
        tags: visualization.value.tags,
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

onBeforeUnmount(() => {
  isActive.value = false
  previewPollToken.value += 1
})

watch(
  [() => visualizationId.value, () => queryIdFromRoute.value],
  ([nextId, nextQuery], [prevId, prevQuery]) => {
    if (nextId === prevId && nextQuery === prevQuery) return
    resetVisualizationState()
    loadVisualization()
  },
)

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

            <div>
              <label class="block text-sm text-text-muted mb-1.5">Tags</label>
              <LTagsInput
                :model-value="visualization.tags || []"
                @update:model-value="visualization.tags = $event"
                placeholder="Add tags..."
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
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                id="show_legend_chart"
                :checked="visualization.config?.show_legend ?? true"
                @change="updateConfig('show_legend', ($event.target as HTMLInputElement).checked)"
                class="rounded border-border"
              />
              <label for="show_legend_chart" class="text-sm text-text-muted">Show legend</label>
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

          <!-- Pie chart config -->
          <div v-if="visualization.chart_type === 'pie'" class="space-y-4">
            <div>
              <label class="block text-sm text-text-muted mb-1.5">Label Column</label>
              <LSelect
                :model-value="visualization.config?.label_column || ''"
                @update:model-value="updateConfig('label_column', $event)"
                :options="columnOptions"
                placeholder="Select column for labels..."
              />
            </div>
            <div>
              <label class="block text-sm text-text-muted mb-1.5">Value Column</label>
              <LSelect
                :model-value="visualization.config?.value_column || ''"
                @update:model-value="updateConfig('value_column', $event)"
                :options="columnOptions"
                placeholder="Select column for values..."
              />
            </div>
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                id="donut"
                :checked="visualization.config?.donut"
                @change="updateConfig('donut', ($event.target as HTMLInputElement).checked)"
                class="rounded border-border"
              />
              <label for="donut" class="text-sm text-text-muted">Donut style (hollow center)</label>
            </div>
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                id="show_legend_pie"
                :checked="visualization.config?.show_legend ?? true"
                @change="updateConfig('show_legend', ($event.target as HTMLInputElement).checked)"
                class="rounded border-border"
              />
              <label for="show_legend_pie" class="text-sm text-text-muted">Show legend</label>
            </div>
          </div>
        </LCard>

        <!-- Source Query -->
        <LCard padding="sm">
          <h3 class="text-sm font-medium text-text mb-3">Source Query</h3>
          <div class="space-y-3">
            <LSelect
              :model-value="visualization.query_id || ''"
              @update:model-value="handleQueryChange($event)"
              :options="queryOptions"
              placeholder="Select a query..."
            />
            <LButton
              v-if="query"
              variant="ghost"
              size="sm"
              class="-ml-2"
              @click="router.push({ name: 'query-editor', params: { id: query.id } })"
            >
              Edit Query
            </LButton>
          </div>
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
