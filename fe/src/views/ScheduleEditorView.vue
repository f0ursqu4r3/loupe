<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Save,
  Loader2,
  AlertCircle,
  CheckCircle,
  X,
  Clock,
  Trash2,
  HelpCircle,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import {
  LButton,
  LInput,
  LSelect,
  LCard,
  LSpinner,
  LCheckbox,
  LTagsInput,
  LTooltip,
} from '@/components/ui'
import { ParameterInputs } from '@/components/query'
import { schedulesApi, queriesApi } from '@/services/api'
import type { Schedule, Query, CreateScheduleRequest } from '@/types'

const route = useRoute()
const router = useRouter()

// Route params
const scheduleId = computed(() => route.params.id as string | undefined)
const queryIdFromRoute = computed(() => route.query.query_id as string | undefined)
const isNew = computed(() => !scheduleId.value || scheduleId.value === 'new')

function createEmptySchedule(): Partial<Schedule> {
  return {
    name: '',
    cron_expression: '0 * * * *',
    parameters: {},
    tags: [],
    enabled: true,
  }
}

// Schedule state
const schedule = ref<Partial<Schedule>>(createEmptySchedule())

// Query state
const query = ref<Query | null>(null)
const allQueries = ref<Query[]>([])

// UI state
const loading = ref(false)
const saving = ref(false)
const deleting = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)
const showDeleteConfirm = ref(false)

function resetScheduleState() {
  schedule.value = createEmptySchedule()
  query.value = null
  selectedPreset.value = schedule.value.cron_expression || '0 * * * *'
  error.value = null
  saveSuccess.value = false
  showDeleteConfirm.value = false
  deleting.value = false
  loading.value = false
  saving.value = false
}

// Cron presets
const cronPresets = [
  { value: '* * * * *', label: 'Every minute' },
  { value: '*/5 * * * *', label: 'Every 5 minutes' },
  { value: '*/15 * * * *', label: 'Every 15 minutes' },
  { value: '*/30 * * * *', label: 'Every 30 minutes' },
  { value: '0 * * * *', label: 'Every hour' },
  { value: '0 */6 * * *', label: 'Every 6 hours' },
  { value: '0 */12 * * *', label: 'Every 12 hours' },
  { value: '0 0 * * *', label: 'Daily at midnight' },
  { value: '0 9 * * *', label: 'Daily at 9am' },
  { value: '0 0 * * 1', label: 'Weekly on Monday' },
  { value: '0 0 1 * *', label: 'Monthly on the 1st' },
  { value: 'custom', label: 'Custom...' },
]

const selectedPreset = ref('0 * * * *')
const isCustomCron = computed(() => {
  const found = cronPresets.find((p) => p.value === schedule.value.cron_expression)
  return !found || found.value === 'custom'
})

// Query options for selector
const queryOptions = computed(() =>
  allQueries.value.map((q) => ({
    value: q.id,
    label: q.name,
  })),
)

// Watch preset changes
watch(selectedPreset, (value) => {
  if (value !== 'custom') {
    schedule.value.cron_expression = value
  }
})

// Load all queries for the selector
async function loadQueries() {
  try {
    allQueries.value = await queriesApi.list()
  } catch (e) {
    console.error('Failed to load queries:', e)
  }
}

