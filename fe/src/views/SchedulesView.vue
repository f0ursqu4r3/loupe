<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, Play, Pause, Clock, Calendar as CalendarIcon } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LBadge, LEmptyState, LSpinner } from '@/components/ui'
import { schedulesApi } from '@/services/api'
import type { Schedule } from '@/types'

const schedules = ref<Schedule[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

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

onMounted(loadSchedules)

function formatDate(dateString: string | undefined): string {
  if (!dateString) return 'Never'
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(new Date(dateString))
}
</script>

<template>
  <AppLayout title="Schedules">
    <template #header-actions>
      <LButton>
        <Plus class="h-4 w-4" />
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
      description="Schedule your queries to run automatically at specified intervals."
    >
      <template #icon>
        <CalendarIcon class="h-8 w-8 text-text-subtle" />
      </template>
      <template #action>
        <LButton>
          <Plus class="h-4 w-4" />
          Create Schedule
        </LButton>
      </template>
    </LEmptyState>

    <!-- Schedules list -->
    <div v-else class="space-y-4">
      <LCard v-for="schedule in schedules" :key="schedule.id">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div
              :class="[
                'w-10 h-10 rounded-lg flex items-center justify-center',
                schedule.enabled
                  ? 'bg-success-muted'
                  : 'bg-surface-sunken',
              ]"
            >
              <Clock
                :class="[
                  'h-5 w-5',
                  schedule.enabled ? 'text-success' : 'text-text-subtle',
                ]"
              />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="font-medium text-text">{{ schedule.name }}</h3>
                <LBadge :variant="schedule.enabled ? 'success' : 'default'">
                  {{ schedule.enabled ? 'Active' : 'Paused' }}
                </LBadge>
              </div>
              <p class="text-sm text-text-muted font-mono">
                {{ schedule.cron_expression }}
              </p>
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
                @click="triggerSchedule(schedule.id)"
              >
                <Play class="h-4 w-4" />
              </LButton>
              <LButton
                variant="secondary"
                size="sm"
                @click="toggleSchedule(schedule)"
              >
                <component :is="schedule.enabled ? Pause : Play" class="h-4 w-4" />
                {{ schedule.enabled ? 'Pause' : 'Enable' }}
              </LButton>
            </div>
          </div>
        </div>
      </LCard>
    </div>
  </AppLayout>
</template>
