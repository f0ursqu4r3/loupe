<script setup lang="ts">
import { ref, watch } from 'vue'
import { AppLayout } from '@/components/layout'
import { LCard, LInput, LButton, LCheckbox, LBadge } from '@/components/ui'
import { ThemeToggle } from '@/components/layout'
import { useAuthStore } from '@/stores/auth'
import { User, Mail, Building2, Calendar, Shield } from 'lucide-vue-next'

const authStore = useAuthStore()

const displayName = ref(authStore.user?.name ?? '')

watch(
  () => authStore.user?.name,
  (name) => {
    displayName.value = name ?? ''
  },
)

function formatDate(dateStr: string | undefined) {
  if (!dateStr) return '—'
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}
</script>

<template>
  <AppLayout title="Settings">
    <div class="max-w-2xl space-y-6">
      <!-- Profile Overview -->
      <LCard>
        <div class="flex items-start gap-4">
          <div
            class="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center shrink-0"
          >
            <User class="w-8 h-8 text-primary" />
          </div>
          <div class="flex-1 min-w-0">
            <h2 class="text-xl font-semibold text-text truncate">
              {{ authStore.user?.name || 'Anonymous' }}
            </h2>
            <p class="text-sm text-text-muted truncate">{{ authStore.user?.email }}</p>
            <div class="flex items-center gap-2 mt-2">
              <LBadge variant="primary">
                <Shield class="w-3 h-3 mr-1" />
                {{ authStore.user?.role || 'user' }}
              </LBadge>
            </div>
          </div>
        </div>
      </LCard>

      <!-- Account Details -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Account Details</h2>

        <div class="space-y-4">
          <div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
            <Mail class="w-5 h-5 text-text-muted" />
            <div class="flex-1 min-w-0">
              <p class="text-xs text-text-muted">Email</p>
              <p class="text-sm text-text truncate">{{ authStore.user?.email }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
            <Building2 class="w-5 h-5 text-text-muted" />
            <div class="flex-1 min-w-0">
              <p class="text-xs text-text-muted">Organization ID</p>
              <p class="text-sm text-text font-mono truncate">{{ authStore.user?.org_id }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
            <Calendar class="w-5 h-5 text-text-muted" />
            <div class="flex-1 min-w-0">
              <p class="text-xs text-text-muted">Member Since</p>
              <p class="text-sm text-text">{{ formatDate(authStore.user?.created_at) }}</p>
            </div>
          </div>
        </div>
      </LCard>

      <!-- Edit Profile -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Edit Profile</h2>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Display Name</label>
            <LInput v-model="displayName" placeholder="Your name" />
          </div>

          <div class="pt-2">
            <LButton>Save Changes</LButton>
          </div>
        </div>
      </LCard>

      <!-- Appearance -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Appearance</h2>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text mb-2">Theme</label>
            <p class="text-sm text-text-muted mb-3">
              Choose how Loupe looks to you. Select a single theme, or sync with your system.
            </p>
            <ThemeToggle />
          </div>
        </div>
      </LCard>

      <!-- Notifications -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Notifications</h2>

        <div class="space-y-4">
          <label class="flex items-center gap-3 cursor-pointer">
            <LCheckbox :model-value="true" />
            <div>
              <p class="text-sm font-medium text-text">Query failures</p>
              <p class="text-xs text-text-muted">Get notified when a scheduled query fails</p>
            </div>
          </label>

          <label class="flex items-center gap-3 cursor-pointer">
            <LCheckbox :model-value="false" />
            <div>
              <p class="text-sm font-medium text-text">Schedule reminders</p>
              <p class="text-xs text-text-muted">Remind me before scheduled queries run</p>
            </div>
          </label>
        </div>
      </LCard>

      <!-- Danger Zone -->
      <LCard class="border-error/50">
        <h2 class="text-lg font-semibold text-error mb-4">Danger Zone</h2>

        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text">Delete Account</p>
            <p class="text-xs text-text-muted">
              Permanently delete your account and all associated data
            </p>
          </div>
          <LButton variant="danger">Delete Account</LButton>
        </div>
      </LCard>
    </div>
  </AppLayout>
</template>
