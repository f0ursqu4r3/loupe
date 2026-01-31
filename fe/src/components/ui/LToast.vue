<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-vue-next'

type Variant = 'success' | 'error' | 'warning' | 'info'

interface Props {
  variant?: Variant
  title?: string
  message: string
  duration?: number
  closable?: boolean
  actionLabel?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'info',
  duration: 5000,
  closable: true,
})

const emit = defineEmits<{
  close: []
  action: []
}>()

const icons = {
  success: CheckCircle,
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info,
}

const variantClasses = {
  success: 'bg-success-muted border-success text-text',
  error: 'bg-error-muted border-error text-text',
  warning: 'bg-warning-muted border-warning text-text',
  info: 'bg-info-muted border-info text-text',
}

const iconClasses = {
  success: 'text-success',
  error: 'text-error',
  warning: 'text-warning',
  info: 'text-info',
}

let timer: number | null = null

onMounted(() => {
  if (props.duration > 0) {
    timer = window.setTimeout(() => {
      emit('close')
    }, props.duration)
  }
})

onUnmounted(() => {
  if (timer) {
    clearTimeout(timer)
  }
})

function handleClose() {
  emit('close')
}

function handleAction() {
  emit('action')
  emit('close')
}
</script>

<template>
  <div
    :class="[
      'relative flex items-start gap-3 p-4 rounded-lg border-l-4 shadow-lg',
      variantClasses[variant],
      'animate-slide-in-right',
    ]"
  >
    <!-- Icon -->
    <component :is="icons[variant]" :size="20" :class="['shrink-0 mt-0.5', iconClasses[variant]]" />

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <h4 v-if="title" class="font-semibold text-text mb-1">{{ title }}</h4>
      <p class="text-sm text-text-muted">{{ message }}</p>

      <!-- Action button -->
      <button
        v-if="actionLabel"
        type="button"
        class="mt-2 text-sm font-medium text-primary-600 hover:text-primary-700 transition-colors"
        @click="handleAction"
      >
        {{ actionLabel }}
      </button>
    </div>

    <!-- Close button -->
    <button
      v-if="closable"
      type="button"
      class="shrink-0 p-1 rounded hover:bg-surface-sunken transition-colors text-text-muted hover:text-text"
      aria-label="Close notification"
      @click="handleClose"
    >
      <X :size="16" />
    </button>
  </div>
</template>
