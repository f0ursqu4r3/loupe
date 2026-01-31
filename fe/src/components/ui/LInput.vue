<script setup lang="ts">
import { computed, ref, type HTMLAttributes } from 'vue'
import { X, AlertCircle } from 'lucide-vue-next'

type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

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
  error?: boolean | string
  helperText?: string
  size?: Size
  class?: HTMLAttributes['class']
  clearable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  disabled: false,
  error: false,
  size: 'md',
  clearable: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
  clear: []
}>()

const inputRef = ref<HTMLInputElement>()

const hasPrefix = computed(() => !!props.$slots?.prefix)
const hasSuffix = computed(() => !!props.$slots?.suffix || props.clearable)
const errorMessage = computed(() => (typeof props.error === 'string' ? props.error : ''))
const hasError = computed(() => !!props.error)

const baseClasses =
  'w-full rounded-md border bg-surface text-text placeholder:text-text-subtle transition-colors focus-ring'

const sizeClasses: Record<Size, string> = {
  xs: 'h-6 text-xs',
  sm: 'h-8 text-sm',
  md: 'h-10 text-sm',
  lg: 'h-12 text-base',
  xl: 'h-14 text-lg',
}

const paddingClasses = computed(() => {
  const sizes = {
    xs: hasPrefix.value ? 'pl-7' : 'pl-2',
    sm: hasPrefix.value ? 'pl-8' : 'pl-2.5',
    md: hasPrefix.value ? 'pl-9' : 'pl-3',
    lg: hasPrefix.value ? 'pl-11' : 'pl-4',
    xl: hasPrefix.value ? 'pl-12' : 'pl-5',
  }
  const rightPadding = {
    xs: hasSuffix.value ? 'pr-7' : 'pr-2',
    sm: hasSuffix.value ? 'pr-8' : 'pr-2.5',
    md: hasSuffix.value ? 'pr-9' : 'pr-3',
    lg: hasSuffix.value ? 'pr-11' : 'pr-4',
    xl: hasSuffix.value ? 'pr-12' : 'pr-5',
  }
  return `${sizes[props.size]} ${rightPadding[props.size]}`
})

const stateClasses = computed(() => ({
  'border-border hover:border-border-strong focus:border-primary-500 focus:shadow-[0_0_0_3px_rgba(var(--loupe-primary-500)/0.1)]':
    !hasError.value,
  'border-error focus:border-error focus:shadow-[0_0_0_3px_rgba(var(--loupe-error)/0.1)]':
    hasError.value,
  'opacity-50 cursor-not-allowed': props.disabled,
}))

function onInput(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', props.type === 'number' ? target.valueAsNumber : target.value)
}

function handleClear() {
  emit('update:modelValue', '')
  emit('clear')
  inputRef.value?.focus()
}
</script>

<template>
  <div class="w-full">
    <!-- Input wrapper with prefix/suffix -->
    <div class="relative">
      <!-- Prefix slot -->
      <div
        v-if="$slots.prefix"
        class="absolute left-0 top-0 h-full flex items-center justify-center px-3 text-text-muted pointer-events-none"
      >
        <slot name="prefix" />
      </div>

      <!-- Input -->
      <input
        ref="inputRef"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :aria-invalid="hasError"
        :aria-describedby="errorMessage || helperText ? 'input-helper' : undefined"
        :class="[baseClasses, sizeClasses[size], paddingClasses, stateClasses, props.class]"
        @input="onInput"
      />

      <!-- Suffix slot or clear button -->
      <div
        v-if="$slots.suffix || (clearable && modelValue)"
        class="absolute right-0 top-0 h-full flex items-center justify-center px-3"
      >
        <slot name="suffix">
          <button
            v-if="clearable && modelValue"
            type="button"
            class="text-text-muted hover:text-text transition-colors"
            @click="handleClear"
            aria-label="Clear input"
          >
            <X :size="16" />
          </button>
        </slot>
      </div>
    </div>

    <!-- Error message -->
    <div v-if="errorMessage" id="input-helper" class="flex items-center gap-1 mt-1 text-xs text-error">
      <AlertCircle :size="12" />
      <span>{{ errorMessage }}</span>
    </div>

    <!-- Helper text -->
    <div
      v-else-if="helperText"
      id="input-helper"
      class="mt-1 text-xs text-text-muted"
    >
      {{ helperText }}
    </div>
  </div>
</template>
