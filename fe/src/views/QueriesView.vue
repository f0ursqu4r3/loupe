<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  Plus,
  Play,
  FileCode,
  Edit,
  Loader2,
  Clock,
  CheckCircle,
  XCircle,
  Tag,
  Download,
  Upload,
} from 'lucide-vue-next'
import { format as formatSql } from 'sql-formatter'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner, LTagFilter } from '@/components/ui'
import { ParameterInputs } from '@/components/query'
import { queriesApi, runsApi, datasourcesApi } from '@/services/api'
import type { Query, Run, RunStatus, Datasource, QueryExport } from '@/types'

const router = useRouter()
const queries = ref<Query[]>([])
const datasources = ref<Datasource[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

// Track last run per query
const lastRuns = ref<Record<string, Run | null>>({})

// Track parameter values per query
const paramValues = reactive<Record<string, Record<string, unknown>>>({})

// Track running state per query
const runningQueries = ref<Set<string>>(new Set())

// Tag filtering
const selectedTags = ref<string[]>([])

// Import modal state
const showImportModal = ref(false)
const importFile = ref<File | null>(null)
const importDatasourceId = ref<string>('')
const importSkipDuplicates = ref(true)
const importFormatSql = ref(false)
const importing = ref(false)
const importResult = ref<{ imported: number; skipped: number; skipped_names: string[] } | null>(
  null,
)

// Get all unique tags across queries
const allTags = computed(() => {
  const tags = new Set<string>()
  for (const query of queries.value) {
    for (const tag of query.tags || []) {
      tags.add(tag)
    }
  }
  return Array.from(tags)
})

// Filter queries by selected tags
const filteredQueries = computed(() => {
  if (selectedTags.value.length === 0) return queries.value
  return queries.value.filter((q) =>
    selectedTags.value.every((tag) => (q.tags || []).includes(tag)),
  )
})

async function loadQueries() {
  try {
    loading.value = true
    error.value = null
    queries.value = await queriesApi.list()
    datasources.value = await datasourcesApi.list()

    // Set default datasource for import
    if (datasources.value.length > 0 && !importDatasourceId.value) {
      importDatasourceId.value = datasources.value[0]!.id
    }

    // Load last run for each query
    for (const query of queries.value) {
      // Initialize param values with defaults
      const queryParams: Record<string, unknown> = {}
      for (const param of query.parameters) {
        queryParams[param.name] = param.default ?? getDefaultForType(param.param_type)
      }
      paramValues[query.id] = queryParams

      // Fetch last run
      try {
        const runs = await runsApi.list(query.id)
        lastRuns.value[query.id] = runs.length > 0 ? runs[0]! : null
      } catch {
        lastRuns.value[query.id] = null
      }
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load queries'
  } finally {
    loading.value = false
  }
}

function getDefaultForType(type: string): unknown {
  switch (type) {
    case 'number':
      return 0
    case 'boolean':
      return false
    case 'date':
      return getLocalDateString()
    case 'datetime':
      return new Date().toISOString()
    default:
      return ''
  }
}

function getLocalDateString(): string {
  const date = new Date()
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

onMounted(loadQueries)

function formatDateTime(dateString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(new Date(dateString))
}

function getStatusColor(status: RunStatus): string {
  switch (status) {
    case 'completed':
      return 'text-success'
    case 'failed':
    case 'timeout':
    case 'cancelled':
      return 'text-error'
    case 'running':
    case 'queued':
      return 'text-warning'
    default:
      return 'text-text-muted'
  }
}

function openEditor(queryId?: string) {
  if (queryId) {
    router.push({ name: 'query-editor', params: { id: queryId } })
  } else {
    router.push({ name: 'query-new' })
  }
}

async function runQuery(query: Query, event: Event) {
  event.stopPropagation()

  if (runningQueries.value.has(query.id)) return

  runningQueries.value.add(query.id)

  try {
    // Create a run with current parameter values
    const run = await runsApi.create({
      query_id: query.id,
      parameters: paramValues[query.id] || {},
    })

    // Poll for completion
    let currentRun = run
    while (currentRun.status === 'queued' || currentRun.status === 'running') {
      await new Promise((resolve) => setTimeout(resolve, 500))
      currentRun = await runsApi.get(run.id)
    }

    lastRuns.value[query.id] = currentRun
  } catch (e) {
    console.error('Failed to run query:', e)
  } finally {
    runningQueries.value.delete(query.id)
  }
}

function updateParamValues(queryId: string, values: Record<string, unknown>) {
  paramValues[queryId] = values
}

// Export queries to JSON file
async function exportQueries() {
  try {
    const exports = await queriesApi.export()
    const blob = new Blob([JSON.stringify(exports, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `queries-export-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (e) {
    console.error('Failed to export queries:', e)
  }
}

// Handle file selection for import
function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files && input.files.length > 0) {
    importFile.value = input.files[0]!
  }
}

// Import queries from JSON file
async function importQueries() {
  if (!importFile.value || !importDatasourceId.value) return

  importing.value = true
  importResult.value = null

  try {
    const text = await importFile.value.text()
    let queriesData: QueryExport[] = JSON.parse(text)

    // Format SQL if option is enabled
    if (importFormatSql.value) {
      queriesData = queriesData.map((q) => ({
        ...q,
        sql: formatSql(q.sql, {
          language: 'postgresql',
          tabWidth: 2,
          keywordCase: 'upper',
          linesBetweenQueries: 2,
        }),
      }))
    }

    const result = await queriesApi.import({
      queries: queriesData,
      datasource_id: importDatasourceId.value,
      skip_duplicates: importSkipDuplicates.value,
    })

    importResult.value = result

    // Reload queries if any were imported
    if (result.imported > 0) {
      await loadQueries()
    }
  } catch (e) {
    console.error('Failed to import queries:', e)
    error.value = e instanceof Error ? e.message : 'Failed to import queries'
  } finally {
    importing.value = false
  }
}

function closeImportModal() {
  showImportModal.value = false
  importFile.value = null
  importResult.value = null
}
</script>

<template>
  <AppLayout title="Queries">
    <template #header-actions>
      <div class="flex items-center gap-2">
        <LButton variant="secondary" @click="exportQueries" :disabled="queries.length === 0">
          <Download class="h-4 w-4" />
          Export
        </LButton>
        <LButton variant="secondary" @click="showImportModal = true">
          <Upload class="h-4 w-4" />
          Import
        </LButton>
        <LButton @click="openEditor()">
          <Plus class="h-4 w-4" />
          New Query
        </LButton>
      </div>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <!-- Empty state -->
    <LEmptyState
      v-else-if="queries.length === 0"
      title="No queries yet"
      description="Write your first SQL query to start exploring your data."
    >
      <template #icon>
        <FileCode class="h-8 w-8 text-text-subtle" />
      </template>
      <template #action>
        <LButton @click="openEditor()">
          <Plus class="h-4 w-4" />
          Create Query
        </LButton>
      </template>
    </LEmptyState>

    <!-- Queries list -->
    <div v-else class="space-y-4">
      <!-- Tag filter -->
      <LTagFilter
        v-if="allTags.length > 0"
        :all-tags="allTags"
        :selected-tags="selectedTags"
        @update:selected-tags="selectedTags = $event"
      />

      <!-- Empty filter result -->
      <div
        v-if="filteredQueries.length === 0 && selectedTags.length > 0"
        class="text-center py-8 text-text-muted"
      >
        No queries match the selected tags
      </div>

      <LCard
        v-for="query in filteredQueries"
        :key="query.id"
        class="group hover:border-primary-500/50 transition-colors cursor-pointer"
        @click="openEditor(query.id)"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <h3 class="font-medium text-text truncate">{{ query.name }}</h3>
              <LBadge v-if="query.parameters.length > 0" size="sm">
                {{ query.parameters.length }} params
              </LBadge>
            </div>
            <!-- Tags display -->
            <div v-if="query.tags && query.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
              <LBadge v-for="tag in query.tags" :key="tag" size="sm">
                <Tag class="h-3 w-3 mr-1" />
                {{ tag }}
              </LBadge>
            </div>
            <p v-if="query.description" class="text-sm text-text-muted mb-3 line-clamp-2">
              {{ query.description }}
            </p>
            <pre
              class="text-xs text-text-subtle bg-surface-sunken p-2 rounded overflow-x-auto max-h-20"
              >{{ query.sql }}</pre
            >
          </div>

          <div class="flex items-center gap-2 ml-4" @click.stop>
            <LButton variant="ghost" size="sm" @click="openEditor(query.id)">
              <Edit class="h-4 w-4" />
            </LButton>
            <LButton
              variant="ghost"
              size="sm"
              :disabled="runningQueries.has(query.id)"
              @click="runQuery(query, $event)"
            >
              <Loader2 v-if="runningQueries.has(query.id)" class="h-4 w-4 animate-spin" />
              <Play v-else class="h-4 w-4" />
            </LButton>
          </div>
        </div>

        <!-- Parameter inputs -->
        <div v-if="query.parameters.length > 0" class="mt-3" @click.stop>
          <ParameterInputs
            :parameters="query.parameters"
            :model-value="paramValues[query.id] || {}"
            @update:model-value="updateParamValues(query.id, $event)"
          />
        </div>

        <div class="flex items-center justify-between mt-4 pt-3 border-t border-border">
          <!-- Last run info -->
          <div class="flex items-center gap-2 text-xs">
            <template v-if="lastRuns[query.id]">
              <Clock class="h-3.5 w-3.5 text-text-subtle" />
              <span class="text-text-subtle">Last run:</span>
              <span :class="getStatusColor(lastRuns[query.id]!.status)">
                {{ lastRuns[query.id]!.status }}
              </span>
              <span class="text-text-subtle">
                {{ formatDateTime(lastRuns[query.id]!.created_at) }}
              </span>
              <CheckCircle
                v-if="lastRuns[query.id]!.status === 'completed'"
                class="h-3.5 w-3.5 text-success"
              />
              <XCircle
                v-else-if="
                  lastRuns[query.id]!.status === 'failed' ||
                  lastRuns[query.id]!.status === 'timeout'
                "
                class="h-3.5 w-3.5 text-error"
              />
            </template>
            <span v-else class="text-text-subtle">Never run</span>
          </div>

          <div class="flex items-center gap-2 text-xs text-text-subtle">
            <span>Timeout: {{ query.timeout_seconds }}s</span>
            <span>·</span>
            <span>Max rows: {{ query.max_rows.toLocaleString() }}</span>
          </div>
        </div>
      </LCard>
    </div>

    <!-- Import Modal -->
    <Teleport to="body">
      <div
        v-if="showImportModal"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
        @click.self="closeImportModal"
      >
        <div class="bg-surface rounded-lg shadow-xl max-w-md w-full mx-4 p-6">
          <h2 class="text-lg font-semibold text-text mb-4">Import Queries</h2>

          <!-- Import result -->
          <div v-if="importResult" class="mb-4">
            <div class="p-3 rounded bg-success/10 text-success" v-if="importResult.imported > 0">
              Successfully imported {{ importResult.imported }} queries
            </div>
            <div
              v-if="importResult.skipped > 0"
              class="mt-2 p-3 rounded bg-warning/10 text-warning"
            >
              Skipped {{ importResult.skipped }} duplicate(s):
              <ul class="text-sm mt-1">
                <li v-for="name in importResult.skipped_names" :key="name">• {{ name }}</li>
              </ul>
            </div>
            <div class="mt-4 flex justify-end">
              <LButton @click="closeImportModal">Done</LButton>
            </div>
          </div>

          <!-- Import form -->
          <template v-else>
            <div class="space-y-4">
              <!-- File input -->
              <div>
                <label class="block text-sm font-medium text-text mb-1">JSON File</label>
                <input
                  type="file"
                  accept=".json,application/json"
                  @change="handleFileSelect"
                  class="block w-full text-sm text-text-muted file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:font-medium file:bg-primary-500 file:text-white hover:file:bg-primary-600 cursor-pointer"
                />
              </div>

              <!-- Datasource selector -->
              <div>
                <label class="block text-sm font-medium text-text mb-1">Target Datasource</label>
                <select
                  v-model="importDatasourceId"
                  class="w-full px-3 py-2 bg-surface-sunken border border-border rounded text-text"
                >
                  <option v-for="ds in datasources" :key="ds.id" :value="ds.id">
                    {{ ds.name }}
                  </option>
                </select>
                <p class="text-xs text-text-muted mt-1">
                  All imported queries will use this datasource
                </p>
              </div>

              <!-- Skip duplicates toggle -->
              <label class="flex items-center gap-2">
                <input
                  type="checkbox"
                  v-model="importSkipDuplicates"
                  class="rounded border-border"
                />
                <span class="text-sm text-text">Skip queries with duplicate names</span>
              </label>

              <!-- Format SQL toggle -->
              <label class="flex items-center gap-2">
                <input type="checkbox" v-model="importFormatSql" class="rounded border-border" />
                <span class="text-sm text-text">Format SQL on import</span>
              </label>
            </div>

            <div class="flex justify-end gap-2 mt-6">
              <LButton variant="secondary" @click="closeImportModal">Cancel</LButton>
              <LButton
                @click="importQueries"
                :disabled="!importFile || !importDatasourceId || importing"
              >
                <Loader2 v-if="importing" class="h-4 w-4 animate-spin" />
                <Upload v-else class="h-4 w-4" />
                Import
              </LButton>
            </div>
          </template>
        </div>
      </div>
    </Teleport>
  </AppLayout>
</template>
