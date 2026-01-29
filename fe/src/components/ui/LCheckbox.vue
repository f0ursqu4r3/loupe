<script setup lang="ts">
import { computed } from 'vue'
import { Check } from 'lucide-vue-next'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

interface Props {
  modelValue?: boolean
  disabled?: boolean
  id?: string
  size?: Size
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  size: 'md',
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const isChecked = computed(() => props.modelValue ?? false)

const sizeClasses: Record<Size, string> = {
  xs: 'h-3.5 w-3.5',
  sm: 'h-4 w-4',
  md: 'h-5 w-5',
  lg: 'h-6 w-6',
  xl: 'h-7 w-7',
}

const iconSizes: Record<Size, string> = {
  xs: 'h-2.5 w-2.5',
  sm: 'h-3 w-3',
  md: 'h-3.5 w-3.5',
  lg: 'h-4 w-4',
  xl: 'h-5 w-5',
}

function toggle() {
  if (!props.disabled) {
    emit('update:modelValue', !isChecked.value)
  }
}
</script>

<template>
  <button
    type="button"
    role="checkbox"
    :aria-checked="isChecked"
    :disabled="disabled"
    :id="id"
    class="rounded border transition-colors focus-ring flex items-center justify-center"
    :class="[
      sizeClasses[props.size],
      isChecked
        ? 'bg-primary-600 border-primary-600 text-white'
        : 'bg-surface border-border hover:border-border-strong',
      props.disabled && 'opacity-50 cursor-not-allowed',
    ]"
    @click="toggle"
  >
    <Check v-if="isChecked" :class="iconSizes[props.size]" />
  </button>
</template>
