<script setup lang="ts">
import { computed } from 'vue'
import { LButton, LSelect, LModal } from '@/components/ui'
import type { CanvasEdge, EdgeRelationship } from '@/types/canvas'

interface Props {
  edge: CanvasEdge | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:edge': [edge: CanvasEdge | null]
  save: [edge: CanvasEdge]
  delete: [edge: CanvasEdge]
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
  if (props.edge) {
    emit('update:edge', { ...props.edge, label: value as EdgeRelationship })
  }
}

function handleSave() {
  if (props.edge) {
    emit('save', props.edge)
    emit('update:edge', null)
  }
}

function handleDelete() {
  if (props.edge) {
    emit('delete', props.edge)
    emit('update:edge', null)
  }
}

function handleClose() {
  emit('update:edge', null)
}
</script>

<template>
  <LModal :open="!!props.edge" title="Edge Relationship" size="sm" @close="handleClose">
    <LSelect
      v-if="props.edge"
      :model-value="props.edge.label"
      :options="edgeRelationshipOptions"
      @update:model-value="updateLabel"
    />
    <template #footer>
      <LButton variant="danger" @click="handleDelete">Delete Edge</LButton>
      <LButton variant="primary" @click="handleSave">Save</LButton>
    </template>
  </LModal>
</template>
