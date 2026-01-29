<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LMinimap } from '@/components/ui'
import { BrainCircuit, StickyNote, Grid2X2Check, Grid2X2X, PanelRight, PanelBottom } from 'lucide-vue-next'
import CanvasNodeCard from './CanvasNodeCard.vue'
import ConnectModeHint from './ConnectModeHint.vue'
import type { CanvasEdge, EdgeRelationship } from '@/types/canvas'
import type { Datasource } from '@/types/api'

interface Props {
  showGrid: boolean
  selectedId: string | null
  datasources: Datasource[]
  splitDirection?: 'vertical' | 'horizontal'
}

const props = withDefaults(defineProps<Props>(), {
  splitDirection: 'vertical',
})

const emit = defineEmits<{
  'update:selectedId': [id: string | null]
  'update:showGrid': [value: boolean]
  'update:splitDirection': [value: 'vertical' | 'horizontal']
  'edit-edge': [edge: CanvasEdge]
  'add-query': []
  'add-note': []
}>()

const canvasStore = useCanvasStore()

// Camera (viewport offset and zoom) - infinite canvas, no bounds
const containerRef = ref<HTMLElement | null>(null)
const camera = reactive({ x: 0, y: 0, zoom: 1 })
const viewportSize = reactive({ width: 800, height: 600 })

// Zoom constraints
const MIN_ZOOM = 0.1
const MAX_ZOOM = 3

// Transform style for canvas content (includes zoom)
const cameraTransform = computed(
  () => `scale(${camera.zoom}) translate(${-camera.x}px, ${-camera.y}px)`,
)

// Grid background (scales and moves with camera)
const gridSize = computed(() => 24 * camera.zoom)
const gridOffset = computed(
  () =>
    `${(-camera.x * camera.zoom) % gridSize.value}px ${(-camera.y * camera.zoom) % gridSize.value}px`,
)
const gridBgSize = computed(() => `${gridSize.value}px ${gridSize.value}px`)

function updateViewportSize() {
  if (!containerRef.value) return
  viewportSize.width = containerRef.value.clientWidth
  viewportSize.height = containerRef.value.clientHeight
}

// Compute bounds from nodes for minimap (dynamic based on content)
const contentBounds = computed(() => {
  const nodes = canvasStore.nodes
  if (nodes.length === 0) {
    // Default bounds when no nodes
    return { minX: 0, minY: 0, maxX: 2000, maxY: 1500 }
  }

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity

  for (const n of nodes) {
    minX = Math.min(minX, n.x)
    minY = Math.min(minY, n.y)
    maxX = Math.max(maxX, n.x + n.w)
    maxY = Math.max(maxY, n.y + n.h)
  }

  // Add padding around content
  const padding = 500
  return {
    minX: minX - padding,
    minY: minY - padding,
    maxX: maxX + padding,
    maxY: maxY + padding,
  }
})

// Canvas size for minimap (based on content bounds + viewport)
const minimapBounds = computed(() => {
  const bounds = contentBounds.value
  const viewW = viewportSize.width / camera.zoom
  const viewH = viewportSize.height / camera.zoom

  // Include both content bounds and current viewport
  const minX = Math.min(bounds.minX, camera.x)
  const minY = Math.min(bounds.minY, camera.y)
  const maxX = Math.max(bounds.maxX, camera.x + viewW)
  const maxY = Math.max(bounds.maxY, camera.y + viewH)

  return {
    w: maxX - minX,
    h: maxY - minY,
    offsetX: minX,
    offsetY: minY,
  }
})

function handleMinimapNavigate(pos: { x: number; y: number }) {
  // Adjust for minimap offset
  camera.x = pos.x + minimapBounds.value.offsetX
  camera.y = pos.y + minimapBounds.value.offsetY
}

