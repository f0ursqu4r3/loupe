<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

interface Props {
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info'
  size?: 'sm' | 'md'
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'sm',
})

const baseClasses = 'inline-flex items-center font-medium rounded-full'

const variantClasses = {
  default: 'bg-surface-sunken text-text-muted',
  success: 'bg-success-muted text-success',
  warning: 'bg-warning-muted text-warning',
  error: 'bg-error-muted text-error',
  info: 'bg-info-muted text-info',
}

const sizeClasses = {
  sm: 'px-2 py-0.5 text-xs',
  md: 'px-2.5 py-1 text-sm',
}

const classes = computed(() => [
  baseClasses,
  variantClasses[props.variant],
  sizeClasses[props.size],
  props.class,
])
</script>

<template>
  <span :class="classes">
    <slot />
  </span>
</template>
