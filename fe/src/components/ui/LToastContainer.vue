<script setup lang="ts">
import { ref } from 'vue'
import LToast from './LToast.vue'

export interface Toast {
  id: string
  variant?: 'success' | 'error' | 'warning' | 'info'
  title?: string
  message: string
  duration?: number
  closable?: boolean
  actionLabel?: string
  onAction?: () => void
}

type Position = 'top-right' | 'top-center' | 'top-left' | 'bottom-right' | 'bottom-center' | 'bottom-left'

interface Props {
  position?: Position
}

const props = withDefaults(defineProps<Props>(), {
  position: 'top-right',
})

const toasts = ref<Toast[]>([])

const positionClasses: Record<Position, string> = {
  'top-right': 'top-4 right-4',
  'top-center': 'top-4 left-1/2 -translate-x-1/2',
  'top-left': 'top-4 left-4',
  'bottom-right': 'bottom-4 right-4',
  'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2',
  'bottom-left': 'bottom-4 left-4',
}

function addToast(toast: Omit<Toast, 'id'>) {
  const id = Math.random().toString(36).substring(7)
  toasts.value.push({ ...toast, id })
  return id
}

function removeToast(id: string) {
  const index = toasts.value.findIndex((t) => t.id === id)
  if (index > -1) {
    toasts.value.splice(index, 1)
  }
}

defineExpose({
  addToast,
  removeToast,
})
</script>

<template>
  <Teleport to="body">
    <div :class="['fixed z-50 flex flex-col gap-2 max-w-sm w-full', positionClasses[position]]">
      <TransitionGroup name="toast">
        <LToast
          v-for="toast in toasts"
          :key="toast.id"
          :variant="toast.variant"
          :title="toast.title"
          :message="toast.message"
          :duration="toast.duration"
          :closable="toast.closable"
          :action-label="toast.actionLabel"
          @close="removeToast(toast.id)"
          @action="toast.onAction?.()"
        />
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-move {
  transition: transform 0.3s ease;
}
</style>
