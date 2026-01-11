<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, Play, FileCode } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner } from '@/components/ui'
import { queriesApi } from '@/services/api'
import type { Query } from '@/types'

const queries = ref<Query[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadQueries() {
  try {
    loading.value = true
    error.value = null
    queries.value = await queriesApi.list()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load queries'
  } finally {
    loading.value = false
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
</script>

<template>
  <AppLayout title="Queries">
    <template #header-actions>
      <LButton>
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
        <LButton>
          <Plus class="h-4 w-4" />
          Create Query
        </LButton>
      </template>
    </LEmptyState>

    <!-- Queries list -->
    <div v-else class="space-y-4">
      <LCard
        v-for="query in queries"
        :key="query.id"
        class="group hover:border-primary-500/50 transition-colors"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <h3 class="font-medium text-text truncate">{{ query.name }}</h3>
              <LBadge v-if="query.parameters.length > 0" size="sm">
                {{ query.parameters.length }} params
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

          <div class="flex items-center gap-2 ml-4">
            <LButton variant="ghost" size="sm">
              <Play class="h-4 w-4" />
            </LButton>
          </div>
        </div>

        <div class="flex items-center justify-between mt-4 pt-3 border-t border-border">
          <span class="text-xs text-text-subtle"> Updated {{ formatDate(query.updated_at) }} </span>
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
