<script setup lang="ts">
import { computed } from 'vue'
import LBadge from './LBadge.vue'
import { X } from 'lucide-vue-next'

const props = defineProps<{
  allTags: string[]
  selectedTags: string[]
}>()

const emit = defineEmits<{
  'update:selectedTags': [value: string[]]
}>()

const sortedTags = computed(() => [...props.allTags].sort())

function toggleTag(tag: string) {
  if (props.selectedTags.includes(tag)) {
    emit(
      'update:selectedTags',
      props.selectedTags.filter((t) => t !== tag),
    )
  } else {
    emit('update:selectedTags', [...props.selectedTags, tag])
  }
}

function clearTags() {
  emit('update:selectedTags', [])
}
</script>

<template>
  <div v-if="allTags.length > 0" class="flex flex-wrap items-center gap-1.5">
    <span class="text-xs text-text-muted mr-1">Filter:</span>
    <LBadge
      v-for="tag in sortedTags"
      :key="tag"
      :variant="selectedTags.includes(tag) ? 'primary' : 'default'"
      size="sm"
      class="cursor-pointer hover:opacity-80"
      @click="toggleTag(tag)"
    >
      {{ tag }}
    </LBadge>
    <button
      v-if="selectedTags.length > 0"
      class="text-xs text-text-muted hover:text-text flex items-center gap-0.5 ml-2"
      @click="clearTags"
    >
      <X :size="12" />
      Clear
    </button>
  </div>
</template>
