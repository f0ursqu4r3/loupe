<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import CanvasNodeCard from './CanvasNodeCard.vue'
import ConnectModeHint from './ConnectModeHint.vue'
import type { CanvasEdge, EdgeRelationship } from '@/types/canvas'
import type { Datasource } from '@/types/api'

interface Props {
  showGrid: boolean
  selectedId: string | null
  datasources: Datasource[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:selectedId': [id: string | null]
  'edit-edge': [edge: CanvasEdge]
}>()

const canvasStore = useCanvasStore()
const canvasSize = reactive({ w: 3000, h: 2000 })

// Connect mode
const connectMode = reactive<{
  active: boolean
  from: string | null
  label: EdgeRelationship
}>({
  active: false,
  from: null,
  label: 'motivates',
})

function startConnect(id: string) {
  connectMode.active = true
  connectMode.from = id
  connectMode.label = 'motivates'
}

function cancelConnect() {
  connectMode.active = false
  connectMode.from = null
}

function updateConnectLabel(label: EdgeRelationship) {
  connectMode.label = label
}

// Time range label
const timeRangeLabel = computed(() => {
  const preset = canvasStore.activeCanvas?.timeRange.preset ?? '7d'
  const live = canvasStore.activeCanvas?.live ? ' (live)' : ''
  return `Last ${preset}${live}`
})

// Datasource name lookup
function getDatasourceName(id?: string): string {
  if (!id) return '(none)'
  const ds = props.datasources.find((d) => d.id === id)
  return ds?.name ?? '(unknown)'
}

// Node selection
function selectNode(id: string) {
  // Handle connect mode
  if (connectMode.active && connectMode.from && connectMode.from !== id) {
    canvasStore.addEdge(connectMode.from, id, connectMode.label)
    cancelConnect()
  }
  emit('update:selectedId', id)
}

function duplicateNode(id: string) {
  const copy = canvasStore.duplicateNode(id)
  if (copy) {
    emit('update:selectedId', copy.id)
  }
}

function deleteNode(id: string) {
  canvasStore.deleteNode(id)
  if (props.selectedId === id) {
    emit('update:selectedId', null)
  }
  if (connectMode.from === id) {
    cancelConnect()
  }
}

// Dragging
const drag = reactive<{
  active: boolean
  id: string | null
  startX: number
  startY: number
  nodeX: number
  nodeY: number
}>({
  active: false,
  id: null,
  startX: 0,
  startY: 0,
  nodeX: 0,
  nodeY: 0,
})

function onNodePointerDown(e: PointerEvent, id: string) {
  const n = canvasStore.nodes.find((x) => x.id === id)
  if (!n) return

  selectNode(id)

  drag.active = true
  drag.id = id
  drag.startX = e.clientX
  drag.startY = e.clientY
  drag.nodeX = n.x
  drag.nodeY = n.y
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onCanvasPointerDown() {
  if (!connectMode.active) {
    emit('update:selectedId', null)
  }
}

function onCanvasPointerMove(e: PointerEvent) {
  if (!drag.active || !drag.id) return

  const dx = e.clientX - drag.startX
  const dy = e.clientY - drag.startY

  canvasStore.updateNode(drag.id, {
    x: Math.max(0, drag.nodeX + dx),
    y: Math.max(0, drag.nodeY + dy),
  })
}

function onCanvasPointerUp() {
  drag.active = false
  drag.id = null
}

// Edge paths
function edgePath(fromId: string, toId: string): string {
  const a = canvasStore.nodes.find((n) => n.id === fromId)
  const b = canvasStore.nodes.find((n) => n.id === toId)
  if (!a || !b) return ''

  const x1 = a.x + a.w
  const y1 = a.y + a.h / 2
  const x2 = b.x
  const y2 = b.y + b.h / 2
  const dx = Math.max(60, (x2 - x1) / 2)

  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
}

function edgeLabelPos(fromId: string, toId: string): { x: number; y: number } {
  const a = canvasStore.nodes.find((n) => n.id === fromId)
  const b = canvasStore.nodes.find((n) => n.id === toId)
  if (!a || !b) return { x: 0, y: 0 }

  const x1 = a.x + a.w
  const y1 = a.y + a.h / 2
  const x2 = b.x
  const y2 = b.y + b.h / 2

  return {
    x: (x1 + x2) / 2,
    y: (y1 + y2) / 2 - 8,
  }
}

// Expose connect mode for parent
defineExpose({
  connectMode,
  startConnect,
  cancelConnect,
})
</script>

<template>
  <section
    class="relative overflow-auto min-h-0 bg-surface-sunken"
    :class="{ 'canvas-grid': props.showGrid }"
    @pointerdown="onCanvasPointerDown"
    @pointermove="onCanvasPointerMove"
    @pointerup="onCanvasPointerUp"
    @pointerleave="onCanvasPointerUp"
  >
    <!-- Edges SVG -->
    <svg
      class="absolute inset-0 w-750 h-500 pointer-events-none"
      :viewBox="`0 0 ${canvasSize.w} ${canvasSize.h}`"
      preserveAspectRatio="none"
    >
      <defs>
        <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="5" orient="auto">
          <path d="M0,0 L10,5 L0,10 Z" class="fill-text-subtle" />
        </marker>
      </defs>

      <g v-for="e in canvasStore.edges" :key="e.id">
        <path
          class="fill-none stroke-text-subtle stroke-2"
          :d="edgePath(e.from, e.to)"
          marker-end="url(#arrow)"
        />
        <text
          class="fill-text-muted text-[11px] text-center pointer-events-auto cursor-pointer hover:fill-text"
          text-anchor="middle"
          :x="edgeLabelPos(e.from, e.to).x"
          :y="edgeLabelPos(e.from, e.to).y"
          @click.stop="$emit('edit-edge', e)"
        >
          {{ e.label }}
        </text>
      </g>
    </svg>

    <!-- Nodes -->
    <CanvasNodeCard
      v-for="n in canvasStore.nodes"
      :key="n.id"
      :node="n"
      :selected="n.id === props.selectedId"
      :time-range-label="timeRangeLabel"
      :datasource-name="getDatasourceName(n.meta.datasourceId)"
      @pointerdown.stop="onNodePointerDown($event, n.id)"
      @dblclick.stop="selectNode(n.id)"
      @connect="startConnect"
      @duplicate="duplicateNode"
      @delete="deleteNode"
    />

    <!-- Connect mode hint -->
    <ConnectModeHint
      v-if="connectMode.active"
      :from-id="connectMode.from"
      :label="connectMode.label"
      @update:label="updateConnectLabel"
      @cancel="cancelConnect"
    />
  </section>
</template>

<style scoped>
.canvas-grid {
  background-image:
    linear-gradient(var(--loupe-border-muted) 1px, transparent 1px),
    linear-gradient(90deg, var(--loupe-border-muted) 1px, transparent 1px);
  background-size: 24px 24px;
}
</style>
