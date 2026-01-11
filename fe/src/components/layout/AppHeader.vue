<script setup lang="ts">
import { Bell, User } from 'lucide-vue-next'
import ThemeToggle from './ThemeToggle.vue'

interface Props {
  title?: string
}

defineProps<Props>()
</script>

<template>
  <header class="h-16 flex items-center justify-between px-6 border-b border-border bg-surface">
    <!-- Page title / breadcrumbs -->
    <div class="flex items-center gap-4">
      <h1 v-if="title" class="text-xl font-semibold text-text">
        {{ title }}
      </h1>
      <slot name="left" />
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-3">
      <slot name="actions" />

      <ThemeToggle />

      <button
        type="button"
        class="p-2 rounded-lg text-text-muted hover:text-text hover:bg-surface-sunken transition-colors focus-ring"
      >
        <Bell class="h-5 w-5" />
      </button>

      <button
        type="button"
        class="flex items-center gap-2 p-1.5 rounded-lg hover:bg-surface-sunken transition-colors focus-ring"
      >
        <div class="w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center">
          <User class="h-4 w-4 text-white" />
        </div>
      </button>
    </div>
  </header>
</template>
