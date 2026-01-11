<script setup lang="ts">
import { AppLayout } from '@/components/layout'
import { LCard, LInput, LSelect, LButton, LCheckbox } from '@/components/ui'
import { ThemeToggle } from '@/components/layout'

const themeOptions = [
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
  { value: 'system', label: 'System' },
]
</script>

<template>
  <AppLayout title="Settings">
    <div class="max-w-2xl space-y-6">
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

      <!-- Account -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Account</h2>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Email</label>
            <LInput value="user@example.com" disabled />
          </div>

          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Display Name</label>
            <LInput placeholder="Your name" />
          </div>

          <div class="pt-2">
            <LButton>Save Changes</LButton>
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
