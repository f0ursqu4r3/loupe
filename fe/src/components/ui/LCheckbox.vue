<script setup lang="ts">
import { computed } from 'vue'
import { Check } from 'lucide-vue-next'

interface Props {
  modelValue?: boolean
  disabled?: boolean
  id?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const isChecked = computed(() => props.modelValue ?? false)

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
    class="h-5 w-5 rounded border transition-colors focus-ring flex items-center justify-center"
    :class="[
      isChecked
        ? 'bg-primary-600 border-primary-600 text-white'
        : 'bg-surface border-border hover:border-border-strong',
      disabled && 'opacity-50 cursor-not-allowed',
    ]"
    @click="toggle"
  >
    <Check v-if="isChecked" class="h-3.5 w-3.5" />
  </button>
</template>
