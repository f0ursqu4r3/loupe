<script setup lang="ts">
import { computed } from 'vue'
import { LButton, LSelect } from '@/components/ui'
import type { EdgeRelationship } from '@/types/canvas'

interface Props {
  fromId: string | null
  label: EdgeRelationship
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:label': [label: EdgeRelationship]
  cancel: []
}>()

const edgeRelationships: EdgeRelationship[] = [
  'motivates',
  'explains',
  'contradicts',
  'supports',
  'derives_from',
  'questions',
]

const edgeRelationshipOptions = computed(() =>
  edgeRelationships.map((rel) => ({ value: rel, label: rel })),
)

function updateLabel(value: string | number) {
  emit('update:label', value as EdgeRelationship)
}
</script>

<template>
  <div
    class="sticky left-4 top-4 inline-flex gap-2.5 items-center px-3 py-2.5 rounded-xl border border-border bg-surface-overlay/80 backdrop-blur-sm m-4 w-fit"
  >
    Click a target node to connect from <b>{{ props.fromId }}</b>
    <LSelect
      :model-value="props.label"
      :options="edgeRelationshipOptions"
      class="w-36"
      @update:model-value="updateLabel"
    />
    <LButton variant="secondary" size="sm" @click="$emit('cancel')">Cancel</LButton>
  </div>
</template>