// Minimap node data with status (adjusted for minimap coordinate space)
const minimapNodes = computed(() =>
  canvasStore.nodes.map((n) => ({
    id: n.id,
    x: n.x - minimapBounds.value.offsetX,
    y: n.y - minimapBounds.value.offsetY,
    w: n.w,
    h: n.h,
    type: n.type,
    status: n.meta.status,
  })),
)

// Minimap edges adjusted for coordinate space
const minimapEdges = computed(() =>
  canvasStore.edges.map((e) => ({
    ...e,
    // Edges reference node IDs, minimap will compute positions from nodes
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

// Zoom with mouse wheel
function onWheel(e: WheelEvent) {
  e.preventDefault()

  const rect = containerRef.value?.getBoundingClientRect()
  if (!rect) return

  // Mouse position relative to container
  const mouseX = e.clientX - rect.left
  const mouseY = e.clientY - rect.top

  // Current mouse position in canvas coordinates (before zoom)
  const canvasX = camera.x + mouseX / camera.zoom
  const canvasY = camera.y + mouseY / camera.zoom

  // Apply zoom
  const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1
  const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, camera.zoom * zoomFactor))

  // Adjust camera to keep mouse position stable
  camera.x = canvasX - mouseX / newZoom
  camera.y = canvasY - mouseY / newZoom
  camera.zoom = newZoom
}

// Use ResizeObserver to detect container size changes (from split toggle, panel resize, etc.)
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  updateViewportSize()

  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      updateViewportSize()
    })
    resizeObserver.observe(containerRef.value)
  }

  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
})

