<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

interface Props {
  modelValue?: string | number
  type?:
    | 'text'
    | 'email'
    | 'password'
    | 'number'
    | 'search'
    | 'url'
    | 'tel'
    | 'date'
    | 'datetime-local'
  placeholder?: string
  disabled?: boolean
  error?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  disabled: false,
  error: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const baseClasses =
  'w-full h-10 px-3 rounded-md border bg-surface text-text placeholder:text-text-subtle transition-colors focus-ring'

const stateClasses = computed(() => ({
  'border-border hover:border-border-strong focus:border-primary-500': !props.error,
  'border-error focus:border-error': props.error,
  'opacity-50 cursor-not-allowed': props.disabled,
}))

function onInput(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', props.type === 'number' ? target.valueAsNumber : target.value)
}
</script>

<template>
  <input
    :type="type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :class="[baseClasses, stateClasses, props.class]"
    @input="onInput"
  />
</template>
