<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LMinimap } from '@/components/ui'
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

// Camera (viewport offset)
const containerRef = ref<HTMLElement | null>(null)
const camera = reactive({ x: 0, y: 0 })
const viewportSize = reactive({ width: 800, height: 600 })

// Transform style for canvas content
const cameraTransform = computed(() => `translate(${-camera.x}px, ${-camera.y}px)`)

// Grid background position (moves with camera)
const gridOffset = computed(() => `${-camera.x % 24}px ${-camera.y % 24}px`)

function updateViewportSize() {
  if (!containerRef.value) return
  viewportSize.width = containerRef.value.clientWidth
  viewportSize.height = containerRef.value.clientHeight
}

function handleMinimapNavigate(pos: { x: number; y: number }) {
  camera.x = Math.max(0, Math.min(pos.x, canvasSize.w - viewportSize.width))
  camera.y = Math.max(0, Math.min(pos.y, canvasSize.h - viewportSize.height))
}

// Minimap node data with status
const minimapNodes = computed(() =>
  canvasStore.nodes.map((n) => ({
    id: n.id,
    x: n.x,
    y: n.y,
    w: n.w,
    h: n.h,
    type: n.type,
    status: n.meta.status,
  })),
)

// Space + drag panning
const spacePressed = ref(false)
const pan = reactive({
  active: false,
  startX: 0,
  startY: 0,
  cameraX: 0,
  cameraY: 0,
})

function isEditableElement(el: EventTarget | null): boolean {
  if (!el || !(el instanceof HTMLElement)) return false
  const tagName = el.tagName.toLowerCase()
  return (
    tagName === 'input' ||
    tagName === 'textarea' ||
    el.isContentEditable ||
    el.closest('.monaco-editor') !== null
  )
}

function onKeyDown(e: KeyboardEvent) {
  // Don't capture space when typing in inputs/editors
  if (isEditableElement(e.target)) return

  if (e.code === 'Space' && !e.repeat) {
    spacePressed.value = true
    e.preventDefault()
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.code === 'Space') {
    spacePressed.value = false
    pan.active = false
  }
}

onMounted(() => {
  updateViewportSize()
  window.addEventListener('resize', updateViewportSize)
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateViewportSize)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
})

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

// Dragging nodes
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
  // If space is pressed, start panning instead of node drag
  if (spacePressed.value) {
    startPan(e)
    return
  }

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

function startPan(e: PointerEvent) {
  pan.active = true
  pan.startX = e.clientX
  pan.startY = e.clientY
  pan.cameraX = camera.x
  pan.cameraY = camera.y
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onCanvasPointerDown(e: PointerEvent) {
  // Start panning if space is pressed
  if (spacePressed.value) {
    startPan(e)
    return
  }

  if (!connectMode.active) {
    emit('update:selectedId', null)
  }
}

function onCanvasPointerMove(e: PointerEvent) {
  // Handle panning (camera movement)
  if (pan.active) {
    const dx = e.clientX - pan.startX
    const dy = e.clientY - pan.startY
    camera.x = Math.max(0, Math.min(pan.cameraX - dx, canvasSize.w - viewportSize.width))
    camera.y = Math.max(0, Math.min(pan.cameraY - dy, canvasSize.h - viewportSize.height))
    return
  }

  // Handle node dragging
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
  pan.active = false
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

// Center camera on a specific node
function centerOnNode(nodeId: string) {
  const node = canvasStore.nodes.find((n) => n.id === nodeId)
  if (!node) return

  // Center the node in the viewport
  const targetX = node.x + node.w / 2 - viewportSize.width / 2
  const targetY = node.y + node.h / 2 - viewportSize.height / 2

  // Clamp to valid camera bounds
  camera.x = Math.max(0, Math.min(targetX, canvasSize.w - viewportSize.width))
  camera.y = Math.max(0, Math.min(targetY, canvasSize.h - viewportSize.height))
}

// Expose methods for parent
defineExpose({
  connectMode,
  startConnect,
  cancelConnect,
  centerOnNode,
})
</script>

<template>
  <section
    ref="containerRef"
    class="relative overflow-hidden min-h-0 bg-surface-sunken"
    :class="{
      'canvas-grid': props.showGrid,
      'cursor-grab': spacePressed && !pan.active,
      'cursor-grabbing': pan.active,
    }"
    :style="props.showGrid ? { backgroundPosition: gridOffset } : undefined"
    @pointerdown="onCanvasPointerDown"
    @pointermove="onCanvasPointerMove"
    @pointerup="onCanvasPointerUp"
    @pointerleave="onCanvasPointerUp"
  >
    <!-- Camera-transformed canvas content -->
    <div
      class="absolute"
      :style="{
        width: `${canvasSize.w}px`,
        height: `${canvasSize.h}px`,
        transform: cameraTransform,
      }"
    >
      <!-- Edges SVG -->
      <svg
        class="absolute inset-0 pointer-events-none"
        :style="{ width: `${canvasSize.w}px`, height: `${canvasSize.h}px` }"
        :viewBox="`0 0 ${canvasSize.w} ${canvasSize.h}`"
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
    </div>

    <!-- Connect mode hint (outside camera transform, stays in viewport) -->
    <ConnectModeHint
      v-if="connectMode.active"
      :from-id="connectMode.from"
      :label="connectMode.label"
      @update:label="updateConnectLabel"
      @cancel="cancelConnect"
    />

    <!-- Minimap (fixed position, outside camera transform) -->
    <div class="absolute bottom-4 right-4 z-10 shadow-lg">
      <LMinimap
        :nodes="minimapNodes"
        :edges="canvasStore.edges"
        :canvas-width="canvasSize.w"
        :canvas-height="canvasSize.h"
        :viewport-x="camera.x"
        :viewport-y="camera.y"
        :viewport-width="viewportSize.width"
        :viewport-height="viewportSize.height"
        @navigate="handleMinimapNavigate"
      />
    </div>
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
