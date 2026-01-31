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
    <div class="w-20 h-20 rounded-full bg-surface-sunken flex items-center justify-center mb-6">
      <slot name="icon">
        <FileQuestion :size="48" class="text-text-subtle" />
      </slot>
    </div>
    <h3 v-if="title" class="text-xl font-semibold text-text mb-2">
      {{ title }}
    </h3>
    <p v-if="description" class="text-text-muted max-w-md mb-1">
      {{ description }}
    </p>
    <div v-if="$slots.action" class="mt-6">
      <slot name="action" />
    </div>
  </div>
</template>
