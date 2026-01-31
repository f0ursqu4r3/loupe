<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'
import { X } from 'lucide-vue-next'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'
type Variant = 'default' | 'primary' | 'success' | 'warning' | 'error' | 'info'
type Style = 'filled' | 'outline' | 'dot'

interface Props {
  variant?: Variant
  size?: Size
  badgeStyle?: Style
  class?: HTMLAttributes['class']
  removable?: boolean
  pulse?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'sm',
  badgeStyle: 'filled',
  removable: false,
  pulse: false,
})

const emit = defineEmits<{
  remove: []
}>()

const baseClasses = 'inline-flex items-center font-medium rounded-full gap-1'

const variantClasses: Record<Style, Record<Variant, string>> = {
  filled: {
    default: 'bg-surface-sunken text-text-muted',
    primary: 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-300',
    success: 'bg-success-muted text-success',
    warning: 'bg-warning-muted text-warning',
    error: 'bg-error-muted text-error',
    info: 'bg-info-muted text-info',
  },
  outline: {
    default: 'border border-border text-text',
    primary: 'border border-primary-600 text-primary-600',
    success: 'border border-success text-success',
    warning: 'border border-warning text-warning',
    error: 'border border-error text-error',
    info: 'border border-info text-info',
  },
  dot: {
    default: 'text-text-muted',
    primary: 'text-primary-600',
    success: 'text-success',
    warning: 'text-warning',
    error: 'text-error',
    info: 'text-info',
  },
}

const sizeClasses: Record<Size, string> = {
  xs: 'px-1.5 py-0.5 text-[10px]',
  sm: 'px-2 py-0.5 text-xs',
  md: 'px-2.5 py-1 text-sm',
  lg: 'px-3 py-1 text-base',
  xl: 'px-4 py-1.5 text-lg',
}

const dotSizeClasses: Record<Size, string> = {
  xs: 'w-1 h-1',
  sm: 'w-1.5 h-1.5',
  md: 'w-2 h-2',
  lg: 'w-2.5 h-2.5',
  xl: 'w-3 h-3',
}

const closeSizeClasses: Record<Size, number> = {
  xs: 10,
  sm: 12,
  md: 14,
  lg: 16,
  xl: 18,
}

const classes = computed(() => [
  baseClasses,
  variantClasses[props.badgeStyle][props.variant],
  props.badgeStyle !== 'dot' ? sizeClasses[props.size] : '',
  props.class,
])

function handleRemove() {
  emit('remove')
}
</script>

<template>
  <span :class="classes">
    <!-- Dot indicator -->
    <span
      v-if="badgeStyle === 'dot'"
      :class="[
        'rounded-full',
        dotSizeClasses[size],
        variant === 'default' ? 'bg-text-muted' : `bg-current`,
        pulse ? 'animate-pulse' : '',
      ]"
    />

    <!-- Icon slot -->
    <slot name="icon" />

    <!-- Content -->
    <slot />

    <!-- Remove button -->
    <button
      v-if="removable"
      type="button"
      class="hover:opacity-70 transition-opacity"
      @click.stop="handleRemove"
      aria-label="Remove"
    >
      <X :size="closeSizeClasses[size]" />
    </button>
  </span>
</template>
