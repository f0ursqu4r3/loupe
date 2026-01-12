<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Trash2, LayoutGrid, Tag } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LSpinner, LBadge, LTagFilter } from '@/components/ui'
import { dashboardsApi } from '@/services/api'
import { clearLastDashboardId } from '@/utils/dashboardHistory'
import type { Dashboard } from '@/types'

const router = useRouter()

const dashboards = ref<Dashboard[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const deleting = ref<string | null>(null)

// Tag filtering
const selectedTags = ref<string[]>([])

// Get all unique tags across dashboards
const allTags = computed(() => {
  const tags = new Set<string>()
  for (const dashboard of dashboards.value) {
    for (const tag of dashboard.tags || []) {
      tags.add(tag)
    }
  }
  return Array.from(tags)
})

// Filter dashboards by selected tags
const filteredDashboards = computed(() => {
  if (selectedTags.value.length === 0) return dashboards.value
  return dashboards.value.filter((d) =>
    selectedTags.value.every((tag) => (d.tags || []).includes(tag)),
  )
})

async function loadDashboards() {
  try {
    loading.value = true
    error.value = null
    dashboards.value = await dashboardsApi.list()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load dashboards'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  clearLastDashboardId()
  loadDashboards()
})

function formatDate(dateString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(new Date(dateString))
}

function openDashboard(dashboard: Dashboard) {
  router.push({ name: 'dashboard-editor', params: { id: dashboard.id } })
}

function createDashboard() {
  router.push({ name: 'dashboard-new' })
}

async function deleteDashboard(id: string, event: Event) {
  event.stopPropagation()
  if (!confirm('Are you sure you want to delete this dashboard?')) return

  try {
    deleting.value = id
    await dashboardsApi.delete(id)
    dashboards.value = dashboards.value.filter((d) => d.id !== id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to delete dashboard'
  } finally {
    deleting.value = null
  }
}
</script>

<template>
  <AppLayout title="Dashboards">
    <template #header-actions>
      <LButton @click="createDashboard">
        <Plus class="h-4 w-4" />
        New Dashboard
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <!-- Empty state -->
    <LEmptyState
      v-else-if="dashboards.length === 0"
      title="No dashboards yet"
      description="Create your first dashboard to start visualizing your data."
    >
      <template #icon>
        <LayoutGrid class="h-8 w-8 text-text-subtle" />
      </template>
      <template #action>
        <LButton @click="createDashboard">
          <Plus class="h-4 w-4" />
          Create Dashboard
        </LButton>
      </template>
    </LEmptyState>

    <!-- Dashboard grid -->
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
        v-if="filteredDashboards.length === 0 && selectedTags.length > 0"
        class="text-center py-8 text-text-muted"
      >
        No dashboards match the selected tags
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <LCard
          v-for="dashboard in filteredDashboards"
          :key="dashboard.id"
          class="group hover:border-primary-500/50 transition-colors cursor-pointer"
          @click="openDashboard(dashboard)"
        >
          <div class="flex items-start justify-between mb-3">
            <h3 class="font-semibold text-text group-hover:text-primary-600 transition-colors">
              {{ dashboard.name }}
            </h3>
            <button
              type="button"
              class="p-1.5 rounded text-text-muted hover:text-error hover:bg-error-muted transition-colors opacity-0 group-hover:opacity-100"
              @click="deleteDashboard(dashboard.id, $event)"
              :disabled="deleting === dashboard.id"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>

          <!-- Tags display -->
          <div v-if="dashboard.tags && dashboard.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
            <LBadge v-for="tag in dashboard.tags" :key="tag" size="sm">
              <Tag class="h-3 w-3 mr-1" />
              {{ tag }}
            </LBadge>
          </div>

          <p v-if="dashboard.description" class="text-sm text-text-muted mb-4 line-clamp-2">
            {{ dashboard.description }}
          </p>

          <div class="flex items-center justify-between text-xs text-text-subtle">
            <span>{{ dashboard.tiles?.length || 0 }} tiles</span>
            <span>Updated {{ formatDate(dashboard.updated_at) }}</span>
          </div>
        </LCard>
      </div>
    </div>
  </AppLayout>
</template>
