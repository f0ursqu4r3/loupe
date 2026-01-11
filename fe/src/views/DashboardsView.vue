<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, MoreVertical, ExternalLink } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LSpinner } from '@/components/ui'
import { dashboardsApi } from '@/services/api'
import type { Dashboard } from '@/types'

const dashboards = ref<Dashboard[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

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

onMounted(loadDashboards)

function formatDate(dateString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(new Date(dateString))
}
</script>

<template>
  <AppLayout title="Dashboards">
    <template #header-actions>
      <LButton>
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
      <template #action>
        <LButton>
          <Plus class="h-4 w-4" />
          Create Dashboard
        </LButton>
      </template>
    </LEmptyState>

    <!-- Dashboard grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <LCard
        v-for="dashboard in dashboards"
        :key="dashboard.id"
        class="group hover:border-primary-500/50 transition-colors cursor-pointer"
      >
        <div class="flex items-start justify-between mb-3">
          <h3 class="font-semibold text-text group-hover:text-primary-600 transition-colors">
            {{ dashboard.name }}
          </h3>
          <button
            type="button"
            class="p-1 rounded text-text-muted hover:text-text hover:bg-surface-sunken transition-colors opacity-0 group-hover:opacity-100"
          >
            <MoreVertical class="h-4 w-4" />
          </button>
        </div>

        <p v-if="dashboard.description" class="text-sm text-text-muted mb-4 line-clamp-2">
          {{ dashboard.description }}
        </p>

        <div class="flex items-center justify-between text-xs text-text-subtle">
          <span>Updated {{ formatDate(dashboard.updated_at) }}</span>
          <ExternalLink class="h-3.5 w-3.5 opacity-0 group-hover:opacity-100 transition-opacity" />
        </div>
      </LCard>
    </div>
  </AppLayout>
</template>
