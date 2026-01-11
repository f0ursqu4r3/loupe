<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { X } from 'lucide-vue-next'

interface Props {
  modelValue?: boolean
  /** Alias for modelValue for convenience */
  open?: boolean
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  closable?: boolean
  closeOnOverlay?: boolean
  closeOnEscape?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: undefined,
  open: undefined,
  size: 'md',
  closable: true,
  closeOnOverlay: true,
  closeOnEscape: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  close: []
}>()

// Support both modelValue and open props
const isOpen = ref(props.modelValue ?? props.open ?? false)

watch(
  () => props.modelValue ?? props.open,
  (value) => {
    isOpen.value = value ?? false
  },
)

function close() {
  if (props.closable) {
    isOpen.value = false
    emit('update:modelValue', false)
    emit('close')
  }
}

function onOverlayClick() {
  if (props.closeOnOverlay) {
    close()
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.closeOnEscape && isOpen.value) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})

const sizeClasses = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  full: 'max-w-[calc(100vw-2rem)] max-h-[calc(100vh-2rem)]',
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- Overlay -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="onOverlayClick" />

        <!-- Modal -->
        <div
          :class="[
            'relative w-full bg-surface-overlay rounded-xl shadow-xl border border-border',
            sizeClasses[size],
          ]"
          role="dialog"
          aria-modal="true"
        >
          <!-- Header -->
          <div
            v-if="title || closable"
            class="flex items-center justify-between px-6 py-4 border-b border-border"
          >
            <h2 v-if="title" class="text-lg font-semibold text-text">
              {{ title }}
            </h2>
            <button
              v-if="closable"
              type="button"
              class="p-1 rounded-md text-text-muted hover:text-text hover:bg-surface-sunken transition-colors focus-ring ml-auto"
              @click="close"
            >
              <X class="h-5 w-5" />
            </button>
          </div>

          <!-- Body -->
          <div class="px-6 py-4">
            <slot />
          </div>

          <!-- Footer -->
          <div
            v-if="$slots.footer"
            class="px-6 py-4 border-t border-border bg-surface-sunken rounded-b-xl"
          >
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 200ms ease;
}

.modal-enter-active > div:last-child,
.modal-leave-active > div:last-child {
  transition:
    transform 200ms ease,
    opacity 200ms ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from > div:last-child,
.modal-leave-to > div:last-child {
  transform: scale(0.95);
  opacity: 0;
}
</style>
