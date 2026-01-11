<script setup lang="ts">
import { ref, computed } from 'vue'
import { X, Plus } from 'lucide-vue-next'
import LBadge from './LBadge.vue'
import LButton from './LButton.vue'

const props = defineProps<{
  modelValue: string[]
  placeholder?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const newTag = ref('')

const tags = computed({
  get: () => props.modelValue || [],
  set: (value) => emit('update:modelValue', value),
})

function addTag() {
  const tag = newTag.value.trim().toLowerCase()
  if (tag && !tags.value.includes(tag)) {
    emit('update:modelValue', [...tags.value, tag])
    newTag.value = ''
  }
}

function removeTag(tagToRemove: string) {
  emit(
    'update:modelValue',
    tags.value.filter((t) => t !== tagToRemove),
  )
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    addTag()
  } else if (e.key === 'Backspace' && !newTag.value && tags.value.length > 0) {
    const lastTag = tags.value[tags.value.length - 1]
    if (lastTag) {
      removeTag(lastTag)
    }
  }
}
</script>

<template>
  <div
    class="flex flex-wrap items-center gap-1.5 min-h-9 p-1.5 border border-border rounded-lg bg-surface"
  >
    <LBadge v-for="tag in tags" :key="tag" size="sm" class="gap-1">
      {{ tag }}
      <button
        v-if="!disabled"
        type="button"
        class="ml-0.5 hover:text-error"
        @click.stop="removeTag(tag)"
      >
        <X class="h-3 w-3" />
      </button>
    </LBadge>
    <div v-if="!disabled" class="flex items-center gap-1 flex-1 min-w-20">
      <input
        v-model="newTag"
        :placeholder="placeholder || 'Add tag...'"
        class="flex-1 bg-transparent border-0 outline-none text-sm text-text placeholder:text-text-muted"
        @keydown="handleKeydown"
      />
      <LButton
        v-if="newTag.trim()"
        type="button"
        variant="ghost"
        size="sm"
        class="h-6 w-6 p-0"
        @click="addTag"
      >
        <Plus class="h-3 w-3" />
      </LButton>
    </div>
  </div>
</template>
