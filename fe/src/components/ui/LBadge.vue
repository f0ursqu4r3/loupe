<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'
type Variant = 'default' | 'primary' | 'success' | 'warning' | 'error' | 'info'

interface Props {
  variant?: Variant
  size?: Size
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'sm',
})

const baseClasses = 'inline-flex items-center font-medium rounded-full'

const variantClasses: Record<Variant, string> = {
  default: 'bg-surface-sunken text-text-muted',
  primary: 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-300',
  success: 'bg-success-muted text-success',
  warning: 'bg-warning-muted text-warning',
  error: 'bg-error-muted text-error',
  info: 'bg-info-muted text-info',
}

const sizeClasses: Record<Size, string> = {
  xs: 'px-1.5 py-0.5 text-[10px]',
  sm: 'px-2 py-0.5 text-xs',
  md: 'px-2.5 py-1 text-sm',
  lg: 'px-3 py-1 text-base',
  xl: 'px-4 py-1.5 text-lg',
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
