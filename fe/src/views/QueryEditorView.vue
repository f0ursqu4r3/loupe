<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Splitpanes, Pane } from 'splitpanes'
import 'splitpanes/dist/splitpanes.css'
import {
  Play,
  Save,
  Database,
  Clock,
  Rows3,
  Loader2,
  ChevronDown,
  ChevronUp,
  AlertCircle,
  CheckCircle,
  X,
  WandSparkles,
  BarChart3,
  GripHorizontal,
  CalendarClock,
  Plus,
  Eye,
  EyeOff,
  ArrowLeft,
  Table,
  LineChart as LineChartIcon,
  PieChart as PieChartIcon,
  Hash,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import {
  LButton,
  LInput,
  LTextarea,
  LSelect,
  LCard,
  LSpinner,
  LBadge,
  LTagsInput,
} from '@/components/ui'
import { SqlEditor } from '@/components/editor'
import { QueryParameters, ParameterInputs } from '@/components/query'
import { VisualizationRenderer } from '@/components/charts'
import { queriesApi, runsApi, datasourcesApi, schedulesApi, visualizationsApi } from '@/services/api'
import { formatDateLike } from '@/utils/dateTime'
import type {
  Query,
  Datasource,
  Run,
  QueryResult,
  CreateQueryRequest,
  Schedule,
  Visualization,
  ChartType,
  VisualizationConfig,
} from '@/types'

const route = useRoute()
const router = useRouter()

// Query state
const queryId = computed(() => route.params.id as string | undefined)
const isNew = computed(() => !queryId.value || queryId.value === 'new')

function createEmptyQuery(): Partial<Query> {
  return {
    name: '',
    description: '',
    sql: 'SELECT * FROM ',
    datasource_id: '',
    parameters: [],
    tags: [],
    timeout_seconds: 30,
    max_rows: 10000,
  }
}

const query = ref<Partial<Query>>(createEmptyQuery())

// UI state
const loading = ref(false)
const saving = ref(false)
const running = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)

// Editor ref
const sqlEditorRef = ref<InstanceType<typeof SqlEditor> | null>(null)

// Datasources
const datasources = ref<Datasource[]>([])
const datasourceOptions = computed(() =>
  datasources.value.map((ds) => ({
    value: ds.id,
    label: ds.name,
  })),
)

// Preview panel layout - persisted to localStorage
const LAYOUT_STORAGE_KEY = 'loupe:query-editor:layout'

function loadLayoutFromStorage() {
  try {
    const stored = localStorage.getItem(LAYOUT_STORAGE_KEY)
    if (stored) return JSON.parse(stored)
  } catch {
    // ignore
  }
  return null
}

const storedLayout = loadLayoutFromStorage()
const previewPanelPct = ref(storedLayout?.previewPanelPct ?? 35)
// Default to false - user can toggle on when needed
const showPreviewPanel = ref(false)
const previewTab = ref<'visualizations' | 'schedules'>('visualizations')

// Persist layout changes
watch([previewPanelPct, showPreviewPanel], () => {
  localStorage.setItem(
    LAYOUT_STORAGE_KEY,
    JSON.stringify({
      previewPanelPct: previewPanelPct.value,
      showPreviewPanel: showPreviewPanel.value,
    }),
  )
})

function onPaneResized(panes: { size: number }[]) {
  if (panes[1]) {
    previewPanelPct.value = panes[1].size
  }
}

function togglePreviewPanel() {
  showPreviewPanel.value = !showPreviewPanel.value
}

// Schedules for this query
const schedules = ref<Schedule[]>([])

// Visualizations for this query
const visualizations = ref<Visualization[]>([])
const loadingVisualizations = ref(false)

// Inline visualization editor state
const editingVisualization = ref<Partial<Visualization> | null>(null)
const savingVisualization = ref(false)
const visualizationPreviewRunning = ref(false)
const visualizationPreviewResult = ref<QueryResult | null>(null)

// Run results
const currentRun = ref<Run | null>(null)
const result = ref<QueryResult | null>(null)
const resultError = ref<string | null>(null)
const showResults = ref(false)
const runPollToken = ref(0)
const isActive = ref(true)

// Editor resizing
const editorHeight = ref(300)
const isResizing = ref(false)
const resizeStartY = ref(0)
const resizeStartHeight = ref(0)
const MIN_EDITOR_HEIGHT = 150
const MAX_EDITOR_HEIGHT = 800

function startResize(e: MouseEvent) {
  isResizing.value = true
  resizeStartY.value = e.clientY
  resizeStartHeight.value = editorHeight.value
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'row-resize'
  document.body.style.userSelect = 'none'
}

