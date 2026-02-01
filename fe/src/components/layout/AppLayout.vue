<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import AppSidebar from './AppSidebar.vue'
import AppHeader from './AppHeader.vue'
import KeyboardShortcutsModal from '@/components/help/KeyboardShortcutsModal.vue'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'

interface Props {
  title?: string
  back?: string
  noPadding?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  back: '',
  noPadding: false,
})

const SIDEBAR_STORAGE_KEY = 'loupe-sidebar-collapsed'

function getSavedSidebarState(): boolean {
  if (typeof window === 'undefined') return false
  const saved = localStorage.getItem(SIDEBAR_STORAGE_KEY)
  return saved === 'true'
}

function saveSidebarState(value: boolean) {
  if (typeof window === 'undefined') return
  localStorage.setItem(SIDEBAR_STORAGE_KEY, value ? 'true' : 'false')
}

const sidebarCollapsed = ref(getSavedSidebarState())

watch(sidebarCollapsed, (value) => {
  saveSidebarState(value)
})

const mainClasses = computed(() => [
  'min-h-screen transition-all duration-300',
  sidebarCollapsed.value ? 'ml-16' : 'ml-64',
])

// Keyboard shortcuts
useKeyboardShortcuts()
const showKeyboardShortcuts = ref(false)

function handleShowKeyboardShortcuts() {
  showKeyboardShortcuts.value = true
}

function handleToggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

onMounted(() => {
  window.addEventListener('show-keyboard-shortcuts', handleShowKeyboardShortcuts)
  window.addEventListener('toggle-sidebar', handleToggleSidebar)
})

onUnmounted(() => {
  window.removeEventListener('show-keyboard-shortcuts', handleShowKeyboardShortcuts)
  window.removeEventListener('toggle-sidebar', handleToggleSidebar)
})
</script>

<template>
  <div class="h-screen bg-surface overflow-hidden">
    <!-- Skip to main content link for keyboard navigation -->
    <a
      href="#main-content"
      class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-primary-600 focus:text-white focus:rounded-md focus:shadow-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
    >
      Skip to main content
    </a>

    <AppSidebar v-model:collapsed="sidebarCollapsed" />

    <div :class="mainClasses">
      <AppHeader :title="props.title" :back="props.back" class="sticky top-0 z-10">
        <template #actions>
          <slot name="header-actions" />
        </template>
      </AppHeader>

      <main
        id="main-content"
        :class="[{ 'p-6': !props.noPadding }, 'overflow-auto h-[calc(100vh-4rem)]']"
        tabindex="-1"
      >
        <slot />
      </main>
    </div>

    <!-- Keyboard shortcuts help modal -->
    <KeyboardShortcutsModal v-model="showKeyboardShortcuts" />
  </div>
</template>
