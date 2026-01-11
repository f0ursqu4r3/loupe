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
  <div class="min-h-screen bg-surface">
    <AppSidebar v-model:collapsed="sidebarCollapsed" />

    <div :class="mainClasses">
      <AppHeader :title="title">
        <template #left>
          <slot name="header-left" />
        </template>
        <template #actions>
          <slot name="header-actions" />
        </template>
      </AppHeader>

      <main class="p-6">
        <slot />
      </main>
    </div>
  </div>
</template>
