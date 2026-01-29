<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'
import { FileQuestion } from 'lucide-vue-next'

interface Props {
  title?: string
  description?: string
  class?: HTMLAttributes['class']
}

const props = defineProps<Props>()

const classes = computed(() => [
  'flex flex-col items-center justify-center py-12 px-4 text-center',
  props.class,
])
</script>

<template>
  <div :class="classes">
    <div class="w-16 h-16 rounded-full bg-surface-sunken flex items-center justify-center mb-4">
      <slot name="icon">
        <FileQuestion :size="32" class="text-text-subtle" />
      </slot>
    </div>
    <h3 v-if="title" class="text-lg font-medium text-text mb-1">
      {{ title }}
    </h3>
    <p v-if="description" class="text-sm text-text-muted max-w-sm">
      {{ description }}
    </p>
    <div v-if="$slots.action" class="mt-4">
      <slot name="action" />
    </div>
  </div>
</template>
