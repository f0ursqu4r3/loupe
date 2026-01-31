<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

type Variant = 'default' | 'outlined' | 'filled' | 'elevated' | 'interactive'
type Padding = 'none' | 'sm' | 'md' | 'lg'

interface Props {
  variant?: Variant
  padding?: Padding
  class?: HTMLAttributes['class']
  /** Optional accent border color (left border) */
  accent?: 'primary' | 'success' | 'warning' | 'error' | 'info'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  padding: 'md',
})

const paddingClasses: Record<Padding, string> = {
  none: '',
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
}

const variantClasses: Record<Variant, string> = {
  default: 'bg-surface-raised border border-border shadow-sm',
  outlined: 'bg-surface border-2 border-border',
  filled: 'bg-surface-sunken border border-transparent',
  elevated: 'bg-surface-raised border border-border shadow-lg',
  interactive: 'bg-surface-raised border border-border shadow-sm hover:shadow-md transition-shadow',
}

const accentClasses = computed(() => {
  if (!props.accent) return ''
  const colors = {
    primary: 'border-l-4 border-l-primary-600',
    success: 'border-l-4 border-l-success',
    warning: 'border-l-4 border-l-warning',
    error: 'border-l-4 border-l-error',
    info: 'border-l-4 border-l-info',
  }
  return colors[props.accent]
})

const classes = computed(() => [
  'rounded-lg',
  variantClasses[props.variant],
  accentClasses.value,
  paddingClasses[props.padding],
  props.class,
])
</script>

<template>
  <div :class="classes">
    <!-- Optional header -->
    <div v-if="$slots.header" class="mb-4 pb-4 border-b border-border">
      <slot name="header" />
    </div>

    <!-- Main content -->
    <slot />

    <!-- Optional footer -->
    <div v-if="$slots.footer" class="mt-4 pt-4 border-t border-border">
      <slot name="footer" />
    </div>
  </div>
</template>
