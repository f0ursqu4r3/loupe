<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Play,
  Save,
  ArrowLeft,
  Database,
  Clock,
  Rows3,
  Loader2,
  ChevronDown,
  ChevronUp,
  AlertCircle,
  CheckCircle,
  X,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LInput, LTextarea, LSelect, LCard, LSpinner, LBadge } from '@/components/ui'
import { SqlEditor } from '@/components/editor'
import { queriesApi, runsApi, datasourcesApi } from '@/services/api'
import type { Query, Datasource, Run, QueryResult, CreateQueryRequest } from '@/types'

const route = useRoute()
const router = useRouter()

// Query state
const queryId = computed(() => route.params.id as string | undefined)
const isNew = computed(() => !queryId.value || queryId.value === 'new')

const query = ref<Partial<Query>>({
  name: '',
  description: '',
  sql: 'SELECT * FROM ',
  datasource_id: '',
  parameters: [],
  timeout_seconds: 30,
  max_rows: 10000,
})

// UI state
const loading = ref(false)
const saving = ref(false)
const running = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)

// Datasources
const datasources = ref<Datasource[]>([])
const datasourceOptions = computed(() =>
  datasources.value.map((ds) => ({
    value: ds.id,
    label: ds.name,
  })),
)

// Run results
const currentRun = ref<Run | null>(null)
const result = ref<QueryResult | null>(null)
const resultError = ref<string | null>(null)
const showResults = ref(false)

// Load datasources
async function loadDatasources() {
  try {
    datasources.value = await datasourcesApi.list()
    // Auto-select first datasource for new queries
    if (isNew.value && datasources.value.length > 0 && !query.value.datasource_id) {
      query.value.datasource_id = datasources.value[0].id
    }
  } catch (e) {
    console.error('Failed to load datasources:', e)
  }
}

// Load existing query
async function loadQuery() {
  if (isNew.value) return

  try {
    loading.value = true
    const data = await queriesApi.get(queryId.value!)
    query.value = data
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load query'
  } finally {
    loading.value = false
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

  // For new queries, we need to save first
  if (isNew.value) {
    await saveQuery()
    if (error.value) return
  }

  try {
    running.value = true
    resultError.value = null
    result.value = null
    showResults.value = true

    // Execute the query
    currentRun.value = await queriesApi.execute(query.value.id || queryId.value!, {
      parameters: {},
    })

    // Poll for completion
    await pollRunStatus()
  } catch (e) {
    resultError.value = e instanceof Error ? e.message : 'Failed to execute query'
    running.value = false
  }
}

// Poll run status until complete
async function pollRunStatus() {
  if (!currentRun.value) return

  const maxAttempts = 60
  let attempts = 0

  while (attempts < maxAttempts) {
    try {
      const run = await runsApi.get(currentRun.value.id)
      currentRun.value = run

      if (run.status === 'succeeded') {
        // Fetch results
        result.value = await runsApi.getResult(run.id)
        running.value = false
        return
      } else if (run.status === 'failed' || run.status === 'canceled') {
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

  resultError.value = 'Query timed out waiting for results'
  running.value = false
}

// Format duration
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

onMounted(() => {
  loadDatasources()
  loadQuery()
})

// Clear error when inputs change
watch([() => query.value.name, () => query.value.sql, () => query.value.datasource_id], () => {
  error.value = null
})
</script>

<template>
  <AppLayout :title="isNew ? 'New Query' : query.name || 'Query Editor'">
    <template #header-left>
      <LButton variant="ghost" size="sm" @click="router.push({ name: 'queries' })">
        <ArrowLeft class="h-4 w-4" />
        Back
      </LButton>
    </template>

    <template #header-actions>
      <LButton variant="secondary" :disabled="saving" @click="saveQuery">
        <Loader2 v-if="saving" class="h-4 w-4 animate-spin" />
        <Save v-else class="h-4 w-4" />
        {{ saving ? 'Saving...' : 'Save' }}
      </LButton>
      <LButton :disabled="running || !query.sql?.trim()" @click="runQuery">
        <Loader2 v-if="running" class="h-4 w-4 animate-spin" />
        <Play v-else class="h-4 w-4" />
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
        <AlertCircle class="h-5 w-5 flex-shrink-0" />
        <span class="flex-1">{{ error }}</span>
        <button @click="error = null" class="p-1 hover:bg-error/20 rounded">
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Success banner -->
      <div
        v-if="saveSuccess"
        class="flex items-center gap-3 p-3 bg-success-muted text-success rounded-lg text-sm"
      >
        <CheckCircle class="h-5 w-5 flex-shrink-0" />
        <span>Query saved successfully</span>
      </div>

      <!-- Query metadata -->
      <LCard padding="sm">
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
                <Database class="h-4 w-4 text-text-muted" />
              </template>
            </LSelect>
          </div>

          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Timeout</label>
            <div class="relative">
              <LInput v-model.number="query.timeout_seconds" type="number" :min="1" :max="300" />
              <div class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted text-sm">
                <Clock class="h-4 w-4" />
              </div>
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Max Rows</label>
            <div class="relative">
              <LInput v-model.number="query.max_rows" type="number" :min="1" :max="100000" />
              <div class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted text-sm">
                <Rows3 class="h-4 w-4" />
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
      </LCard>

      <!-- SQL Editor -->
      <LCard padding="none">
        <div class="p-3 border-b border-border flex items-center justify-between">
          <span class="text-sm font-medium text-text">SQL</span>
          <span class="text-xs text-text-subtle">Press ⌘+Enter to run</span>
        </div>
        <SqlEditor v-model="query.sql" height="300px" @run="runQuery" />
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
              <LBadge :variant="currentRun.status === 'succeeded' ? 'success' : 'error'" size="sm">
                {{ currentRun.status }}
              </LBadge>
              <span v-if="currentRun.duration_ms" class="text-xs text-text-muted">
                {{ formatDuration(currentRun.duration_ms) }}
              </span>
              <span v-if="result" class="text-xs text-text-muted">
                {{ result.rows.length.toLocaleString() }} rows
              </span>
            </template>
          </div>
          <ChevronUp v-if="showResults" class="h-4 w-4 text-text-muted" />
          <ChevronDown v-else class="h-4 w-4 text-text-muted" />
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
              <AlertCircle class="h-5 w-5 flex-shrink-0 mt-0.5" />
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
                      <span class="text-xs font-normal text-text-subtle">{{ col.type }}</span>
                    </div>
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(row, idx) in result.rows"
                  :key="idx"
                  class="border-b border-border hover:bg-surface-sunken/50"
                >
                  <td
                    v-for="col in result.columns"
                    :key="col.name"
                    class="px-4 py-2 text-text whitespace-nowrap max-w-xs truncate"
                  >
                    <span v-if="row[col.name] === null" class="text-text-subtle italic">null</span>
                    <span v-else>{{ row[col.name] }}</span>
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
  </AppLayout>
</template>
