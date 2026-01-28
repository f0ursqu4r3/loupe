<script setup lang="ts">
import { computed, ref } from 'vue'

interface MinimapNode {
  id: string
  x: number
  y: number
  w: number
  h: number
  type?: string
  status?: string
}

interface MinimapEdge {
  id: string
  from: string
  to: string
}

interface Props {
  /** All nodes to display */
  nodes: MinimapNode[]
  /** Edges between nodes */
  edges?: MinimapEdge[]
  /** Total canvas width */
  canvasWidth?: number
  /** Total canvas height */
  canvasHeight?: number
  /** Current viewport scroll position X */
  viewportX?: number
  /** Current viewport scroll position Y */
  viewportY?: number
  /** Viewport visible width */
  viewportWidth?: number
  /** Viewport visible height */
  viewportHeight?: number
  /** Minimap display width */
  width?: number
  /** Minimap display height */
  height?: number
}

const props = withDefaults(defineProps<Props>(), {
  edges: () => [],
  canvasWidth: 3000,
  canvasHeight: 2000,
  viewportX: 0,
  viewportY: 0,
  viewportWidth: 800,
  viewportHeight: 600,
  width: 200,
  height: 140,
})

const emit = defineEmits<{
  'navigate': [position: { x: number; y: number }]
}>()

const minimapRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)

// Calculate scale factor to fit canvas into minimap
const scale = computed(() => {
  const scaleX = props.width / props.canvasWidth
  const scaleY = props.height / props.canvasHeight
  return Math.min(scaleX, scaleY)
})

// Viewport rectangle in minimap coordinates
const viewportRect = computed(() => ({
  x: props.viewportX * scale.value,
  y: props.viewportY * scale.value,
  width: Math.min(props.viewportWidth, props.canvasWidth) * scale.value,
  height: Math.min(props.viewportHeight, props.canvasHeight) * scale.value,
}))

// Scale a node for minimap display
function scaleNode(node: MinimapNode) {
  return {
    x: node.x * scale.value,
    y: node.y * scale.value,
    width: node.w * scale.value,
    height: node.h * scale.value,
  }
}

// Get node color based on type/status
function getNodeColor(node: MinimapNode): string {
  if (node.status === 'error') return 'var(--loupe-error)'
  if (node.status === 'running') return 'var(--loupe-info)'
  if (node.status === 'ok') return 'var(--loupe-success)'
  if (node.type === 'query') return 'var(--loupe-info)'
  if (node.type === 'note') return 'var(--loupe-warning)'
  return 'var(--loupe-text-muted)'
}

// Calculate edge path in minimap coordinates
function getEdgePath(edge: MinimapEdge): string {
  const fromNode = props.nodes.find((n) => n.id === edge.from)
  const toNode = props.nodes.find((n) => n.id === edge.to)
  if (!fromNode || !toNode) return ''

  const from = scaleNode(fromNode)
  const to = scaleNode(toNode)

  const x1 = from.x + from.width
  const y1 = from.y + from.height / 2
  const x2 = to.x
  const y2 = to.y + to.height / 2

  return `M ${x1} ${y1} L ${x2} ${y2}`
}

// Handle click/drag to navigate
function handlePointerDown(e: PointerEvent) {
  isDragging.value = true
  navigateToPosition(e)
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function handlePointerMove(e: PointerEvent) {
  if (!isDragging.value) return
  navigateToPosition(e)
}

function handlePointerUp() {
  isDragging.value = false
}

function navigateToPosition(e: PointerEvent) {
  const rect = minimapRef.value?.getBoundingClientRect()
  if (!rect) return

  // Calculate click position in minimap coordinates
  const minimapX = e.clientX - rect.left
  const minimapY = e.clientY - rect.top

  // Convert to canvas coordinates, centering the viewport on click point
  const canvasX = minimapX / scale.value - props.viewportWidth / 2
  const canvasY = minimapY / scale.value - props.viewportHeight / 2

  // Clamp to valid range
  const x = Math.max(0, Math.min(canvasX, props.canvasWidth - props.viewportWidth))
  const y = Math.max(0, Math.min(canvasY, props.canvasHeight - props.viewportHeight))

  emit('navigate', { x, y })
}
</script>

<template>
  <div
    ref="minimapRef"
    class="relative rounded-lg border border-border bg-surface-sunken overflow-hidden cursor-crosshair select-none"
    :style="{ width: `${props.width}px`, height: `${props.height}px` }"
    @pointerdown="handlePointerDown"
    @pointermove="handlePointerMove"
    @pointerup="handlePointerUp"
    @pointerleave="handlePointerUp"
  >
    <!-- Edges layer -->
    <svg class="absolute inset-0 w-full h-full pointer-events-none">
      <path
        v-for="edge in props.edges"
        :key="edge.id"
        :d="getEdgePath(edge)"
        class="fill-none stroke-text-subtle"
        stroke-width="1"
        stroke-opacity="0.5"
      />
    </svg>

    <!-- Nodes layer -->
    <div
      v-for="node in props.nodes"
      :key="node.id"
      class="absolute rounded-sm pointer-events-none"
      :style="{
        left: `${scaleNode(node).x}px`,
        top: `${scaleNode(node).y}px`,
        width: `${Math.max(4, scaleNode(node).width)}px`,
        height: `${Math.max(3, scaleNode(node).height)}px`,
        backgroundColor: getNodeColor(node),
        opacity: 0.8,
      }"
    />

    <!-- Viewport indicator -->
    <div
      class="absolute border-2 border-primary-500 bg-primary-500/10 rounded-sm pointer-events-none"
      :style="{
        left: `${viewportRect.x}px`,
        top: `${viewportRect.y}px`,
        width: `${viewportRect.width}px`,
        height: `${viewportRect.height}px`,
      }"
    />
  </div>
</template>
