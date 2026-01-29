<script setup lang="ts">
import { computed } from 'vue'
import { ChevronDown } from 'lucide-vue-next'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

interface Option {
  value: string | number
  label: string
  disabled?: boolean
}

interface Props {
  modelValue?: string | number
  options: Option[]
  placeholder?: string
  disabled?: boolean
  error?: boolean
  size?: Size
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  error: false,
  size: 'md',
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const baseClasses =
  'w-full rounded-md border bg-surface text-text transition-colors focus-ring appearance-none cursor-pointer'

const sizeClasses: Record<Size, string> = {
  xs: 'h-6 px-2 pr-7 text-xs',
  sm: 'h-8 px-2.5 pr-8 text-sm',
  md: 'h-10 px-3 pr-10 text-sm',
  lg: 'h-12 px-4 pr-11 text-base',
  xl: 'h-14 px-5 pr-12 text-lg',
}

const iconSizes: Record<Size, string> = {
  xs: 'h-3 w-3 right-1.5',
  sm: 'h-3.5 w-3.5 right-2',
  md: 'h-4 w-4 right-3',
  lg: 'h-5 w-5 right-3',
  xl: 'h-6 w-6 right-4',
}

const stateClasses = computed(() => ({
  'border-border hover:border-border-strong focus:border-primary-500': !props.error,
  'border-error focus:border-error': props.error,
  'opacity-50 cursor-not-allowed': props.disabled,
}))

function onChange(event: Event) {
  const target = event.target as HTMLSelectElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <div class="relative">
    <select
      :value="modelValue"
      :disabled="disabled"
      :class="[baseClasses, sizeClasses[size], stateClasses]"
      @change="onChange"
    >
      <option v-if="placeholder" value="" disabled>{{ placeholder }}</option>
      <option
        v-for="option in options"
        :key="option.value"
        :value="option.value"
        :disabled="option.disabled"
      >
        {{ option.label }}
      </option>
    </select>
    <ChevronDown
      :class="['absolute top-1/2 -translate-y-1/2 text-text-muted pointer-events-none', iconSizes[size]]"
    />
  </div>
</template>