function handleResize(e: MouseEvent) {
  if (!isResizing.value) return
  const delta = e.clientY - resizeStartY.value
  const newHeight = Math.min(
    MAX_EDITOR_HEIGHT,
    Math.max(MIN_EDITOR_HEIGHT, resizeStartHeight.value + delta),
  )
  editorHeight.value = newHeight
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

onBeforeUnmount(() => {
  isActive.value = false
  runPollToken.value += 1
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
})

// Parameter values for running
const parameterValues = ref<Record<string, unknown>>({})

// Show parameters section
const showParameters = ref(false)

// Show query metadata section
const showMetadata = ref(false)

function resetQueryState() {
  query.value = createEmptyQuery()
  parameterValues.value = {}
  schedules.value = []
  visualizations.value = []
  currentRun.value = null
  result.value = null
  resultError.value = null
  showResults.value = false
  showParameters.value = false
  running.value = false
  error.value = null
  saveSuccess.value = false
  loading.value = false
  saving.value = false
  runPollToken.value += 1
  // Don't reset preview panel state - let user's preference persist
}

// Load datasources
async function loadDatasources() {
  try {
    const response = await datasourcesApi.list()
    datasources.value = response.data
    // Auto-select first datasource for new queries
    const firstDatasource = datasources.value[0]
    if (isNew.value && firstDatasource && !query.value.datasource_id) {
      query.value.datasource_id = firstDatasource.id
    }
  } catch (e) {
    console.error('Failed to load datasources:', e)
  }
}

// Load existing query
async function loadQuery() {
  if (isNew.value) return

  const activeQueryId = queryId.value
  try {
    loading.value = true
    const data = await queriesApi.get(activeQueryId!)
    if (activeQueryId !== queryId.value) return
    query.value = data
    // Load schedules and visualizations for this query
    await Promise.all([loadSchedules(), loadVisualizations()])
    // Show preview panel when editing existing query (if there are visualizations or schedules)
    if (!showPreviewPanel.value && storedLayout?.showPreviewPanel !== false) {
      showPreviewPanel.value = true
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load query'
  } finally {
    if (activeQueryId === queryId.value) {
      loading.value = false
    }
  }
}

// Load schedules for this query
async function loadSchedules() {
  if (isNew.value || !queryId.value) return

  try {
    const response = await schedulesApi.list()
    schedules.value = response.data.filter((s) => s.query_id === queryId.value)
  } catch (e) {
    console.error('Failed to load schedules:', e)
  }
}

// Load visualizations for this query
async function loadVisualizations() {
  if (isNew.value || !queryId.value) return

  try {
    loadingVisualizations.value = true
    const response = await visualizationsApi.list()
    visualizations.value = response.data.filter((v) => v.query_id === queryId.value)
  } catch (e) {
    console.error('Failed to load visualizations:', e)
  } finally {
    loadingVisualizations.value = false
  }
}

// Inline visualization editor

// Chart type options
const chartTypeOptions = [
  { value: 'table', label: 'Table', icon: Table },
  { value: 'line', label: 'Line Chart', icon: LineChartIcon },
  { value: 'bar', label: 'Bar Chart', icon: BarChart3 },
  { value: 'pie', label: 'Pie Chart', icon: PieChartIcon },
  { value: 'single_stat', label: 'Single Stat', icon: Hash },
]

// Column options from preview result
const vizColumnOptions = computed(() => {
  if (!visualizationPreviewResult.value) return []
  return visualizationPreviewResult.value.columns.map((col) => ({
    value: col.name,
    label: `${col.name} (${col.data_type})`,
  }))
})

function openVisualizationEditor(viz: Visualization) {
  editingVisualization.value = { ...viz }
  previewTab.value = 'visualizations'
  runVisualizationPreview()
}

function closeVisualizationEditor() {
  editingVisualization.value = null
  visualizationPreviewResult.value = null
}

function createNewVisualization() {
  if (!queryId.value) return
  editingVisualization.value = {
    query_id: queryId.value,
    name: `${query.value.name} - Visualization`,
    chart_type: 'table',
    config: {},
    tags: [],
  }
  runVisualizationPreview()
}

async function saveInlineVisualization() {
  if (!editingVisualization.value?.name?.trim()) {
    error.value = 'Visualization name is required'
    return
  }

  try {
    savingVisualization.value = true
    error.value = null

    const isNewViz = !editingVisualization.value.id || editingVisualization.value.id === ''

    if (isNewViz) {
      const created = await visualizationsApi.create({
        query_id: editingVisualization.value.query_id!,
        name: editingVisualization.value.name,
        chart_type: editingVisualization.value.chart_type!,
        config: editingVisualization.value.config,
        tags: editingVisualization.value.tags,
      })
      editingVisualization.value = created
      visualizations.value.push(created)
    } else {
      if (!editingVisualization.value.id) {
        error.value = 'Invalid visualization ID'
        return
      }
      const updated = await visualizationsApi.update(editingVisualization.value.id, {
        query_id: editingVisualization.value.query_id,
        name: editingVisualization.value.name,
        chart_type: editingVisualization.value.chart_type,
        config: editingVisualization.value.config,
        tags: editingVisualization.value.tags,
      })
      editingVisualization.value = updated
      const idx = visualizations.value.findIndex((v) => v.id === updated.id)
      if (idx !== -1) {
        visualizations.value[idx] = updated
      }
    }

    saveSuccess.value = true
    setTimeout(() => (saveSuccess.value = false), 2000)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save visualization'
  } finally {
    savingVisualization.value = false
  }
}

async function runVisualizationPreview() {
  if (!query.value?.id) return

  try {
    visualizationPreviewRunning.value = true
    visualizationPreviewResult.value = null

    // Reuse the current query result if available
    if (result.value) {
      visualizationPreviewResult.value = result.value
      visualizationPreviewRunning.value = false
      return
    }

    // Otherwise run the query
    const run = await runsApi.create({
      query_id: query.value.id,
      parameters: {},
    })

    // Poll for completion (simplified)
    const pollToken = ++runPollToken.value
    await pollVisualizationPreview(run.id, pollToken)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to run preview'
  } finally {
    visualizationPreviewRunning.value = false
  }
}

async function pollVisualizationPreview(runId: string, pollToken: number) {
  const maxAttempts = 60
  let attempts = 0

  while (attempts < maxAttempts) {
    if (!isActive.value || pollToken !== runPollToken.value) return
    try {
      const run = await runsApi.get(runId)
      if (!isActive.value || pollToken !== runPollToken.value) return

      if (run.status === 'completed') {
        const data = await runsApi.getResult(run.id)
        if (!isActive.value || pollToken !== runPollToken.value) return
        visualizationPreviewResult.value = data
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

  if (!isActive.value || pollToken !== runPollToken.value) return
  error.value = 'Query timed out waiting for results'
}

function updateVisualizationConfig(key: keyof VisualizationConfig, value: unknown) {
  if (!editingVisualization.value) return
  editingVisualization.value.config = {
    ...editingVisualization.value.config,
    [key]: value,
  }
}

// Save query
async function saveQuery() {
  if (!query.value.name?.trim()) {
    error.value = 'Query name is required'
    return
  }
  if (!query.value.datasource_id) {
    error.value = 'Please select a datasource'
    return
  }
  if (!query.value.sql?.trim()) {
    error.value = 'SQL query is required'
    return
  }

  try {
    saving.value = true
    error.value = null

    const payload: CreateQueryRequest = {
      name: query.value.name!,
      description: query.value.description,
      sql: query.value.sql!,
      datasource_id: query.value.datasource_id!,
      parameters: query.value.parameters,
      tags: query.value.tags,
      timeout_seconds: query.value.timeout_seconds,
      max_rows: query.value.max_rows,
    }

    if (isNew.value) {
      const created = await queriesApi.create(payload)
      // Navigate to edit mode with new ID
      router.replace({ name: 'query-editor', params: { id: created.id } })
      query.value = created
    } else {
      const updated = await queriesApi.update(queryId.value!, payload)
      query.value = updated
    }

    saveSuccess.value = true
    setTimeout(() => (saveSuccess.value = false), 2000)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save query'
  } finally {
    saving.value = false
  }
}

// Run query
async function runQuery() {
  if (!query.value.sql?.trim()) {
    resultError.value = 'No SQL to execute'
    return
  }

  // Always save before running to ensure latest changes are executed
  await saveQuery()
  if (error.value) return

  try {
    running.value = true
    resultError.value = null
    result.value = null
    showResults.value = true

    // Execute the query via runs API
    const queryIdToRun = query.value.id || queryId.value!
    currentRun.value = await runsApi.create({
      query_id: queryIdToRun,
      parameters: parameterValues.value,
    })

    // Poll for completion
    const pollToken = ++runPollToken.value
    await pollRunStatus(pollToken)
  } catch (e) {
    resultError.value = e instanceof Error ? e.message : 'Failed to execute query'
    running.value = false
  }
}

// Poll run status until complete
async function pollRunStatus(pollToken: number) {
  if (!currentRun.value) return

  const maxAttempts = 60
  let attempts = 0

  while (attempts < maxAttempts) {
    if (!isActive.value || pollToken !== runPollToken.value) {
      running.value = false
      return
    }
    try {
      const run = await runsApi.get(currentRun.value.id)
      if (!isActive.value || pollToken !== runPollToken.value) {
        running.value = false
        return
      }
      currentRun.value = run

      if (run.status === 'completed') {
        // Fetch results
        const data = await runsApi.getResult(run.id)
        if (!isActive.value || pollToken !== runPollToken.value) {
          running.value = false
          return
        }
        result.value = data
        running.value = false
        return
      } else if (
        run.status === 'failed' ||
        run.status === 'cancelled' ||
        run.status === 'timeout'
      ) {
        resultError.value = run.error_message || `Query ${run.status}`
        running.value = false
        return
      }

      // Still running, wait and poll again
      await new Promise((resolve) => setTimeout(resolve, 500))
      attempts++
    } catch (e) {
      resultError.value = e instanceof Error ? e.message : 'Failed to check query status'
      running.value = false
      return
    }
  }

  if (!isActive.value || pollToken !== runPollToken.value) return
  resultError.value = 'Query timed out waiting for results'
  running.value = false
}

// Format duration from ms
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

function formatResultCell(value: unknown, dataType?: string): string {
  if (value === null || value === undefined) return ''
  const formatted = formatDateLike(value, dataType)
  if (formatted !== null) {
    return formatted
  }
  return String(value)
}

// Compute duration from run timestamps
function getRunDuration(run: Run): number | null {
  if (run.started_at && run.completed_at) {
    return new Date(run.completed_at).getTime() - new Date(run.started_at).getTime()
  }
  return null
}

onMounted(() => {
  loadDatasources()
  loadQuery()
})

watch(
  () => queryId.value,
  (next, prev) => {
    if (next === prev) return
    resetQueryState()
    loadQuery()
  },
)

// Clear error when inputs change
watch([() => query.value.name, () => query.value.sql, () => query.value.datasource_id], () => {
  error.value = null
})
</script>

<template>
  <AppLayout :title="isNew ? 'New Query' : query.name || 'Query Editor'" back="queries">
    <template #header-actions>
      <LButton
        v-if="!isNew"
        variant="ghost"
        size="sm"
        @click="togglePreviewPanel"
        :title="showPreviewPanel ? 'Hide preview panel' : 'Show preview panel'"
      >
        <component :is="showPreviewPanel ? EyeOff : Eye" :size="16" />
        <span class="ml-1.5">Preview</span>
      </LButton>
      <LButton variant="secondary" :disabled="saving" @click="saveQuery">
        <Loader2 v-if="saving" :size="16" class="animate-spin" />
        <Save v-else :size="16" />
        {{ saving ? 'Saving...' : 'Save' }}
      </LButton>
      <LButton :disabled="running || !query.sql?.trim()" @click="runQuery">
        <Loader2 v-if="running" :size="16" class="animate-spin" />
        <Play v-else :size="16" />
        {{ running ? 'Running...' : 'Run' }}
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <div v-else class="space-y-4">
      <!-- Error banner -->
      <div
        v-if="error"
        class="flex items-center gap-3 p-3 bg-error-muted text-error rounded-lg text-sm"
      >
        <AlertCircle :size="20" class="shrink-0" />
        <span class="flex-1">{{ error }}</span>
        <button @click="error = null" class="p-1 hover:bg-error/20 rounded">
          <X :size="16" />
        </button>
      </div>

      <!-- Success banner -->
      <div
        v-if="saveSuccess"
        class="flex items-center gap-3 p-3 bg-success-muted text-success rounded-lg text-sm"
      >
        <CheckCircle :size="20" class="shrink-0" />
        <span>Query saved successfully</span>
      </div>

      <!-- Main content with conditional splitpanes layout -->
      <Splitpanes
        v-if="!isNew && showPreviewPanel"
        class="default-theme"
        @resized="onPaneResized"
      >
        <Pane :size="100 - previewPanelPct" :min-size="40">
          <div class="h-full overflow-y-auto pr-2 space-y-4">
      <!-- Query metadata -->
      <LCard padding="sm">
        <button
          type="button"
          class="flex items-center gap-2 text-sm font-medium text-text hover:text-primary-600 transition-colors mb-3"
          @click="showMetadata = !showMetadata"
        >
          <ChevronDown
            :size="16"
            class="transition-transform"
            :class="{ '-rotate-90': !showMetadata }"
          />
          Query Settings
        </button>

        <div v-if="showMetadata" class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Name</label>
            <LInput v-model="query.name" placeholder="My Query" />
          </div>

          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Datasource</label>
            <LSelect
              v-model="query.datasource_id"
              :options="datasourceOptions"
              placeholder="Select datasource..."
            >
              <template #prefix>
                <Database :size="16" class="text-text-muted" />
              </template>
            </LSelect>
          </div>

          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Timeout</label>
            <div class="relative">
              <LInput v-model.number="query.timeout_seconds" type="number" :min="1" :max="300" />
              <div class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted text-sm">
                <Clock :size="16" />
              </div>
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Max Rows</label>
            <div class="relative">
              <LInput v-model.number="query.max_rows" type="number" :min="1" :max="100000" />
              <div class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted text-sm">
                <Rows3 :size="16" />
              </div>
            </div>
          </div>
        </div>

        <div class="mt-4">
          <label class="block text-sm font-medium text-text mb-1.5">Description (optional)</label>
          <LTextarea
            v-model="query.description"
            placeholder="Describe what this query does..."
            :rows="2"
          />
        </div>

        <!-- Tags -->
        <div class="mt-4">
          <label class="block text-sm font-medium text-text mb-1.5">Tags</label>
          <LTagsInput
            :model-value="query.tags || []"
            @update:model-value="query.tags = $event"
            placeholder="Add tags for filtering..."
          />
        </div>

        <!-- Parameters section -->
        <div class="mt-4">
          <button
            type="button"
            class="flex items-center gap-2 text-sm font-medium text-text hover:text-primary-600 transition-colors"
            @click="showParameters = !showParameters"
          >
            <ChevronDown
              :size="16" class="transition-transform"
              :class="{ '-rotate-90': !showParameters }"
            />
            Parameters
            <span v-if="query.parameters?.length" class="text-xs text-text-muted">
              ({{ query.parameters.length }})
            </span>
          </button>
          <div v-if="showParameters" class="mt-3">
            <QueryParameters
              :model-value="query.parameters || []"
              @update:model-value="query.parameters = $event"
              :sql="query.sql || ''"
            />
          </div>
        </div>
        </div>
      </LCard>

      <!-- Parameter inputs for running -->
      <ParameterInputs
        v-if="query.parameters?.length"
        :parameters="query.parameters"
        v-model="parameterValues"
      />

      <!-- SQL Editor -->
      <LCard padding="none" class="overflow-hidden">
        <div class="p-3 border-b border-border flex items-center justify-between">
          <span class="text-sm font-medium text-text">SQL</span>
          <div class="flex items-center gap-3">
            <button
              type="button"
              class="flex items-center gap-1.5 text-xs text-text-muted hover:text-text transition-colors"
              @click="sqlEditorRef?.format()"
              title="Format SQL (⌘I)"
            >
              <WandSparkles :size="14" />
              Format
            </button>
            <span class="text-xs text-text-subtle">⌘+Enter to run</span>
          </div>
        </div>
        <SqlEditor
          ref="sqlEditorRef"
          :model-value="query.sql ?? ''"
          @update:model-value="query.sql = $event"
          :height="`${editorHeight}px`"
          @run="runQuery"
        />
        <!-- Resize handle -->
        <div
          class="h-2 bg-surface-sunken hover:bg-primary-500/20 cursor-row-resize flex items-center justify-center transition-colors border-t border-border group"
          @mousedown="startResize"
        >
          <GripHorizontal
            :size="12"
            class="text-text-subtle group-hover:text-primary-500 transition-colors"
          />
        </div>
      </LCard>

      <!-- Results panel -->
      <LCard v-if="showResults" padding="none">
        <button
          class="w-full p-3 border-b border-border flex items-center justify-between hover:bg-surface-sunken transition-colors"
          @click="showResults = !showResults"
        >
          <div class="flex items-center gap-3">
            <span class="text-sm font-medium text-text">Results</span>
            <template v-if="currentRun && !running">
              <LBadge :variant="currentRun.status === 'completed' ? 'success' : 'error'" size="sm">
                {{ currentRun.status }}
              </LBadge>
              <span v-if="getRunDuration(currentRun)" class="text-xs text-text-muted">
                {{ formatDuration(getRunDuration(currentRun)!) }}
              </span>
              <span v-if="result" class="text-xs text-text-muted">
                {{ result.rows.length.toLocaleString() }} rows
              </span>
            </template>
          </div>
          <div class="flex items-center gap-2">
            <LButton
              v-if="result && !isNew"
              variant="ghost"
              size="sm"
              @click.stop="
                router.push({ name: 'visualization-new', query: { query_id: query.id || queryId } })
              "
            >
              <BarChart3 :size="16" />
              Visualize
            </LButton>
            <ChevronUp v-if="showResults" :size="16" class="text-text-muted" />
            <ChevronDown v-else :size="16" class="text-text-muted" />
          </div>
        </button>

        <div v-if="showResults" class="max-h-96 overflow-auto">
          <!-- Running state -->
          <div v-if="running" class="flex items-center justify-center py-12">
            <div class="flex flex-col items-center gap-3">
              <LSpinner size="lg" />
              <span class="text-sm text-text-muted">Executing query...</span>
            </div>
          </div>

          <!-- Error state -->
          <div v-else-if="resultError" class="p-4 bg-error-muted text-error text-sm">
            <div class="flex items-start gap-2">
              <AlertCircle :size="20" class="shrink-0 mt-0.5" />
              <pre class="whitespace-pre-wrap font-mono text-xs">{{ resultError }}</pre>
            </div>
          </div>

          <!-- Results table -->
          <div v-else-if="result && result.rows.length > 0" class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="bg-surface-sunken sticky top-0">
                <tr>
                  <th
                    v-for="col in result.columns"
                    :key="col.name"
                    class="px-4 py-2 text-left font-medium text-text-muted border-b border-border whitespace-nowrap"
                  >
                    <div class="flex flex-col">
                      <span>{{ col.name }}</span>
                      <span class="text-xs font-normal text-text-subtle">{{ col.data_type }}</span>
                    </div>
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(row, rowIdx) in result.rows"
                  :key="rowIdx"
                  class="border-b border-border hover:bg-surface-sunken/50"
                >
                  <td
                    v-for="(cell, colIdx) in row"
                    :key="colIdx"
                    class="px-4 py-2 text-text whitespace-nowrap max-w-xs truncate"
                  >
                    <span v-if="cell === null" class="text-text-subtle italic">null</span>
                    <span v-else>{{
                      formatResultCell(cell, result.columns[colIdx]?.data_type)
                    }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Empty results -->
          <div v-else-if="result && result.rows.length === 0" class="p-8 text-center">
            <p class="text-text-muted">Query returned no rows</p>
          </div>
        </div>
      </LCard>
            </div>
          </Pane>

          <!-- Preview panel -->
          <Pane :size="previewPanelPct" :min-size="25" :max-size="60">
          <div class="h-full overflow-y-auto pl-2 space-y-4">
            <!-- Tabs for visualizations and schedules -->
            <div class="flex gap-1 border-b border-border sticky top-0 bg-surface z-10">
              <button
                @click="previewTab = 'visualizations'"
                :class="[
                  'px-3 py-2 text-sm font-medium transition-colors relative',
                  previewTab === 'visualizations'
                    ? 'text-primary-600 after:absolute after:bottom-0 after:left-0 after:right-0 after:h-0.5 after:bg-primary-600'
                    : 'text-text-muted hover:text-text',
                ]"
              >
                <div class="flex items-center gap-2">
                  <BarChart3 :size="14" />
                  Visualizations
                  <span v-if="visualizations.length" class="text-xs">({{ visualizations.length }})</span>
                </div>
              </button>
              <button
                @click="previewTab = 'schedules'"
                :class="[
                  'px-3 py-2 text-sm font-medium transition-colors relative',
                  previewTab === 'schedules'
                    ? 'text-primary-600 after:absolute after:bottom-0 after:left-0 after:right-0 after:h-0.5 after:bg-primary-600'
                    : 'text-text-muted hover:text-text',
                ]"
              >
                <div class="flex items-center gap-2">
                  <CalendarClock :size="14" />
                  Schedules
                  <span v-if="schedules.length" class="text-xs">({{ schedules.length }})</span>
                </div>
              </button>
            </div>

            <!-- Visualizations tab -->
            <div v-if="previewTab === 'visualizations'" class="space-y-3">
              <!-- Inline visualization editor -->
              <div v-if="editingVisualization" class="space-y-3">
                <!-- Header with back button -->
                <div class="flex items-center gap-2 pb-2 border-b border-border">
                  <LButton variant="ghost" size="sm" @click="closeVisualizationEditor">
                    <ArrowLeft :size="14" />
                    Back
                  </LButton>
                  <span class="text-sm font-medium text-text">
                    {{ editingVisualization.id ? 'Edit' : 'New' }} Visualization
                  </span>
                  <div class="flex-1"></div>
                  <LButton size="sm" :loading="savingVisualization" @click="saveInlineVisualization">
                    <Save :size="14" />
                    Save
                  </LButton>
                </div>

                <!-- Basic settings -->
                <div class="space-y-3">
                  <div>
                    <label class="block text-xs text-text-muted mb-1.5">Name</label>
                    <LInput v-model="editingVisualization.name" placeholder="My Visualization" />
                  </div>

                  <div>
                    <label class="block text-xs text-text-muted mb-1.5">Chart Type</label>
                    <div class="grid grid-cols-2 gap-2">
                      <button
                        v-for="opt in chartTypeOptions"
                        :key="opt.value"
                        type="button"
                        class="flex flex-col items-center gap-1 p-2 rounded-lg border transition-colors text-xs"
                        :class="
                          editingVisualization.chart_type === opt.value
                            ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
                            : 'border-border hover:border-border-hover'
                        "
                        @click="editingVisualization.chart_type = opt.value as ChartType"
                      >
                        <component
                          :is="opt.icon"
                          :size="16"
                          :class="[
                            editingVisualization.chart_type === opt.value
                              ? 'text-primary-600'
                              : 'text-text-muted'
                          ]"
                        />
                        <span
                          :class="
                            editingVisualization.chart_type === opt.value
                              ? 'text-primary-600 font-medium'
                              : 'text-text-muted'
                          "
                        >
                          {{ opt.label }}
                        </span>
                      </button>
                    </div>
                  </div>

                  <!-- Line/Bar chart config -->
                  <div
                    v-if="editingVisualization.chart_type === 'line' || editingVisualization.chart_type === 'bar'"
                    class="space-y-3 pt-2 border-t border-border"
                  >
                    <div>
                      <label class="block text-xs text-text-muted mb-1.5">X-Axis Column</label>
                      <LSelect
                        :model-value="editingVisualization.config?.x_axis || ''"
                        @update:model-value="updateVisualizationConfig('x_axis', $event)"
                        :options="vizColumnOptions"
                        placeholder="Select column..."
                      />
                    </div>
                    <div>
                      <label class="block text-xs text-text-muted mb-1.5">Y-Axis Column</label>
                      <LSelect
                        :model-value="editingVisualization.config?.y_axis || ''"
                        @update:model-value="updateVisualizationConfig('y_axis', $event)"
                        :options="vizColumnOptions"
                        placeholder="Select column..."
                      />
                    </div>
                    <div>
                      <label class="block text-xs text-text-muted mb-1.5">Series Column (optional)</label>
                      <LSelect
                        :model-value="editingVisualization.config?.series_column || ''"
                        @update:model-value="updateVisualizationConfig('series_column', $event || undefined)"
                        :options="[{ value: '', label: 'None' }, ...vizColumnOptions]"
                        placeholder="Group by..."
                      />
                    </div>
                  </div>

                  <!-- Single stat config -->
                  <div
                    v-if="editingVisualization.chart_type === 'single_stat'"
                    class="space-y-3 pt-2 border-t border-border"
                  >
                    <div>
                      <label class="block text-xs text-text-muted mb-1.5">Value Column</label>
                      <LSelect
                        :model-value="editingVisualization.config?.value_column || ''"
                        @update:model-value="updateVisualizationConfig('value_column', $event)"
                        :options="vizColumnOptions"
                        placeholder="Select column..."
                      />
                    </div>
                    <div class="grid grid-cols-2 gap-2">
                      <div>
                        <label class="block text-xs text-text-muted mb-1.5">Prefix</label>
                        <LInput
                          :model-value="editingVisualization.config?.prefix || ''"
                          @update:model-value="updateVisualizationConfig('prefix', $event)"
                          placeholder="$"
                        />
                      </div>
                      <div>
                        <label class="block text-xs text-text-muted mb-1.5">Suffix</label>
                        <LInput
                          :model-value="editingVisualization.config?.suffix || ''"
                          @update:model-value="updateVisualizationConfig('suffix', $event)"
                          placeholder="%"
                        />
                      </div>
                    </div>
                  </div>

                  <!-- Pie chart config -->
                  <div
                    v-if="editingVisualization.chart_type === 'pie'"
                    class="space-y-3 pt-2 border-t border-border"
                  >
                    <div>
                      <label class="block text-xs text-text-muted mb-1.5">Label Column</label>
                      <LSelect
                        :model-value="editingVisualization.config?.label_column || ''"
                        @update:model-value="updateVisualizationConfig('label_column', $event)"
                        :options="vizColumnOptions"
                        placeholder="Select column..."
                      />
                    </div>
                    <div>
                      <label class="block text-xs text-text-muted mb-1.5">Value Column</label>
                      <LSelect
                        :model-value="editingVisualization.config?.value_column || ''"
                        @update:model-value="updateVisualizationConfig('value_column', $event)"
                        :options="vizColumnOptions"
                        placeholder="Select column..."
                      />
                    </div>
                  </div>

                  <!-- Tags -->
                  <div class="pt-2 border-t border-border">
                    <label class="block text-xs text-text-muted mb-1.5">Tags</label>
                    <LTagsInput
                      :model-value="editingVisualization.tags || []"
                      @update:model-value="editingVisualization.tags = $event"
                      placeholder="Add tags..."
                    />
                  </div>
                </div>

                <!-- Preview -->
                <div class="pt-2 border-t border-border">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs font-medium text-text">Preview</span>
                    <LButton
                      variant="ghost"
                      size="sm"
                      :disabled="visualizationPreviewRunning"
                      @click="runVisualizationPreview"
                    >
                      <Loader2 v-if="visualizationPreviewRunning" :size="12" class="animate-spin" />
                      <Play v-else :size="12" />
                      Refresh
                    </LButton>
                  </div>
                  <div class="border border-border rounded-lg overflow-hidden bg-surface-raised">
                    <VisualizationRenderer
                      :chart-type="editingVisualization.chart_type || 'table'"
                      :data="visualizationPreviewResult"
                      :config="editingVisualization.config || {}"
                      :loading="visualizationPreviewRunning"
                      height="300px"
                    />
                  </div>
                </div>
              </div>

              <!-- Visualizations list (when not editing) -->
              <div v-else>
                <div class="flex items-center justify-between mb-3">
                  <LButton size="sm" @click="createNewVisualization">
                    <Plus :size="14" />
                    Create Visualization
                  </LButton>
                </div>

                <!-- Loading state -->
                <div v-if="loadingVisualizations" class="flex items-center justify-center py-12">
                  <LSpinner />
                </div>

                <!-- Empty state -->
                <div v-else-if="visualizations.length === 0" class="text-center py-12">
                  <BarChart3 :size="48" class="mx-auto text-text-muted mb-3" />
                  <p class="text-sm text-text-muted mb-4">No visualizations yet</p>
                  <LButton size="sm" @click="createNewVisualization">
                    <Plus :size="14" />
                    Create Visualization
                  </LButton>
                </div>

                <!-- Visualizations list -->
                <div v-else class="space-y-2">
                  <LCard
                    v-for="viz in visualizations"
                    :key="viz.id"
                    padding="sm"
                    class="cursor-pointer hover:border-primary-500/50 hover:shadow-md transition-all"
                    @click="openVisualizationEditor(viz)"
                  >
                    <div class="flex items-center gap-2 mb-1">
                      <BarChart3 :size="16" class="text-primary-600" />
                      <span class="text-sm font-medium text-text truncate">{{ viz.name }}</span>
                    </div>
                    <p class="text-xs text-text-muted capitalize">{{ viz.chart_type }} chart</p>
                  </LCard>
                </div>
              </div>
            </div>

            <!-- Schedules tab -->
            <div v-if="previewTab === 'schedules'" class="space-y-3">
              <div class="flex items-center justify-between">
                <LButton
                  size="sm"
                  @click="router.push({ name: 'schedule-new', query: { query_id: query.id || queryId } })"
                >
                  <Plus :size="14" />
                  Create Schedule
                </LButton>
              </div>

              <!-- Empty state -->
              <div v-if="schedules.length === 0" class="text-center py-12">
                <CalendarClock :size="48" class="mx-auto text-text-muted mb-3" />
                <p class="text-sm text-text-muted mb-4">No schedules yet</p>
                <LButton
                  size="sm"
                  @click="router.push({ name: 'schedule-new', query: { query_id: query.id || queryId } })"
                >
                  <Plus :size="14" />
                  Create Schedule
                </LButton>
              </div>

              <!-- Schedules list -->
              <div v-else class="space-y-2">
                <LCard
                  v-for="sched in schedules"
                  :key="sched.id"
                  padding="sm"
                  class="cursor-pointer hover:border-primary-500/50 hover:shadow-md transition-all"
                  @click="router.push({ name: 'schedule-editor', params: { id: sched.id } })"
                >
                  <div class="flex items-center gap-2 mb-1">
                    <CalendarClock :size="16" class="text-primary-600" />
                    <span class="text-sm font-medium text-text truncate">{{ sched.name }}</span>
                    <LBadge :variant="sched.enabled ? 'success' : 'default'" size="sm">
                      {{ sched.enabled ? 'Active' : 'Paused' }}
                    </LBadge>
                  </div>
                  <p class="text-xs text-text-muted font-mono">{{ sched.cron_expression }}</p>
                </LCard>
              </div>
            </div>
          </div>
          </Pane>
        </Splitpanes>

      <!-- When preview panel is hidden or creating new query, redirect to enable it -->
      <div v-else class="text-center py-12">
        <p class="text-text-muted mb-4">Preview panel is hidden</p>
        <LButton size="sm" @click="showPreviewPanel = true">
          <Eye :size="14" />
          Show Preview Panel
        </LButton>
      </div>
    </div>
  </AppLayout>
</template>
