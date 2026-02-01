<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Trash2, LayoutGrid, Tag } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LSpinner, LBadge, LTagFilter, LModal } from '@/components/ui'
import { dashboardsApi } from '@/services/api'
import { clearLastDashboardId } from '@/utils/dashboardHistory'
import { formatDateShort } from '@/utils/dateTime'
import { usePermissions } from '@/composables/usePermissions'
import { useApiError } from '@/composables/useApiError'
import type { Dashboard } from '@/types'

const router = useRouter()
const { canEdit } = usePermissions()
const { handleError } = useApiError()

const dashboards = ref<Dashboard[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const deleting = ref<string | null>(null)

// Delete confirmation modal
const showDeleteModal = ref(false)
const dashboardToDelete = ref<Dashboard | null>(null)

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
    handleError(e, 'Failed to load dashboards')
    error.value = e instanceof Error ? e.message : 'Failed to load dashboards'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  clearLastDashboardId()
  loadDashboards()
})

function openDashboard(dashboard: Dashboard) {
  router.push({ name: 'dashboard-editor', params: { id: dashboard.id } })
}

function createDashboard() {
  router.push({ name: 'dashboard-new' })
}

function deleteDashboard(id: string, event: Event) {
  event.stopPropagation()
  const dashboard = dashboards.value.find((d) => d.id === id)
  if (dashboard) {
    dashboardToDelete.value = dashboard
    showDeleteModal.value = true
  }
}

async function confirmDelete() {
  if (!dashboardToDelete.value) return

  try {
    deleting.value = dashboardToDelete.value.id
    await dashboardsApi.delete(dashboardToDelete.value.id)
    dashboards.value = dashboards.value.filter((d) => d.id !== dashboardToDelete.value!.id)
    showDeleteModal.value = false
    dashboardToDelete.value = null
  } catch (e) {
    handleError(e, 'Failed to delete dashboard')
    error.value = e instanceof Error ? e.message : 'Failed to delete dashboard'
  } finally {
    deleting.value = null
  }
}
</script>

<template>
  <AppLayout title="Dashboards">
    <template #header-actions>
      <LButton v-if="canEdit" @click="createDashboard">
        <Plus :size="16" />
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
      :description="canEdit ? 'Create your first dashboard to start visualizing your data.' : 'No dashboards have been created yet. Contact an editor or admin to create dashboards.'"
    >
      <template #icon>
        <LayoutGrid :size="48" class="text-text-subtle" />
      </template>
      <template #action>
        <LButton v-if="canEdit" @click="createDashboard">
          <Plus :size="16" />
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
          class="group hover:border-primary-500/50 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-200 cursor-pointer"
          @click="openDashboard(dashboard)"
        >
          <div class="flex items-start justify-between mb-3">
            <h3 class="font-semibold text-text group-hover:text-primary-600 transition-colors">
              {{ dashboard.name }}
            </h3>
            <button
              v-if="canEdit"
              type="button"
              class="p-1.5 rounded text-text-muted hover:text-error hover:bg-error-muted transition-colors opacity-0 group-hover:opacity-100"
              @click="deleteDashboard(dashboard.id, $event)"
              :disabled="deleting === dashboard.id"
            >
              <Trash2 :size="16" />
            </button>
          </div>

          <!-- Tags display -->
          <div v-if="dashboard.tags && dashboard.tags.length > 0" class="flex flex-wrap gap-1 mb-2">
            <LBadge v-for="tag in dashboard.tags" :key="tag" size="sm">
              <Tag :size="12" class="mr-1" />
              {{ tag }}
            </LBadge>
          </div>

          <p v-if="dashboard.description" class="text-sm text-text-muted mb-4 line-clamp-2">
            {{ dashboard.description }}
          </p>

          <div class="flex items-center justify-between text-xs text-text-subtle">
            <span>{{ dashboard.tiles?.length || 0 }} tiles</span>
            <span>Updated {{ formatDateShort(dashboard.updated_at) }}</span>
          </div>
        </LCard>
      </div>
    </div>

    <!-- Delete confirmation modal -->
    <LModal v-model="showDeleteModal" title="Delete Dashboard" size="sm">
      <p class="text-text">
        Are you sure you want to delete
        <strong>{{ dashboardToDelete?.name }}</strong
        >?
      </p>
      <p class="text-sm text-text-muted mt-2">This action cannot be undone.</p>

      <template #footer>
        <LButton variant="secondary" @click="showDeleteModal = false">Cancel</LButton>
        <LButton variant="danger" :loading="deleting !== null" @click="confirmDelete">
          Delete Dashboard
        </LButton>
      </template>
    </LModal>
  </AppLayout>
</template>
