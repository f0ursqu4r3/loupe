<script setup lang="ts">
import { LModal } from '@/components/ui'
import { Command, Option } from 'lucide-vue-next'

interface Props {
  open?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  close: []
}>()

const shortcuts = [
  {
    category: 'Navigation',
    items: [
      { keys: ['G', 'D'], description: 'Go to Dashboards' },
      { keys: ['G', 'Q'], description: 'Go to Queries' },
      { keys: ['G', 'V'], description: 'Go to Visualizations' },
      { keys: ['G', 'S'], description: 'Go to Settings' },
      { keys: ['G', 'C'], description: 'Go to Canvases' },
    ],
  },
  {
    category: 'Actions',
    items: [
      { keys: ['N'], description: 'Create new (context-dependent)' },
      { keys: ['S'], description: 'Save current item' },
      { keys: ['?'], description: 'Show keyboard shortcuts' },
    ],
  },
  {
    category: 'Accessibility',
    items: [
      { keys: ['Tab'], description: 'Navigate between interactive elements' },
      { keys: ['Shift', 'Tab'], description: 'Navigate backwards' },
      { keys: ['Enter'], description: 'Activate focused element' },
      { keys: ['Escape'], description: 'Close modal or cancel action' },
      { keys: ['Space'], description: 'Pan canvas (hold and drag)' },
    ],
  },
  {
    category: 'General',
    items: [
      { keys: ['Ctrl', 'K'], description: 'Quick search (coming soon)', disabled: true },
      { keys: ['Ctrl', 'B'], description: 'Toggle sidebar' },
      { keys: ['Ctrl', '/'], description: 'Toggle theme' },
    ],
  },
]

// Detect platform for modifier key display
const isMac =
  typeof navigator !== 'undefined' && /Mac|iPhone|iPad|iPod/.test(navigator.userAgent)
const modifierKey = isMac ? '⌘' : 'Ctrl'
</script>

<template>
  <LModal :open="open" title="Keyboard Shortcuts" size="lg" @close="emit('close')">
    <div class="space-y-6">
      <p class="text-text-muted text-sm">
        Use these keyboard shortcuts to navigate and interact with Loupe more efficiently.
      </p>

      <div v-for="section in shortcuts" :key="section.category" class="space-y-3">
        <h3 class="text-lg font-semibold text-text">{{ section.category }}</h3>
        <div class="space-y-2">
          <div
            v-for="(shortcut, index) in section.items"
            :key="index"
            class="flex items-center justify-between py-2 px-3 rounded-md bg-surface-sunken"
            :class="{ 'opacity-50': shortcut.disabled }"
          >
            <span class="text-sm text-text">
              {{ shortcut.description }}
              <span v-if="shortcut.disabled" class="text-text-muted ml-2">(Coming soon)</span>
            </span>
            <div class="flex items-center gap-1">
              <kbd
                v-for="(key, keyIndex) in shortcut.keys"
                :key="keyIndex"
                class="px-2 py-1 text-xs font-semibold text-text bg-surface border border-border rounded shadow-sm"
              >
                {{ key.replace('Ctrl', modifierKey) }}
              </kbd>
            </div>
          </div>
        </div>
      </div>

      <div class="pt-4 border-t border-border">
        <div class="flex items-start gap-3 p-3 rounded-lg bg-info-muted">
          <div class="flex items-center gap-2 text-info shrink-0">
            <Component :is="isMac ? Command : Option" :size="20" />
          </div>
          <div class="text-sm text-text">
            <p class="font-medium mb-1">Tip</p>
            <p class="text-text-muted">
              Press <kbd class="px-1.5 py-0.5 text-xs bg-surface border border-border rounded">?</kbd>
              at any time to view this keyboard shortcuts reference.
            </p>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <LButton @click="emit('close')">Close</LButton>
    </template>
  </LModal>
</template>
