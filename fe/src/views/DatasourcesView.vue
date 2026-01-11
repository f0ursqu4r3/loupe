<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, TestTube, CheckCircle, XCircle, Loader2 } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner, LModal, LInput, LSelect } from '@/components/ui'
import { datasourcesApi } from '@/services/api'
import type { Datasource, ConnectionTestResult } from '@/types'

const datasources = ref<Datasource[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

// Modal state
const showCreateModal = ref(false)
const createForm = ref({
  name: '',
  ds_type: 'postgres' as const,
  connection_string: '',
})
const creating = ref(false)

// Test connection state
const testingId = ref<string | null>(null)
const testResults = ref<Record<string, ConnectionTestResult>>({})

async function loadDatasources() {
  try {
    loading.value = true
    error.value = null
    datasources.value = await datasourcesApi.list()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load datasources'
  } finally {
    loading.value = false
  }
}

async function createDatasource() {
  try {
    creating.value = true
    await datasourcesApi.create(createForm.value)
    showCreateModal.value = false
    createForm.value = { name: '', ds_type: 'postgres', connection_string: '' }
    await loadDatasources()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to create datasource'
  } finally {
    creating.value = false
  }
}

async function testConnection(id: string) {
  try {
    testingId.value = id
    const result = await datasourcesApi.test(id)
    testResults.value[id] = result
  } catch (e) {
    testResults.value[id] = {
      success: false,
      message: e instanceof Error ? e.message : 'Connection test failed',
    }
  } finally {
    testingId.value = null
  }
}

onMounted(loadDatasources)

function formatDate(dateString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(new Date(dateString))
}

const dsTypeOptions = [
  { value: 'postgres', label: 'PostgreSQL' },
]
</script>

<template>
  <AppLayout title="Datasources">
    <template #header-actions>
      <LButton @click="showCreateModal = true">
        <Plus class="h-4 w-4" />
        New Datasource
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <!-- Empty state -->
    <LEmptyState
      v-else-if="datasources.length === 0"
      title="No datasources configured"
      description="Connect your first database to start running queries."
    >
      <template #action>
        <LButton @click="showCreateModal = true">
          <Plus class="h-4 w-4" />
          Add Datasource
        </LButton>
      </template>
    </LEmptyState>

    <!-- Datasources list -->
    <div v-else class="space-y-4">
      <LCard v-for="ds in datasources" :key="ds.id" class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
            <span class="text-primary-600 font-semibold text-sm">PG</span>
          </div>
          <div>
            <h3 class="font-medium text-text">{{ ds.name }}</h3>
            <p class="text-sm text-text-muted">
              {{ ds.ds_type }} · Created {{ formatDate(ds.created_at) }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <!-- Test result -->
          <div v-if="testResults[ds.id]" class="flex items-center gap-2">
            <CheckCircle v-if="testResults[ds.id]?.success" class="h-4 w-4 text-success" />
            <XCircle v-else class="h-4 w-4 text-error" />
            <span class="text-sm" :class="testResults[ds.id]?.success ? 'text-success' : 'text-error'">
              {{ testResults[ds.id]?.success ? `${testResults[ds.id]?.latency_ms}ms` : 'Failed' }}
            </span>
          </div>

          <LButton
            variant="secondary"
            size="sm"
            :loading="testingId === ds.id"
            @click="testConnection(ds.id)"
          >
            <TestTube class="h-4 w-4" />
            Test
          </LButton>
        </div>
      </LCard>
    </div>

    <!-- Create Modal -->
    <LModal v-model="showCreateModal" title="New Datasource">
      <form class="space-y-4" @submit.prevent="createDatasource">
        <div>
          <label class="block text-sm font-medium text-text mb-1.5">Name</label>
          <LInput v-model="createForm.name" placeholder="Production Database" />
        </div>

        <div>
          <label class="block text-sm font-medium text-text mb-1.5">Type</label>
          <LSelect v-model="createForm.ds_type" :options="dsTypeOptions" />
        </div>

        <div>
          <label class="block text-sm font-medium text-text mb-1.5">Connection String</label>
          <LInput
            v-model="createForm.connection_string"
            type="password"
            placeholder="postgres://user:pass@host:5432/db"
          />
        </div>
      </form>

      <template #footer>
        <div class="flex justify-end gap-3">
          <LButton variant="secondary" @click="showCreateModal = false">Cancel</LButton>
          <LButton :loading="creating" @click="createDatasource">Create</LButton>
        </div>
      </template>
    </LModal>
  </AppLayout>
</template>
