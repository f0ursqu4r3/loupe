<script setup lang="ts">
import { ref, computed } from 'vue'
import AppSidebar from './AppSidebar.vue'
import AppHeader from './AppHeader.vue'

interface Props {
  title?: string
}

defineProps<Props>()

const sidebarCollapsed = ref(false)

const mainClasses = computed(() => [
  'min-h-screen transition-all duration-300',
  sidebarCollapsed.value ? 'ml-16' : 'ml-64',
])
</script>

<template>
  <div class="h-screen bg-surface overflow-hidden">
    <AppSidebar v-model:collapsed="sidebarCollapsed" />

    <div :class="mainClasses">
      <AppHeader :title="title" class="sticky top-0 z-10">
        <template #left>
          <slot name="header-left" />
        </template>
        <template #actions>
          <slot name="header-actions" />
        </template>
      </AppHeader>

      <main class="p-6 overflow-auto h-[calc(100vh-4rem)]">
        <slot />
      </main>
    </div>
  </div>
</template>
