<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

interface Props {
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  error?: boolean
  rows?: number
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  error: false,
  rows: 4,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const baseClasses =
  'w-full px-3 py-2 rounded-md border bg-surface text-text placeholder:text-text-subtle transition-colors focus-ring resize-y'

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
    :class="[baseClasses, stateClasses, props.class]"
    @input="onInput"
  />
</template>
