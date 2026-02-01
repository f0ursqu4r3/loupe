<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

type Variant = 'text' | 'rectangle' | 'circle' | 'card'

interface Props {
  variant?: Variant
  width?: string
  height?: string
  class?: HTMLAttributes['class']
  animate?: boolean
  /** Number of lines for text variant */
  lines?: number
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'rectangle',
  animate: true,
  lines: 1,
})

const baseClasses = 'bg-surface-sunken'

const variantClasses: Record<Variant, string> = {
  text: 'rounded h-4',
  rectangle: 'rounded-lg',
  circle: 'rounded-full',
  card: 'rounded-lg',
}

const animationClass = props.animate ? 'animate-shimmer' : ''

const classes = computed(() => [baseClasses, variantClasses[props.variant], animationClass, props.class])

const style = computed(() => ({
  width: props.width,
  height: props.height,
}))

// For card variant, provide a structured skeleton
const isCard = computed(() => props.variant === 'card')
</script>

<template>
  <!-- Card variant with internal structure -->
  <div v-if="isCard" :class="[classes, 'p-4 space-y-3']" :style="style">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-full bg-surface animate-shimmer" />
      <div class="flex-1 space-y-2">
        <div class="h-4 bg-surface rounded w-3/4 animate-shimmer" />
        <div class="h-3 bg-surface rounded w-1/2 animate-shimmer" />
      </div>
    </div>
    <div class="space-y-2">
      <div class="h-3 bg-surface rounded animate-shimmer" />
      <div class="h-3 bg-surface rounded w-5/6 animate-shimmer" />
    </div>
  </div>

  <!-- Text variant with multiple lines -->
  <div v-else-if="variant === 'text' && lines > 1" class="space-y-2">
    <div
      v-for="(_, index) in Array(lines)"
      :key="index"
      :class="classes"
      :style="{
        width: index === lines - 1 ? '75%' : '100%',
      }"
    />
  </div>

  <!-- Simple variants (text with 1 line, rectangle, circle) -->
  <div v-else :class="classes" :style="style" />
</template>
