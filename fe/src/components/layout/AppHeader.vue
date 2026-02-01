<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import ThemeToggle from './ThemeToggle.vue'
import { LButton, LBadge } from '@/components/ui'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { usePermissions } from '@/composables/usePermissions'
import { ArrowLeft, User, LogOut, ChevronDown } from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()
const { getRoleBadgeVariant, getRoleDisplayName } = usePermissions()

const showUserMenu = ref(false)

interface Props {
  title?: string
  back?: string
}

defineProps<Props>()

function logout() {
  authStore.logout()
  router.push({ name: 'auth' })
}

function toggleUserMenu() {
  showUserMenu.value = !showUserMenu.value
}

// Close menu when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.user-menu-container')) {
    showUserMenu.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <header class="h-16 flex items-center justify-between px-6 border-b border-border bg-surface">
    <!-- Page title / breadcrumbs -->
    <div class="flex items-center gap-4">
      <LButton
        v-if="back"
        variant="ghost"
        size="sm"
        aria-label="Go back"
        @click="router.push({ name: back })"
        square
      >
        <ArrowLeft :size="16" />
      </LButton>

      <h1 v-if="title" class="text-xl font-semibold text-text">
        {{ title }}
      </h1>
      <slot name="left" />
    </div>

    <!-- Actions and user menu -->
    <div class="flex items-center gap-4">
      <slot name="actions" />

      <!-- Theme toggle -->
      <ThemeToggle />

      <!-- User menu -->
      <div v-if="authStore.user" class="relative user-menu-container">
        <button
          type="button"
          class="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-surface-hover transition-colors"
          @click="toggleUserMenu"
          aria-haspopup="true"
          :aria-expanded="showUserMenu"
        >
          <div class="flex items-center gap-2">
            <div class="w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
              <User :size="16" class="text-primary-600 dark:text-primary-400" />
            </div>
            <div class="hidden md:block text-left">
              <div class="text-sm font-medium text-text">{{ authStore.user.name }}</div>
              <div class="text-xs text-text-muted">{{ authStore.user.email }}</div>
            </div>
            <ChevronDown :size="16" class="text-text-muted" />
          </div>
        </button>

        <!-- Dropdown menu -->
        <transition
          enter-active-class="transition ease-out duration-100"
          enter-from-class="transform opacity-0 scale-95"
          enter-to-class="transform opacity-100 scale-100"
          leave-active-class="transition ease-in duration-75"
          leave-from-class="transform opacity-100 scale-100"
          leave-to-class="transform opacity-0 scale-95"
        >
          <div
            v-if="showUserMenu"
            class="absolute right-0 mt-2 w-64 rounded-lg shadow-lg bg-surface border border-border overflow-hidden z-50"
          >
            <!-- User info -->
            <div class="px-4 py-3 border-b border-border">
              <div class="flex items-center justify-between">
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium text-text truncate">{{ authStore.user.name }}</p>
                  <p class="text-xs text-text-muted truncate">{{ authStore.user.email }}</p>
                </div>
                <LBadge :variant="getRoleBadgeVariant()" size="sm" class="ml-2">
                  {{ getRoleDisplayName() }}
                </LBadge>
              </div>
            </div>

            <!-- Menu items -->
            <div class="py-1">
              <button
                type="button"
                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-text hover:bg-surface-hover transition-colors"
                @click="logout"
              >
                <LogOut :size="16" class="text-text-muted" />
                <span>Log out</span>
              </button>
            </div>
          </div>
        </transition>
      </div>
    </div>
  </header>
</template>

<style scoped>
.user-menu-container {
  position: relative;
}
</style>