onUnmounted(() => {
  resizeObserver?.disconnect()
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

  // Don't clear selection if click originated from within a node
  // (the node's pointerdown handler will handle selection)
  const target = e.target as HTMLElement
  if (target.closest('[data-canvas-node]')) {
    return
  }

  if (!connectMode.active) {
    emit('update:selectedId', null)
  }
}

function onCanvasPointerMove(e: PointerEvent) {
  // Handle panning (camera movement) - no bounds for infinite canvas
  if (pan.active) {
    const dx = e.clientX - pan.startX
    const dy = e.clientY - pan.startY
    camera.x = pan.cameraX - dx / camera.zoom
    camera.y = pan.cameraY - dy / camera.zoom
    return
  }

  // Handle node dragging - account for zoom, no bounds
  if (!drag.active || !drag.id) return

  const dx = (e.clientX - drag.startX) / camera.zoom
  const dy = (e.clientY - drag.startY) / camera.zoom

  canvasStore.updateNode(drag.id, {
    x: drag.nodeX + dx,
    y: drag.nodeY + dy,
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

// Center camera on a specific node (no bounds for infinite canvas)
function centerOnNode(nodeId: string) {
  const node = canvasStore.nodes.find((n) => n.id === nodeId)
  if (!node) return

  // Center the node in the viewport
  camera.x = node.x + node.w / 2 - viewportSize.width / (2 * camera.zoom)
  camera.y = node.y + node.h / 2 - viewportSize.height / (2 * camera.zoom)
}

// Reset zoom to 100%
function resetZoom() {
  camera.zoom = 1
}

// Zoom to fit all content
function zoomToFit() {
  const bounds = contentBounds.value
  if (bounds.maxX <= bounds.minX || bounds.maxY <= bounds.minY) return

  const contentW = bounds.maxX - bounds.minX
  const contentH = bounds.maxY - bounds.minY

  // Calculate zoom to fit content with some padding
  const padding = 100
  const zoomX = (viewportSize.width - padding * 2) / contentW
  const zoomY = (viewportSize.height - padding * 2) / contentH
  const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, Math.min(zoomX, zoomY)))

  // Center on content
  const centerX = (bounds.minX + bounds.maxX) / 2
  const centerY = (bounds.minY + bounds.maxY) / 2

  camera.zoom = newZoom
  camera.x = centerX - viewportSize.width / (2 * newZoom)
  camera.y = centerY - viewportSize.height / (2 * newZoom)
}

// Expose methods for parent
defineExpose({
  connectMode,
  startConnect,
  cancelConnect,
  centerOnNode,
  resetZoom,
  zoomToFit,
  camera,
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
      'select-none': drag.active || pan.active,
    }"
    :style="
      props.showGrid ? { backgroundPosition: gridOffset, backgroundSize: gridBgSize } : undefined
    "
    @pointerdown="onCanvasPointerDown"
    @pointermove="onCanvasPointerMove"
    @pointerup="onCanvasPointerUp"
    @pointerleave="onCanvasPointerUp"
    @wheel.prevent="onWheel"
  >
    <!-- Camera-transformed canvas content -->
    <div class="absolute origin-top-left" :style="{ transform: cameraTransform }">
      <!-- Edges SVG - large viewBox to cover infinite space -->
      <svg
        class="absolute pointer-events-none overflow-visible"
        style="left: -10000px; top: -10000px; width: 20000px; height: 20000px"
        viewBox="-10000 -10000 20000 20000"
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

    <!-- Floating toolbar -->
    <div
      class="absolute top-4 left-4 z-10 flex items-center gap-1 p-1.5 rounded-lg bg-surface-overlay/90 backdrop-blur-sm border border-border shadow-lg"
    >
      <!-- Add node buttons -->
      <button
        type="button"
        class="p-2 rounded-md text-text hover:bg-surface-sunken transition-colors"
        title="Add Query Node"
        @click="$emit('add-query')"
      >
        <BrainCircuit :size="16" />
      </button>
      <button
        type="button"
        class="p-2 rounded-md text-text hover:bg-surface-sunken transition-colors"
        title="Add Note"
        @click="$emit('add-note')"
      >
        <StickyNote :size="16" />
      </button>

      <div class="w-px h-5 bg-border mx-1" />

      <!-- Grid toggle -->
      <button
        type="button"
        class="p-2 rounded-md text-text hover:bg-surface-sunken transition-colors"
        :title="props.showGrid ? 'Hide Grid' : 'Show Grid'"
        @click="$emit('update:showGrid', !props.showGrid)"
      >
        <Grid2X2Check v-if="props.showGrid" :size="16" />
        <Grid2X2X v-else :size="16" />
      </button>

      <!-- Split direction toggle -->
      <button
        type="button"
        class="p-2 rounded-md text-text hover:bg-surface-sunken transition-colors"
        :title="props.splitDirection === 'vertical' ? 'Switch to Horizontal Split' : 'Switch to Vertical Split'"
        @click="$emit('update:splitDirection', props.splitDirection === 'vertical' ? 'horizontal' : 'vertical')"
      >
        <PanelRight v-if="props.splitDirection === 'vertical'" :size="16" />
        <PanelBottom v-else :size="16" />
      </button>

      <div class="w-px h-5 bg-border mx-1" />

      <!-- Zoom indicator -->
      <span class="px-2 text-xs text-text-muted tabular-nums min-w-12 text-center">
        {{ Math.round(camera.zoom * 100) }}%
      </span>
    </div>

    <!-- Connect mode hint (outside camera transform, stays in viewport) -->
    <ConnectModeHint
      v-if="connectMode.active"
      class="z-20"
      :from-id="connectMode.from"
      :label="connectMode.label"
      @update:label="updateConnectLabel"
      @cancel="cancelConnect"
    />

    <!-- Minimap (fixed position, outside camera transform) -->
    <div class="absolute bottom-4 right-4 z-20 shadow-lg">
      <LMinimap
        :nodes="minimapNodes"
        :edges="minimapEdges"
        :canvas-width="minimapBounds.w"
        :canvas-height="minimapBounds.h"
        :viewport-x="camera.x - minimapBounds.offsetX"
        :viewport-y="camera.y - minimapBounds.offsetY"
        :viewport-width="viewportSize.width / camera.zoom"
        :viewport-height="viewportSize.height / camera.zoom"
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
}
</style>
