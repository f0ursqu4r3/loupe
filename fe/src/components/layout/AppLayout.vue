<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import AppSidebar from './AppSidebar.vue'
import AppHeader from './AppHeader.vue'

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
</script>

<template>
  <div class="h-screen bg-surface overflow-hidden">
    <AppSidebar v-model:collapsed="sidebarCollapsed" />

    <div :class="mainClasses">
      <AppHeader :title="props.title" :back="props.back" class="sticky top-0 z-10">
        <template #actions>
          <slot name="header-actions" />
        </template>
      </AppHeader>

      <main :class="[{ 'p-6': !props.noPadding }, 'overflow-auto h-[calc(100vh-4rem)]']">
        <slot />
      </main>
    </div>
  </div>
</template>
