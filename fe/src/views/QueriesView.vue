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
  Eye,
} from 'lucide-vue-next'
import { format as formatSql } from 'sql-formatter'
import { AppLayout } from '@/components/layout'
import {
  LButton,
  LCard,
  LCheckbox,
  LBadge,
  LEmptyState,
  LSpinner,
  LTagFilter,
} from '@/components/ui'
import { ParameterInputs } from '@/components/query'
import { queriesApi, runsApi, datasourcesApi } from '@/services/api'
import { formatDateTimeShort } from '@/utils/dateTime'
import { usePermissions } from '@/composables/usePermissions'
import { useApiError } from '@/composables/useApiError'
import type { Query, Run, RunStatus, Datasource, QueryExport } from '@/types'

const router = useRouter()
const { canEdit, role } = usePermissions()
const { handleError } = useApiError()

const isViewer = computed(() => role.value === 'viewer')

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
const showCanvasQueries = ref(false)

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

// Get all unique tags across queries (excludes 'canvas' when toggle is off)
const allTags = computed(() => {
  const tags = new Set<string>()
  for (const query of queries.value) {
    for (const tag of query.tags || []) {
      if (tag === 'canvas' && !showCanvasQueries.value) continue
      tags.add(tag)
    }
  }
  return Array.from(tags)
})

// Filter queries by selected tags (excludes 'canvas' queries when toggle is off)
const filteredQueries = computed(() => {
  let result = queries.value

  if (!showCanvasQueries.value) {
    result = result.filter((q) => !(q.tags || []).includes('canvas'))
  }

  if (selectedTags.value.length > 0) {
    result = result.filter((q) => selectedTags.value.every((tag) => (q.tags || []).includes(tag)))
  }

  return result
})

// Check if there are hidden canvas queries
const hasHiddenCanvasQueries = computed(() => {
  if (showCanvasQueries.value) return false
  return queries.value.some((q) => (q.tags || []).includes('canvas'))
})

async function loadQueries() {
  try {
    loading.value = true
    error.value = null
    const [queriesResponse, datasourcesResponse] = await Promise.all([
      queriesApi.list(),
      datasourcesApi.list()
    ])
    queries.value = queriesResponse.data
    datasources.value = datasourcesResponse.data

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
        const runsResponse = await runsApi.list({ query_id: query.id, limit: 1 })
        lastRuns.value[query.id] = runsResponse.data.length > 0 ? runsResponse.data[0]! : null
      } catch {
        lastRuns.value[query.id] = null
      }
    }
  } catch (e) {
    handleError(e, 'Failed to load queries')
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
  return formatDateTimeShort(dateString)
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
          <Download :size="16" />
          Export
        </LButton>
        <LButton v-if="canEdit" variant="secondary" @click="showImportModal = true">
          <Upload :size="16" />
          Import
        </LButton>
        <LButton v-if="canEdit" @click="openEditor()">
          <Plus :size="16" />
          New Query
        </LButton>
      </div>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <!-- Empty state (no queries or only canvas queries) -->
    <div v-else-if="filteredQueries.length === 0 && selectedTags.length === 0" class="space-y-4">
      <div v-if="hasHiddenCanvasQueries" class="flex justify-end">
        <LCheckbox v-model="showCanvasQueries" label="Show canvas queries" />
      </div>
      <LEmptyState
        title="No queries yet"
        :description="canEdit ? 'Write your first SQL query to start exploring your data.' : 'No queries have been created yet. Contact an editor or admin to create queries.'"
      >
        <template #icon>
          <FileCode :size="48" class="text-text-subtle" />
        </template>
        <template #action>
          <LButton v-if="canEdit" @click="openEditor()">
            <Plus :size="16" />
            Create Query
          </LButton>
        </template>
      </LEmptyState>
    </div>

    <!-- Queries list -->
    <div v-else class="space-y-4">
      <!-- Tag filter and canvas toggle -->
      <div class="flex items-center justify-between gap-4">
        <LTagFilter
          v-if="allTags.length > 0"
          :all-tags="allTags"
          :selected-tags="selectedTags"
          @update:selected-tags="selectedTags = $event"
        />
        <div class="m-auto"></div>
        <LCheckbox v-model="showCanvasQueries" label="Show canvas queries" />
      </div>

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
            <div class="flex items-center gap-2 mb-1 flex-wrap">
              <h3 class="font-medium text-text truncate">{{ query.name }}</h3>
              <LBadge v-if="isViewer" variant="info" size="sm">
                <Eye :size="12" class="mr-1" />
                Read-only
              </LBadge>
              <LBadge v-if="query.parameters.length > 0" size="sm">
                {{ query.parameters.length }} params
              </LBadge>
            </div>
            <!-- Tags display -->
            <div v-if="query.tags && query.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
              <LBadge v-for="tag in query.tags" :key="tag" size="sm">
                <Tag :size="12" class="mr-1" />
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
            <LButton v-if="canEdit" variant="ghost" size="sm" @click="openEditor(query.id)">
              <Edit :size="16" />
            </LButton>
            <LButton
              variant="ghost"
              size="sm"
              :disabled="runningQueries.has(query.id)"
              @click="runQuery(query, $event)"
            >
              <Loader2 v-if="runningQueries.has(query.id)" :size="16" class="animate-spin" />
              <Play v-else :size="16" />
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
              <Clock :size="14" class="text-text-subtle" />
              <span class="text-text-subtle">Last run:</span>
              <span :class="getStatusColor(lastRuns[query.id]!.status)">
                {{ lastRuns[query.id]!.status }}
              </span>
              <span class="text-text-subtle">
                {{ formatDateTime(lastRuns[query.id]!.created_at) }}
              </span>
              <CheckCircle
                v-if="lastRuns[query.id]!.status === 'completed'"
                :size="14"
                class="text-success"
              />
              <XCircle
                v-else-if="
                  lastRuns[query.id]!.status === 'failed' ||
                  lastRuns[query.id]!.status === 'timeout'
                "
                :size="14"
                class="text-error"
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
                <Loader2 v-if="importing" :size="16" class="animate-spin" />
                <Upload v-else :size="16" />
                Import
              </LButton>
            </div>
          </template>
        </div>
      </div>
    </Teleport>
  </AppLayout>
</template>
, LCheckbox
