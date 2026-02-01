<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  Plus,
  Play,
  Pause,
  Clock,
  Calendar as CalendarIcon,
  Tag,
  Pencil,
  Trash2,
  Loader2,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner, LTagFilter } from '@/components/ui'
import { schedulesApi } from '@/services/api'
import { formatDateTimeShort } from '@/utils/dateTime'
import { usePermissions } from '@/composables/usePermissions'
import { useApiError } from '@/composables/useApiError'
import type { Schedule } from '@/types'

const router = useRouter()
const { canEdit } = usePermissions()
const { handleError } = useApiError()

const schedules = ref<Schedule[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

// Tag filtering
const selectedTags = ref<string[]>([])

// Delete confirmation state
const showDeleteConfirm = ref(false)
const scheduleToDelete = ref<Schedule | null>(null)
const deleting = ref(false)

// Get all unique tags across schedules
const allTags = computed(() => {
  const tags = new Set<string>()
  for (const schedule of schedules.value) {
    for (const tag of schedule.tags || []) {
      tags.add(tag)
    }
  }
  return Array.from(tags)
})

// Filter schedules by selected tags
const filteredSchedules = computed(() => {
  if (selectedTags.value.length === 0) return schedules.value
  return schedules.value.filter((s) =>
    selectedTags.value.every((tag) => (s.tags || []).includes(tag)),
  )
})

async function loadSchedules() {
  try {
    loading.value = true
    error.value = null
    schedules.value = await schedulesApi.list()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load schedules'
  } finally {
    loading.value = false
  }
}

async function toggleSchedule(schedule: Schedule) {
  try {
    if (schedule.enabled) {
      await schedulesApi.disable(schedule.id)
    } else {
      await schedulesApi.enable(schedule.id)
    }
    await loadSchedules()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to update schedule'
  }
}

async function triggerSchedule(id: string) {
  try {
    await schedulesApi.trigger(id)
    await loadSchedules()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to trigger schedule'
  }
}

function confirmDelete(schedule: Schedule) {
  scheduleToDelete.value = schedule
  showDeleteConfirm.value = true
}

async function deleteSchedule() {
  if (!scheduleToDelete.value) return
  try {
    deleting.value = true
    await schedulesApi.delete(scheduleToDelete.value.id)
    await loadSchedules()
    showDeleteConfirm.value = false
    scheduleToDelete.value = null
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to delete schedule'
  } finally {
    deleting.value = false
  }
}

onMounted(loadSchedules)

// Human-readable cron description
function formatDate(dateString: string | undefined): string {
  if (!dateString) return 'Never'
  return formatDateTimeShort(dateString)
}

function describeCron(expr: string): string {
  // Common presets
  const presets: Record<string, string> = {
    '* * * * *': 'Every minute',
    '*/5 * * * *': 'Every 5 minutes',
    '*/15 * * * *': 'Every 15 minutes',
    '*/30 * * * *': 'Every 30 minutes',
    '0 * * * *': 'Every hour',
    '0 */6 * * *': 'Every 6 hours',
    '0 */12 * * *': 'Every 12 hours',
    '0 0 * * *': 'Daily at midnight',
    '0 9 * * *': 'Daily at 9am',
    '0 0 * * 1': 'Weekly on Monday',
    '0 0 1 * *': 'Monthly on the 1st',
  }

  if (presets[expr]) return presets[expr]

  // Parse basic cron
  const parts = expr.split(' ')
  if (parts.length !== 5) return expr

  const [min, hour, dayOfMonth, month, dayOfWeek] = parts

  // Very basic descriptions
  if (min === '*' && hour === '*') return 'Every minute'
  if (min?.startsWith('*/')) return `Every ${min.slice(2)} minutes`
  if (hour?.startsWith('*/') && min === '0') return `Every ${hour.slice(2)} hours`
  if (hour === '*' && min === '0') return 'Every hour'
  if (dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    if (hour !== '*' && hour !== undefined && min !== '*' && min !== undefined) {
      const h = parseInt(hour)
      const ampm = h >= 12 ? 'pm' : 'am'
      const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h
      return `Daily at ${h12}:${min.padStart(2, '0')}${ampm}`
    }
  }

  return expr
}
</script>

<template>
  <AppLayout title="Schedules">
    <template #header-actions>
      <LButton v-if="canEdit" @click="router.push({ name: 'schedule-new' })">
        <Plus :size="16" />
        New Schedule
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <!-- Empty state -->
    <LEmptyState
      v-else-if="schedules.length === 0"
      title="No schedules configured"
      :description="canEdit ? 'Schedule your queries to run automatically at specified intervals.' : 'No schedules have been configured yet. Contact an editor or admin to create schedules.'"
    >
      <template #icon>
        <CalendarIcon :size="48" class="text-text-subtle" />
      </template>
      <template #action>
        <LButton v-if="canEdit" @click="router.push({ name: 'schedule-new' })">
          <Plus :size="16" />
          Create Schedule
        </LButton>
      </template>
    </LEmptyState>

    <!-- Schedules list -->
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
        v-if="filteredSchedules.length === 0 && selectedTags.length > 0"
        class="text-center py-8 text-text-muted"
      >
        No schedules match the selected tags
      </div>

      <LCard v-for="schedule in filteredSchedules" :key="schedule.id">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div
              :class="[
                'w-10 h-10 rounded-lg flex items-center justify-center',
                schedule.enabled ? 'bg-success-muted' : 'bg-surface-sunken',
              ]"
            >
              <Clock :size="20" :class="[schedule.enabled ? 'text-success' : 'text-text-subtle']" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3
                  class="font-medium text-text cursor-pointer hover:text-primary"
                  @click="router.push({ name: 'schedule-editor', params: { id: schedule.id } })"
                >
                  {{ schedule.name }}
                </h3>
                <LBadge :variant="schedule.enabled ? 'success' : 'default'">
                  {{ schedule.enabled ? 'Active' : 'Paused' }}
                </LBadge>
              </div>
              <p class="text-sm text-text-muted">
                {{ describeCron(schedule.cron_expression) }}
                <span class="font-mono text-text-subtle text-xs ml-1"
                  >({{ schedule.cron_expression }})</span
                >
              </p>
              <!-- Tags display -->
              <div
                v-if="schedule.tags && schedule.tags.length > 0"
                class="flex flex-wrap gap-1 mt-1"
              >
                <LBadge v-for="tag in schedule.tags" :key="tag" size="sm">
                  <Tag :size="12" class="mr-1" />
                  {{ tag }}
                </LBadge>
              </div>
            </div>
          </div>

          <div class="flex items-center gap-6">
            <div class="text-right">
              <p class="text-xs text-text-subtle">Last run</p>
              <p class="text-sm text-text-muted">{{ formatDate(schedule.last_run_at) }}</p>
            </div>
            <div class="text-right">
              <p class="text-xs text-text-subtle">Next run</p>
              <p class="text-sm text-text-muted">{{ formatDate(schedule.next_run_at) }}</p>
            </div>

            <div class="flex items-center gap-2">
              <LButton
                variant="ghost"
                size="sm"
                title="Run now"
                @click="triggerSchedule(schedule.id)"
              >
                <Play :size="16" />
              </LButton>
              <LButton
                variant="ghost"
                size="sm"
                title="Edit"
                @click="router.push({ name: 'schedule-editor', params: { id: schedule.id } })"
              >
                <Pencil :size="16" />
              </LButton>
              <LButton variant="ghost" size="sm" title="Delete" @click="confirmDelete(schedule)">
                <Trash2 :size="16" />
              </LButton>
              <LButton variant="secondary" size="sm" @click="toggleSchedule(schedule)">
                <component :is="schedule.enabled ? Pause : Play" :size="16" />
                {{ schedule.enabled ? 'Pause' : 'Enable' }}
              </LButton>
            </div>
          </div>
        </div>
      </LCard>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="showDeleteConfirm" class="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/50" @click="showDeleteConfirm = false" />
      <div
        class="relative bg-surface-overlay rounded-xl shadow-xl border border-border p-6 max-w-sm w-full"
      >
        <h3 class="text-lg font-semibold text-text mb-2">Delete Schedule</h3>
        <p class="text-text-muted text-sm mb-4">
          Are you sure you want to delete "{{ scheduleToDelete?.name }}"? This action cannot be
          undone.
        </p>
        <div class="flex justify-end gap-2">
          <LButton variant="ghost" @click="showDeleteConfirm = false">Cancel</LButton>
          <LButton variant="danger" @click="deleteSchedule" :disabled="deleting">
            <Loader2 v-if="deleting" :size="16" class="animate-spin" />
            Delete
          </LButton>
        </div>
      </div>
    </div>
  </AppLayout>
</template>
