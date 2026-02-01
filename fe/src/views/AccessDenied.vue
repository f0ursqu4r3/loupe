<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ShieldAlert, Home, ArrowLeft } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard } from '@/components/ui'
import { usePermissions } from '@/composables/usePermissions'

const route = useRoute()
const router = useRouter()
const { role } = usePermissions()

const requiredPermission = computed(() => {
  return (route.query.required as string) || 'Unknown'
})

const currentRole = computed(() => {
  return role.value || 'none'
})

function goHome() {
  router.push({ name: 'home' })
}

function goBack() {
  router.go(-1)
}
</script>

<template>
  <AppLayout title="Access Denied">
    <div class="flex items-center justify-center min-h-[60vh]">
      <LCard class="max-w-md w-full text-center">
        <div class="flex justify-center mb-6">
          <div
            class="w-20 h-20 rounded-full bg-error-muted flex items-center justify-center"
          >
            <ShieldAlert :size="48" class="text-error" />
          </div>
        </div>

        <h1 class="text-2xl font-bold text-text mb-2">Access Denied</h1>
        <p class="text-text-muted mb-6">
          You don't have permission to access this page.
        </p>

        <div class="bg-surface-sunken rounded-lg p-4 mb-6">
          <div class="flex items-center justify-between text-sm mb-2">
            <span class="text-text-muted">Your Role:</span>
            <span class="font-medium text-text capitalize">{{ currentRole }}</span>
          </div>
          <div class="flex items-center justify-between text-sm">
            <span class="text-text-muted">Required:</span>
            <span class="font-medium text-text capitalize">{{ requiredPermission }}</span>
          </div>
        </div>

        <div class="space-y-4">
          <p class="text-sm text-text-muted">
            If you believe you should have access to this page, please contact your administrator to request the appropriate permissions.
          </p>

          <div class="flex gap-3 justify-center">
            <LButton variant="secondary" @click="goBack">
              <ArrowLeft :size="16" />
              Go Back
            </LButton>
            <LButton @click="goHome">
              <Home :size="16" />
              Go Home
            </LButton>
          </div>
        </div>
      </LCard>
    </div>
  </AppLayout>
</template>
