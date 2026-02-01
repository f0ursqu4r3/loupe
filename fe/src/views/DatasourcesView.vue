<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, TestTube, CheckCircle, XCircle, Pencil, Trash2 } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LSpinner, LModal, LInput, LSelect } from '@/components/ui'
import { datasourcesApi } from '@/services/api'
import { formatDateShort } from '@/utils/dateTime'
import { useToast } from '@/composables/useToast'
import type { Datasource, ConnectionTestResult } from '@/types'

const toast = useToast()
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

// Edit modal state
const showEditModal = ref(false)
const editingDatasource = ref<Datasource | null>(null)
const editForm = ref({
  name: '',
  connection_string: '',
})
const updating = ref(false)

// Delete confirmation state
const showDeleteModal = ref(false)
const deletingDatasource = ref<Datasource | null>(null)
const deleting = ref(false)

async function loadDatasources() {
  try {
    loading.value = true
    error.value = null
    datasources.value = await datasourcesApi.list()
  } catch (e) {
    error.value =
      e instanceof Error
        ? e.message
        : 'Unable to load datasources. Please check your connection and try again.'
  } finally {
    loading.value = false
  }
}

async function createDatasource() {
  try {
    creating.value = true
    const datasource = await datasourcesApi.create(createForm.value)
    showCreateModal.value = false
    toast.success(`Datasource "${createForm.value.name}" created successfully`)
    createForm.value = { name: '', ds_type: 'postgres', connection_string: '' }
    await loadDatasources()
  } catch (e) {
    error.value =
      e instanceof Error
        ? e.message
        : 'Unable to create datasource. Please verify your connection string and try again.'
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

function openEditModal(ds: Datasource) {
  editingDatasource.value = ds
  editForm.value = {
    name: ds.name,
    connection_string: '',
  }
  showEditModal.value = true
}

async function updateDatasource() {
  if (!editingDatasource.value) return
  const datasourceName = editingDatasource.value.name
  try {
    updating.value = true
    const updateData: { name?: string; connection_string?: string } = {}
    if (editForm.value.name !== editingDatasource.value.name) {
      updateData.name = editForm.value.name
    }
    if (editForm.value.connection_string) {
      updateData.connection_string = editForm.value.connection_string
    }
    await datasourcesApi.update(editingDatasource.value.id, updateData)
    toast.success(`Datasource "${datasourceName}" updated successfully`)
    showEditModal.value = false
    editingDatasource.value = null
    await loadDatasources()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to update datasource'
  } finally {
    updating.value = false
  }
}

function openDeleteModal(ds: Datasource) {
  deletingDatasource.value = ds
  showDeleteModal.value = true
}

async function deleteDatasource() {
  if (!deletingDatasource.value) return
  const datasourceName = deletingDatasource.value.name
  try {
    deleting.value = true
    await datasourcesApi.delete(deletingDatasource.value.id)
    toast.success(`Datasource "${datasourceName}" deleted successfully`)
    showDeleteModal.value = false
    deletingDatasource.value = null
    await loadDatasources()
  } catch (e) {
    error.value =
      e instanceof Error
        ? e.message
        : 'Unable to delete datasource. This datasource may be in use by queries or visualizations.'
  } finally {
    deleting.value = false
  }
}

onMounted(loadDatasources)

const dsTypeOptions = [{ value: 'postgres', label: 'PostgreSQL' }]
</script>

<template>
  <AppLayout title="Datasources">
    <template #header-actions>
      <LButton @click="showCreateModal = true">
        <Plus :size="16" />
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
          <Plus :size="16" />
          Add Datasource
        </LButton>
      </template>
    </LEmptyState>

    <!-- Datasources list -->
    <div v-else class="space-y-4">
      <LCard v-for="ds in datasources" :key="ds.id" class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center"
          >
            <span class="text-primary-600 font-semibold text-sm">PG</span>
          </div>
          <div>
            <h3 class="font-medium text-text">{{ ds.name }}</h3>
            <p class="text-sm text-text-muted">
              {{ ds.ds_type }} · Created {{ formatDateShort(ds.created_at) }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <!-- Test result -->
          <div v-if="testResults[ds.id]" class="flex items-center gap-2">
            <CheckCircle v-if="testResults[ds.id]?.success" :size="16" class="text-success" />
            <XCircle v-else :size="16" class="text-error" />
            <span
              class="text-sm"
              :class="testResults[ds.id]?.success ? 'text-success' : 'text-error'"
            >
              {{ testResults[ds.id]?.success ? `${testResults[ds.id]?.latency_ms}ms` : 'Failed' }}
            </span>
          </div>

          <LButton
            variant="secondary"
            size="sm"
            :loading="testingId === ds.id"
            @click="testConnection(ds.id)"
          >
            <TestTube :size="16" />
            Test
          </LButton>

          <LButton variant="secondary" size="sm" @click="openEditModal(ds)">
            <Pencil :size="16" />
            Edit
          </LButton>

          <LButton variant="secondary" size="sm" @click="openDeleteModal(ds)">
            <Trash2 :size="16" />
            Delete
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

    <!-- Edit Modal -->
    <LModal v-model="showEditModal" title="Edit Datasource">
      <form class="space-y-4" @submit.prevent="updateDatasource">
        <div>
          <label class="block text-sm font-medium text-text mb-1.5">Name</label>
          <LInput v-model="editForm.name" placeholder="Production Database" />
        </div>

        <div>
          <label class="block text-sm font-medium text-text mb-1.5">Connection String</label>
          <LInput
            v-model="editForm.connection_string"
            type="password"
            placeholder="Leave empty to keep current"
          />
          <p class="mt-1 text-xs text-text-muted">
            Only fill this in if you want to change the connection string.
          </p>
        </div>
      </form>

      <template #footer>
        <div class="flex justify-end gap-3">
          <LButton variant="secondary" @click="showEditModal = false">Cancel</LButton>
          <LButton :loading="updating" @click="updateDatasource">Save Changes</LButton>
        </div>
      </template>
    </LModal>

    <!-- Delete Confirmation Modal -->
    <LModal v-model="showDeleteModal" title="Delete Datasource">
      <p class="text-text">
        Are you sure you want to delete <strong>{{ deletingDatasource?.name }}</strong>? This action
        cannot be undone.
      </p>

      <template #footer>
        <div class="flex justify-end gap-3">
          <LButton variant="secondary" @click="showDeleteModal = false">Cancel</LButton>
          <LButton variant="danger" :loading="deleting" @click="deleteDatasource">Delete</LButton>
        </div>
      </template>
    </LModal>
  </AppLayout>
</template>
