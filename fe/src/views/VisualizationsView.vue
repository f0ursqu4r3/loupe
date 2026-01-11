<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  Plus,
  BarChart3,
  Table,
  LineChart,
  Hash,
  Trash2,
  PieChart,
  LayoutGrid,
  List,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LSpinner, LModal } from '@/components/ui'
import { visualizationsApi, queriesApi } from '@/services/api'
import type { Visualization, Query, ChartType } from '@/types'

const router = useRouter()

const visualizations = ref<Visualization[]>([])
const queries = ref<Query[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

// View mode: 'grid' or 'list'
const viewMode = ref<'grid' | 'list'>('grid')

// Modal state
const showNewModal = ref(false)
const selectedQueryId = ref<string | null>(null)
const deleting = ref<string | null>(null)

async function loadVisualizations() {
  try {
    loading.value = true
    error.value = null
    const [vizs, qs] = await Promise.all([visualizationsApi.list(), queriesApi.list()])
    visualizations.value = vizs
    queries.value = qs
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
  pie: PieChart,
}

const chartTypeLabels: Record<ChartType, string> = {
  table: 'Table',
  bar: 'Bar Chart',
  line: 'Line Chart',
  single_stat: 'Single Stat',
  pie: 'Pie Chart',
}

function openVisualization(viz: Visualization) {
  router.push({ name: 'visualization-editor', params: { id: viz.id } })
}

function createFromQuery() {
  if (!selectedQueryId.value) return
  showNewModal.value = false
  router.push({ name: 'visualization-new', query: { query_id: selectedQueryId.value } })
}

async function deleteVisualization(id: string, event: Event) {
  event.stopPropagation()
  if (!confirm('Are you sure you want to delete this visualization?')) return

  try {
    deleting.value = id
    await visualizationsApi.delete(id)
    visualizations.value = visualizations.value.filter((v) => v.id !== id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to delete visualization'
  } finally {
    deleting.value = null
  }
}
</script>

<template>
  <AppLayout title="Visualizations">
    <template #header-actions>
      <!-- View toggle -->
      <div class="flex items-center border border-border rounded-lg overflow-hidden mr-2">
        <button
          type="button"
          class="p-2 transition-colors"
          :class="
            viewMode === 'grid'
              ? 'bg-primary-100 dark:bg-primary-900 text-primary-600'
              : 'text-text-muted hover:text-text hover:bg-surface-sunken'
          "
          @click="viewMode = 'grid'"
          title="Grid view"
        >
          <LayoutGrid class="h-4 w-4" />
        </button>
        <button
          type="button"
          class="p-2 transition-colors"
          :class="
            viewMode === 'list'
              ? 'bg-primary-100 dark:bg-primary-900 text-primary-600'
              : 'text-text-muted hover:text-text hover:bg-surface-sunken'
          "
          @click="viewMode = 'list'"
          title="List view"
        >
          <List class="h-4 w-4" />
        </button>
      </div>
      <LButton @click="showNewModal = true">
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
        <LButton @click="showNewModal = true">
          <Plus class="h-4 w-4" />
          Create Visualization
        </LButton>
      </template>
    </LEmptyState>

    <!-- Visualizations grid -->
    <div
      v-else
      :class="
        viewMode === 'grid' ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6' : 'space-y-3'
      "
    >
      <!-- Grid view card -->
      <LCard
        v-if="viewMode === 'grid'"
        v-for="viz in visualizations"
        :key="viz.id"
        class="group hover:border-primary-500/50 transition-colors cursor-pointer"
        @click="openVisualization(viz)"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div
              class="w-10 h-10 shrink-0 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center"
            >
              <component :is="chartTypeIcons[viz.chart_type]" class="h-5 w-5 text-primary-600" />
            </div>
            <div>
              <h3 class="font-medium text-text group-hover:text-primary-600 transition-colors">
                {{ viz.name }}
              </h3>
              <span class="text-xs text-text-muted">{{ chartTypeLabels[viz.chart_type] }}</span>
            </div>
          </div>
          <button
            type="button"
            class="opacity-0 group-hover:opacity-100 p-1.5 rounded hover:bg-error-muted text-text-muted hover:text-error transition-all"
            @click="deleteVisualization(viz.id, $event)"
            :disabled="deleting === viz.id"
          >
            <Trash2 class="h-4 w-4" />
          </button>
        </div>

        <!-- Preview placeholder -->
        <div class="h-32 bg-surface-sunken rounded-lg flex items-center justify-center mb-3">
          <component :is="chartTypeIcons[viz.chart_type]" class="h-12 w-12 text-text-subtle/30" />
        </div>

        <div class="text-xs text-text-subtle">Updated {{ formatDate(viz.updated_at) }}</div>
      </LCard>

      <!-- List view row -->
      <LCard
        v-if="viewMode === 'list'"
        v-for="viz in visualizations"
        :key="viz.id"
        padding="sm"
        class="group hover:border-primary-500/50 transition-colors cursor-pointer"
        @click="openVisualization(viz)"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div
              class="w-10 h-10 shrink-0 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center"
            >
              <component :is="chartTypeIcons[viz.chart_type]" class="h-5 w-5 text-primary-600" />
            </div>
            <div>
              <h3 class="font-medium text-text group-hover:text-primary-600 transition-colors">
                {{ viz.name }}
              </h3>
              <span class="text-xs text-text-muted">{{ chartTypeLabels[viz.chart_type] }}</span>
            </div>
          </div>
          <div class="flex items-center gap-4">
            <span class="text-xs text-text-subtle">Updated {{ formatDate(viz.updated_at) }}</span>
            <button
              type="button"
              class="opacity-0 group-hover:opacity-100 p-1.5 rounded hover:bg-error-muted text-text-muted hover:text-error transition-all"
              @click="deleteVisualization(viz.id, $event)"
              :disabled="deleting === viz.id"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        </div>
      </LCard>
    </div>

    <!-- New visualization modal -->
    <LModal :open="showNewModal" title="New Visualization" @close="showNewModal = false">
      <div class="space-y-4">
        <p class="text-sm text-text-muted">
          Select a query to create a visualization from its results.
        </p>

        <div v-if="queries.length === 0" class="text-center py-4">
          <p class="text-text-muted mb-3">No queries found.</p>
          <LButton variant="secondary" @click="router.push({ name: 'query-new' })">
            Create a Query First
          </LButton>
        </div>

        <div v-else class="space-y-2 max-h-64 overflow-y-auto">
          <button
            v-for="q in queries"
            :key="q.id"
            type="button"
            class="w-full text-left p-3 rounded-lg border transition-colors"
            :class="
              selectedQueryId === q.id
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
                : 'border-border hover:border-border-hover'
            "
            @click="selectedQueryId = q.id"
          >
            <div class="font-medium text-text">{{ q.name }}</div>
            <div v-if="q.description" class="text-xs text-text-muted mt-0.5 truncate">
              {{ q.description }}
            </div>
          </button>
        </div>
      </div>

      <template #footer>
        <LButton variant="secondary" @click="showNewModal = false">Cancel</LButton>
        <LButton :disabled="!selectedQueryId" @click="createFromQuery">Create</LButton>
      </template>
    </LModal>
  </AppLayout>
</template>
