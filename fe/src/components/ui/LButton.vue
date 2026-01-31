<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'
import { Loader2 } from 'lucide-vue-next'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'
type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline' | 'text' | 'soft' | 'link'

interface Props {
  variant?: Variant
  size?: Size
  disabled?: boolean
  loading?: boolean
  class?: HTMLAttributes['class']
  square?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
  square: false,
})

const baseClasses =
  'inline-flex items-center justify-center font-medium transition-all focus-ring rounded-md cursor-pointer disabled:opacity-50 disabled:pointer-events-none active:scale-95'

const variantClasses: Record<Variant, string> = {
  primary: 'bg-primary-600 text-white hover:bg-primary-700 active:bg-primary-800',
  secondary:
    'bg-surface-raised text-text border border-border hover:bg-surface-sunken active:bg-surface-sunken',
  ghost: 'text-text hover:bg-surface-sunken active:bg-surface-sunken',
  danger: 'bg-error text-white hover:opacity-90 active:opacity-80',
  outline:
    'border border-primary-600 text-primary-600 hover:bg-primary-50 active:bg-primary-100 dark:hover:bg-primary-950 dark:active:bg-primary-900',
  text: 'text-text-muted hover:text-text active:text-text',
  soft: 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 hover:bg-primary-200 dark:hover:bg-primary-900/50 active:bg-primary-300 dark:active:bg-primary-900/70',
  link: 'text-primary-600 hover:text-primary-700 active:text-primary-800 underline-offset-4 hover:underline',
}

const sizeClasses: Record<Size, string> = {
  xs: 'h-6 px-2 text-xs gap-1',
  sm: 'h-8 px-3 text-sm gap-1.5',
  md: 'h-10 px-4 text-sm gap-2',
  lg: 'h-12 px-6 text-base gap-2',
  xl: 'h-14 px-8 text-lg gap-2.5',
}

const squareSizeClasses: Record<Size, string> = {
  xs: 'h-6 w-6 text-xs',
  sm: 'h-8 w-8 text-sm',
  md: 'h-10 w-10 text-sm',
  lg: 'h-12 w-12 text-base',
  xl: 'h-14 w-14 text-lg',
}

const loaderSizes: Record<Size, string> = {
  xs: 'h-3 w-3',
  sm: 'h-3.5 w-3.5',
  md: 'h-4 w-4',
  lg: 'h-5 w-5',
  xl: 'h-6 w-6',
}

const classes = computed(() => [
  baseClasses,
  variantClasses[props.variant],
  props.square ? squareSizeClasses[props.size] : sizeClasses[props.size],
  props.class,
])

const isDisabled = computed(() => props.disabled || props.loading)
</script>

<template>
  <button :class="classes" :disabled="isDisabled">
    <Loader2 v-if="loading" :class="[loaderSizes[size], 'animate-spin']" />
    <slot />
  </button>
</template>
