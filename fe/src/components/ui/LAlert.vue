<script setup lang="ts">
import { computed } from 'vue'
import { AlertCircle, CheckCircle, AlertTriangle, Info, X } from 'lucide-vue-next'

interface Props {
  variant?: 'success' | 'warning' | 'error' | 'info'
  title?: string
  dismissible?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'info',
  dismissible: false,
})

const emit = defineEmits<{
  dismiss: []
}>()

const icons = {
  success: CheckCircle,
  warning: AlertTriangle,
  error: AlertCircle,
  info: Info,
}

const variantClasses = {
  success: 'bg-success-muted border-success/30 text-success',
  warning: 'bg-warning-muted border-warning/30 text-warning',
  error: 'bg-error-muted border-error/30 text-error',
  info: 'bg-info-muted border-info/30 text-info',
}

const icon = computed(() => icons[props.variant])
const classes = computed(() => ['flex gap-3 p-4 rounded-lg border', variantClasses[props.variant]])
</script>

<template>
  <div :class="classes" role="alert">
    <component :is="icon" class="h-5 w-5 shrink-0 mt-0.5" />
    <div class="flex-1 min-w-0">
      <h3 v-if="title" class="font-medium mb-1">{{ title }}</h3>
      <div class="text-sm opacity-90">
        <slot />
      </div>
    </div>
    <button
      v-if="dismissible"
      type="button"
      class="shrink-0 p-1 rounded hover:bg-black/10 transition-colors"
      @click="emit('dismiss')"
    >
      <X class="h-4 w-4" />
    </button>
  </div>
</template>
