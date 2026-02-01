<script setup lang="ts">
import { computed } from 'vue'
import { Check } from 'lucide-vue-next'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

interface Props {
  modelValue?: boolean
  disabled?: boolean
  label?: string
  id?: string
  indeterminate?: boolean
  size?: Size
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  size: 'md',
  indeterminate: false,
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
  <div
    class="inline-flex items-center gap-2 cursor-pointer"
    :class="props.disabled && 'opacity-50 cursor-not-allowed'"
    @click="toggle"
  >
    <div
      role="checkbox"
      :id="id"
      :aria-checked="isChecked"
      :aria-disabled="disabled"
      :indeterminate="indeterminate"
      tabindex="0"
      class="rounded border transition-colors focus-ring flex items-center justify-center shrink-0"
      :class="[
        sizeClasses[size],
        isChecked
          ? 'bg-primary-600 border-primary-600 text-white'
          : 'bg-surface border-border hover:border-border-strong',
      ]"
      @keydown.space.prevent="toggle"
      @keydown.enter.prevent="toggle"
    >
      <Check v-if="isChecked" :class="iconSizes[size]" />
    </div>
    <span v-if="label" class="select-none text-sm text-text-muted">{{ props.label }}</span>
  </div>
</template>
