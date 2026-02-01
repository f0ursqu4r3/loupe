<script setup lang="ts">
import { computed } from 'vue'
import { Loader2 } from 'lucide-vue-next'

interface Props {
  size?: 'sm' | 'md' | 'lg'
  color?: 'primary' | 'current'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  color: 'primary',
})

const sizeClasses = {
  sm: 'h-4 w-4',
  md: 'h-6 w-6',
  lg: 'h-8 w-8',
}

const colorClasses = {
  primary: 'text-primary-600',
  current: 'text-current',
}

const classes = computed(() => ['animate-spin', sizeClasses[props.size], colorClasses[props.color]])
</script>

<template>
  <div role="status" aria-label="Loading">
    <Loader2 :class="classes" aria-hidden="true" />
    <span class="sr-only">Loading...</span>
  </div>
</template>
