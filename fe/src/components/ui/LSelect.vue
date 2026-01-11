<script setup lang="ts">
import { computed } from 'vue'
import { ChevronDown } from 'lucide-vue-next'

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
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  error: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const baseClasses =
  'w-full h-10 px-3 pr-10 rounded-md border bg-surface text-text transition-colors focus-ring appearance-none cursor-pointer'

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
      :class="[baseClasses, stateClasses]"
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
      class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-muted pointer-events-none"
    />
  </div>
</template>
