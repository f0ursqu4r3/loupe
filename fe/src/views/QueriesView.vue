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
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner, LTagFilter } from '@/components/ui'
import { ParameterInputs } from '@/components/query'
import { queriesApi, runsApi } from '@/services/api'
import type { Query, Run, RunStatus } from '@/types'

const router = useRouter()
const queries = ref<Query[]>([])
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
    case 'datetime':
      return new Date().toISOString().split('T')[0]
    default:
      return ''
  }
}

onMounted(loadQueries)

function formatDate(dateString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(new Date(dateString))
}

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
</script>

<template>
  <AppLayout title="Queries">
    <template #header-actions>
      <LButton @click="openEditor()">
        <Plus class="h-4 w-4" />
        New Query
      </LButton>
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
  </AppLayout>
</template>
