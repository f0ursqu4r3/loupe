<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

interface Props {
  padding?: 'none' | 'sm' | 'md' | 'lg'
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  padding: 'md',
})

const paddingClasses = {
  none: '',
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
}

const classes = computed(() => [
  'bg-surface-raised border border-border rounded-lg shadow-sm',
  paddingClasses[props.padding],
  props.class,
])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>
