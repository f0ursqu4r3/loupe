<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

interface Props {
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  error?: boolean
  rows?: number
  size?: Size
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  error: false,
  rows: 4,
  size: 'md',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const baseClasses =
  'w-full rounded-md border bg-surface text-text placeholder:text-text-subtle transition-colors focus-ring resize-y'

const sizeClasses: Record<Size, string> = {
  xs: 'px-2 py-1.5 text-xs',
  sm: 'px-2.5 py-1.5 text-sm',
  md: 'px-3 py-2 text-sm',
  lg: 'px-4 py-2.5 text-base',
  xl: 'px-5 py-3 text-lg',
}

const stateClasses = computed(() => ({
  'border-border hover:border-border-strong focus:border-primary-500': !props.error,
  'border-error focus:border-error': props.error,
  'opacity-50 cursor-not-allowed': props.disabled,
}))

function onInput(event: Event) {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <textarea
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :rows="rows"
    :class="[baseClasses, sizeClasses[size], stateClasses, props.class]"
    @input="onInput"
  />
</template>
