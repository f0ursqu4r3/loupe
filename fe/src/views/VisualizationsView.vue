<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
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
  Tag,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LSpinner, LModal, LBadge, LTagFilter } from '@/components/ui'
import { visualizationsApi, queriesApi } from '@/services/api'
import { formatDateShort } from '@/utils/dateTime'
import type { Visualization, Query, ChartType } from '@/types'

const router = useRouter()

const visualizations = ref<Visualization[]>([])
const queries = ref<Query[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

// View mode: 'grid' or 'list'
const viewMode = ref<'grid' | 'list'>('grid')

// Tag filtering
const selectedTags = ref<string[]>([])

// Get all unique tags across visualizations
const allTags = computed(() => {
  const tags = new Set<string>()
  for (const viz of visualizations.value) {
    for (const tag of viz.tags || []) {
      tags.add(tag)
    }
  }
  return Array.from(tags)
})

// Filter visualizations by selected tags
const filteredVisualizations = computed(() => {
  if (selectedTags.value.length === 0) return visualizations.value
  return visualizations.value.filter((v) =>
    selectedTags.value.every((tag) => (v.tags || []).includes(tag)),
  )
})

// Modal state
const showNewModal = ref(false)
const selectedQueryId = ref<string | null>(null)
const deleting = ref<string | null>(null)

// Delete confirmation modal
const showDeleteModal = ref(false)
const visualizationToDelete = ref<Visualization | null>(null)

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

function deleteVisualization(id: string, event: Event) {
  event.stopPropagation()
  const viz = visualizations.value.find((v) => v.id === id)
  if (viz) {
    visualizationToDelete.value = viz
    showDeleteModal.value = true
  }
}

async function confirmDelete() {
  if (!visualizationToDelete.value) return

  try {
    deleting.value = visualizationToDelete.value.id
    await visualizationsApi.delete(visualizationToDelete.value.id)
    visualizations.value = visualizations.value.filter((v) => v.id !== visualizationToDelete.value!.id)
    showDeleteModal.value = false
    visualizationToDelete.value = null
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
          <LayoutGrid :size="16" />
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
          <List :size="16" />
        </button>
      </div>
      <LButton @click="showNewModal = true">
        <Plus :size="16" />
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
        <BarChart3 :size="48" class="text-text-subtle" />
      </template>
      <template #action>
        <LButton @click="showNewModal = true">
          <Plus :size="16" />
          Create Visualization
        </LButton>
      </template>
    </LEmptyState>

    <!-- Visualizations grid -->
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
        v-if="filteredVisualizations.length === 0 && selectedTags.length > 0"
        class="text-center py-8 text-text-muted"
      >
        No visualizations match the selected tags
      </div>

      <div
        :class="
          viewMode === 'grid' ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6' : 'space-y-3'
        "
      >
        <!-- Grid view card -->
        <LCard
          v-if="viewMode === 'grid'"
          v-for="viz in filteredVisualizations"
          :key="viz.id"
          class="group hover:border-primary-500/50 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-200 cursor-pointer"
          @click="openVisualization(viz)"
        >
          <div class="flex items-start justify-between mb-3">
            <div class="flex items-center gap-3">
              <div
                class="w-10 h-10 shrink-0 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center"
              >
                <component :is="chartTypeIcons[viz.chart_type]" :size="20" class="text-primary-600" />
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
              <Trash2 :size="16" />
            </button>
          </div>

          <!-- Tags display -->
          <div v-if="viz.tags && viz.tags.length > 0" class="flex flex-wrap gap-1 mb-3">
            <LBadge v-for="tag in viz.tags" :key="tag" size="sm">
              <Tag :size="12" class="mr-1" />
              {{ tag }}
            </LBadge>
          </div>

          <!-- Preview placeholder -->
          <div class="h-32 bg-surface-sunken rounded-lg flex items-center justify-center mb-3">
            <component :is="chartTypeIcons[viz.chart_type]" :size="48" class="text-text-subtle/30" />
          </div>

          <div class="text-xs text-text-subtle">Updated {{ formatDateShort(viz.updated_at) }}</div>
        </LCard>

        <!-- List view row -->
        <LCard
          v-if="viewMode === 'list'"
          v-for="viz in filteredVisualizations"
          :key="viz.id"
          padding="sm"
          class="group hover:border-primary-500/50 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-200 cursor-pointer"
          @click="openVisualization(viz)"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div
                class="w-10 h-10 shrink-0 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center"
              >
                <component :is="chartTypeIcons[viz.chart_type]" :size="20" class="text-primary-600" />
              </div>
              <div class="min-w-0">
                <h3 class="font-medium text-text group-hover:text-primary-600 transition-colors">
                  {{ viz.name }}
                </h3>
                <div class="flex items-center gap-2">
                  <span class="text-xs text-text-muted">{{ chartTypeLabels[viz.chart_type] }}</span>
                  <!-- Tags in list view -->
                  <div v-if="viz.tags && viz.tags.length > 0" class="flex flex-wrap gap-1">
                    <LBadge v-for="tag in viz.tags" :key="tag" size="sm">
                      {{ tag }}
                    </LBadge>
                  </div>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-4">
              <span class="text-xs text-text-subtle">
                Updated {{ formatDateShort(viz.updated_at) }}
              </span>
              <button
                type="button"
                class="opacity-0 group-hover:opacity-100 p-1.5 rounded hover:bg-error-muted text-text-muted hover:text-error transition-all"
                @click="deleteVisualization(viz.id, $event)"
                :disabled="deleting === viz.id"
              >
                <Trash2 :size="16" />
              </button>
            </div>
          </div>
        </LCard>
      </div>
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

    <!-- Delete confirmation modal -->
    <LModal v-model="showDeleteModal" title="Delete Visualization" size="sm">
      <p class="text-text">
        Are you sure you want to delete
        <strong>{{ visualizationToDelete?.name }}</strong
        >?
      </p>
      <p class="text-sm text-text-muted mt-2">This action cannot be undone.</p>

      <template #footer>
        <LButton variant="secondary" @click="showDeleteModal = false">Cancel</LButton>
        <LButton variant="danger" :loading="deleting !== null" @click="confirmDelete">
          Delete Visualization
        </LButton>
      </template>
    </LModal>
  </AppLayout>
</template>