// Handle query change
async function handleQueryChange(queryId: string) {
  if (!queryId || queryId === schedule.value.query_id) return

  schedule.value.query_id = queryId

  try {
    query.value = await queriesApi.get(queryId)
    // Initialize parameter values with defaults
    const params: Record<string, unknown> = {}
    for (const p of query.value.parameters || []) {
      params[p.name] = p.default ?? ''
    }
    schedule.value.parameters = params

    // Auto-name for new schedules
    if (isNew.value && !schedule.value.name) {
      schedule.value.name = `${query.value.name} Schedule`
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load query'
  }
}

// Load schedule
async function loadSchedule() {
  const activeScheduleId = scheduleId.value
  const activeQueryId = queryIdFromRoute.value

  // Load all queries for the selector
  await loadQueries()

  if (activeScheduleId !== scheduleId.value || activeQueryId !== queryIdFromRoute.value) {
    return
  }

  if (isNew.value) {
    // For new schedules, load the query from query_id param
    if (activeQueryId) {
      await handleQueryChange(activeQueryId)
    }
    return
  }

  try {
    loading.value = true
    const sched = await schedulesApi.get(activeScheduleId!)
    if (activeScheduleId !== scheduleId.value) return
    schedule.value = sched

    // Set the preset selector
    const foundPreset = cronPresets.find((p) => p.value === sched.cron_expression)
    selectedPreset.value = foundPreset ? foundPreset.value : 'custom'

    // Load associated query
    query.value = await queriesApi.get(sched.query_id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load schedule'
  } finally {
    if (activeScheduleId === scheduleId.value) {
      loading.value = false
    }
  }
}

// Save schedule
async function saveSchedule() {
  if (!schedule.value.name?.trim()) {
    error.value = 'Schedule name is required'
    return
  }
  if (!schedule.value.query_id) {
    error.value = 'Please select a query'
    return
  }
  if (!schedule.value.cron_expression?.trim()) {
    error.value = 'Cron expression is required'
    return
  }

  try {
    saving.value = true
    error.value = null

    const payload: CreateScheduleRequest = {
      query_id: schedule.value.query_id!,
      name: schedule.value.name!,
      cron_expression: schedule.value.cron_expression!,
      parameters: schedule.value.parameters,
      tags: schedule.value.tags,
      enabled: schedule.value.enabled ?? true,
    }

    if (isNew.value) {
      const created = await schedulesApi.create(payload)
      router.replace({ name: 'schedule-editor', params: { id: created.id } })
      schedule.value = created
    } else {
      const updated = await schedulesApi.update(scheduleId.value!, payload)
      schedule.value = updated
    }

    saveSuccess.value = true
    setTimeout(() => (saveSuccess.value = false), 2000)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save schedule'
  } finally {
    saving.value = false
  }
}

// Delete schedule
async function deleteSchedule() {
  if (!scheduleId.value) return

  try {
    deleting.value = true
    error.value = null
    await schedulesApi.delete(scheduleId.value)
    router.push({ name: 'schedules' })
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to delete schedule'
    showDeleteConfirm.value = false
  } finally {
    deleting.value = false
  }
}

// Get human-readable cron description
function describeCron(expr: string): string {
  const preset = cronPresets.find((p) => p.value === expr)
  if (preset && preset.value !== 'custom') return preset.label

  // Parse basic cron
  const parts = expr.split(' ')
  if (parts.length !== 5) return 'Invalid expression'

  const [min, hour, dayOfMonth, month, dayOfWeek] = parts

  // Very basic descriptions
  if (min === '*' && hour === '*') return 'Every minute'
  if (min?.startsWith('*/')) return `Every ${min.slice(2)} minutes`
  if (hour?.startsWith('*/') && min === '0') return `Every ${hour.slice(2)} hours`
  if (hour === '*' && min === '0') return 'Every hour'
  if (dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    if (hour !== '*' && hour !== undefined && min !== '*' && min !== undefined) {
      return `Daily at ${hour.padStart(2, '0')}:${min.padStart(2, '0')}`
    }
  }

  return expr
}

onMounted(loadSchedule)

watch(
  [() => scheduleId.value, () => queryIdFromRoute.value],
  ([nextId, nextQuery], [prevId, prevQuery]) => {
    if (nextId === prevId && nextQuery === prevQuery) return
    resetScheduleState()
    loadSchedule()
  },
)
</script>

<template>
  <AppLayout :title="isNew ? 'New Schedule' : 'Edit Schedule'" back="schedules">
    <template #header-actions>
      <div class="flex items-center gap-2">
        <div v-if="!isNew" class="flex items-center gap-2">
          <LButton variant="danger" size="sm" @click="showDeleteConfirm = true">
            <Trash2 class="h-4 w-4" />
          </LButton>
        </div>

        <LButton @click="saveSchedule" :disabled="saving">
          <Loader2 v-if="saving" class="h-4 w-4 animate-spin" />
          <Save v-else class="h-4 w-4" />
          {{ isNew ? 'Create' : 'Save' }}
        </LButton>
      </div>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <div v-else class="max-w-2xl mx-auto space-y-6">
      <!-- Error alert -->
      <div
        v-if="error"
        class="flex items-center gap-2 p-3 rounded-lg bg-error-muted text-error text-sm"
      >
        <AlertCircle class="h-4 w-4 shrink-0" />
        {{ error }}
        <button @click="error = null" class="ml-auto hover:opacity-80">
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Success message -->
      <div
        v-if="saveSuccess"
        class="flex items-center gap-2 p-3 rounded-lg bg-success-muted text-success text-sm"
      >
        <CheckCircle class="h-4 w-4 shrink-0" />
        Schedule saved successfully
      </div>

      <!-- Basic Info -->
      <LCard title="Basic Information">
        <div class="space-y-4">
          <!-- Name -->
          <div>
            <label class="block text-sm font-medium text-text mb-1">Name</label>
            <LInput v-model="schedule.name" placeholder="e.g., Daily Sales Report" />
          </div>

          <!-- Query Selection -->
          <div>
            <label class="block text-sm font-medium text-text mb-1">Query</label>
            <LSelect
              :model-value="schedule.query_id"
              @update:model-value="handleQueryChange($event as string)"
              :options="queryOptions"
              placeholder="Select a query..."
            />
            <p class="mt-1 text-xs text-text-muted">
              Select the query that will run on this schedule
            </p>
          </div>

          <!-- Tags -->
          <div>
            <label class="block text-sm font-medium text-text mb-1">Tags</label>
            <LTagsInput
              :model-value="schedule.tags || []"
              @update:model-value="schedule.tags = $event"
              placeholder="Add tags..."
            />
          </div>

          <!-- Enabled -->
          <div class="flex items-center gap-2">
            <LCheckbox
              :model-value="schedule.enabled ?? true"
              @update:model-value="schedule.enabled = $event"
            />
            <label class="text-sm text-text">Enable schedule</label>
          </div>
        </div>
      </LCard>

      <!-- Schedule Configuration -->
      <LCard title="Schedule">
        <div class="space-y-4">
          <!-- Preset Selector -->
          <div>
            <label class="block text-sm font-medium text-text mb-1">Frequency</label>
            <LSelect v-model="selectedPreset" :options="cronPresets" />
          </div>

          <!-- Custom Cron Expression -->
          <div v-if="isCustomCron || selectedPreset === 'custom'">
            <label class="block text-sm font-medium text-text mb-1">
              Cron Expression
              <LTooltip>
                <template #trigger>
                  <HelpCircle class="inline h-3.5 w-3.5 text-text-muted ml-1" />
                </template>
                <div class="text-xs">
                  <p class="font-medium mb-1">Format: minute hour day month weekday</p>
                  <p>* = any value</p>
                  <p>*/n = every n units</p>
                  <p>0-23 = specific hour</p>
                  <p>1-7 = Sun-Sat</p>
                </div>
              </LTooltip>
            </label>
            <LInput v-model="schedule.cron_expression" placeholder="0 * * * *" class="font-mono" />
          </div>

          <!-- Preview -->
          <div class="flex items-center gap-2 p-3 bg-surface-sunken rounded-lg">
            <Clock class="h-4 w-4 text-text-muted" />
            <span class="text-sm text-text">
              Runs: <strong>{{ describeCron(schedule.cron_expression || '') }}</strong>
            </span>
          </div>
        </div>
      </LCard>

      <!-- Parameters (if query has any) -->
      <LCard v-if="query && query.parameters && query.parameters.length > 0" title="Parameters">
        <p class="text-sm text-text-muted mb-4">
          Configure the parameter values that will be used when this schedule runs.
        </p>
        <ParameterInputs
          :parameters="query.parameters"
          :model-value="schedule.parameters || {}"
          @update:model-value="schedule.parameters = $event"
        />
      </LCard>

      <!-- Delete Confirmation -->
      <div v-if="showDeleteConfirm" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/50" @click="showDeleteConfirm = false" />
        <div
          class="relative bg-surface-overlay rounded-xl shadow-xl border border-border p-6 max-w-sm w-full"
        >
          <h3 class="text-lg font-semibold text-text mb-2">Delete Schedule</h3>
          <p class="text-text-muted text-sm mb-4">
            Are you sure you want to delete this schedule? This action cannot be undone.
          </p>
          <div class="flex justify-end gap-2">
            <LButton variant="ghost" @click="showDeleteConfirm = false">Cancel</LButton>
            <LButton variant="danger" @click="deleteSchedule" :disabled="deleting">
              <Loader2 v-if="deleting" class="h-4 w-4 animate-spin" />
              Delete
            </LButton>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>
