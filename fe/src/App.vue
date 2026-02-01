<script setup lang="ts">
import { RouterView } from 'vue-router'
import { initTheme } from '@/composables/useTheme'

// Initialize theme from localStorage immediately
initTheme()
</script>

<template>
  <RouterView v-slot="{ Component, route }">
    <component :is="Component" :key="route.path" />
  </RouterView>
</template>
