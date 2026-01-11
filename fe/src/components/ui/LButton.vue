<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'
import { Loader2 } from 'lucide-vue-next'

interface Props {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
})

const baseClasses =
  'inline-flex items-center justify-center font-medium transition-colors focus-ring rounded-md disabled:opacity-50 disabled:pointer-events-none'

const variantClasses = {
  primary: 'bg-primary-600 text-white hover:bg-primary-700 active:bg-primary-800',
  secondary:
    'bg-surface-raised text-text border border-border hover:bg-surface-sunken active:bg-surface-sunken',
  ghost: 'text-text hover:bg-surface-sunken active:bg-surface-sunken',
  danger: 'bg-error text-white hover:opacity-90 active:opacity-80',
  outline:
    'border border-primary-600 text-primary-600 hover:bg-primary-50 active:bg-primary-100 dark:hover:bg-primary-950 dark:active:bg-primary-900',
}

const sizeClasses = {
  sm: 'h-8 px-3 text-sm gap-1.5',
  md: 'h-10 px-4 text-sm gap-2',
  lg: 'h-12 px-6 text-base gap-2',
}

const classes = computed(() => [
  baseClasses,
  variantClasses[props.variant],
  sizeClasses[props.size],
  props.class,
])

const isDisabled = computed(() => props.disabled || props.loading)
</script>

<template>
  <button :class="classes" :disabled="isDisabled">
    <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
    <slot />
  </button>
</template>
