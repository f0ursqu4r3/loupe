<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'

const props = defineProps<{
  x: number
  y: number
  width: number
  height: number
  minWidth?: number
  minHeight?: number
  maxWidth?: number
  maxHeight?: number
  gridCols?: number
  rowHeight?: number
  gap?: number
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:position': [x: number, y: number]
  'update:size': [width: number, height: number]
  change: [x: number, y: number, width: number, height: number]
  dragStart: []
  dragEnd: []
  resizeStart: []
  resizeEnd: []
}>()

const gridCols = computed(() => props.gridCols ?? 12)
const rowHeight = computed(() => props.rowHeight ?? 80)
const gap = computed(() => props.gap ?? 16)
const minWidth = computed(() => props.minWidth ?? 2)
const minHeight = computed(() => props.minHeight ?? 2)
const maxWidth = computed(() => props.maxWidth ?? gridCols.value)
const maxHeight = computed(() => props.maxHeight ?? 20)

// Interaction state
const isDragging = ref(false)
const isResizing = ref(false)
const containerRef = ref<HTMLElement | null>(null)

// Temporary position/size during drag/resize
const tempX = ref(props.x)
const tempY = ref(props.y)
const tempWidth = ref(props.width)
const tempHeight = ref(props.height)

// Drag state
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartPosX = ref(0)
const dragStartPosY = ref(0)

// Resize state
const resizeStartX = ref(0)
const resizeStartY = ref(0)
const resizeStartWidth = ref(0)
const resizeStartHeight = ref(0)
const resizeHandle = ref<string>('')

// Get parent grid element for calculations
function getGridContainer(): HTMLElement | null {
  return containerRef.value?.parentElement ?? null
}

function getCellWidth(): number {
  const container = getGridContainer()
  if (!container) return 100
  const containerWidth = container.clientWidth
  return (containerWidth - gap.value * (gridCols.value - 1)) / gridCols.value
}

// Start dragging
function startDrag(e: MouseEvent | TouchEvent) {
  if (props.disabled) return
  e.preventDefault()

  isDragging.value = true
  emit('dragStart')

  const clientX = 'touches' in e && e.touches[0] ? e.touches[0].clientX : (e as MouseEvent).clientX
  const clientY = 'touches' in e && e.touches[0] ? e.touches[0].clientY : (e as MouseEvent).clientY

  dragStartX.value = clientX
  dragStartY.value = clientY
  dragStartPosX.value = props.x
  dragStartPosY.value = props.y
  tempX.value = props.x
  tempY.value = props.y

  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', endDrag)
  document.addEventListener('touchmove', onDrag, { passive: false })
  document.addEventListener('touchend', endDrag)
}

function onDrag(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return
  e.preventDefault()

  const clientX = 'touches' in e && e.touches[0] ? e.touches[0].clientX : (e as MouseEvent).clientX
  const clientY = 'touches' in e && e.touches[0] ? e.touches[0].clientY : (e as MouseEvent).clientY

  const cellWidth = getCellWidth()
  const deltaX = clientX - dragStartX.value
  const deltaY = clientY - dragStartY.value

  const deltaCols = Math.round(deltaX / (cellWidth + gap.value))
  const deltaRows = Math.round(deltaY / (rowHeight.value + gap.value))

  const newX = Math.max(0, Math.min(gridCols.value - props.width, dragStartPosX.value + deltaCols))
  const newY = Math.max(0, dragStartPosY.value + deltaRows)

  tempX.value = newX
  tempY.value = newY
}

function endDrag() {
  if (!isDragging.value) return

  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', endDrag)
  document.removeEventListener('touchmove', onDrag)
  document.removeEventListener('touchend', endDrag)

  if (tempX.value !== props.x || tempY.value !== props.y) {
    emit('update:position', tempX.value, tempY.value)
    emit('change', tempX.value, tempY.value, props.width, props.height)
  }
  emit('dragEnd')
}

// Start resizing
function startResize(e: MouseEvent | TouchEvent, handle: string) {
  if (props.disabled) return
  e.preventDefault()
  e.stopPropagation()

  isResizing.value = true
  resizeHandle.value = handle
  emit('resizeStart')

  const clientX = 'touches' in e && e.touches[0] ? e.touches[0].clientX : (e as MouseEvent).clientX
  const clientY = 'touches' in e && e.touches[0] ? e.touches[0].clientY : (e as MouseEvent).clientY

  resizeStartX.value = clientX
  resizeStartY.value = clientY
  resizeStartWidth.value = props.width
  resizeStartHeight.value = props.height
  tempWidth.value = props.width
  tempHeight.value = props.height
  tempX.value = props.x
  tempY.value = props.y

  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', endResize)
  document.addEventListener('touchmove', onResize, { passive: false })
  document.addEventListener('touchend', endResize)
}

function onResize(e: MouseEvent | TouchEvent) {
  if (!isResizing.value) return
  e.preventDefault()

  const clientX = 'touches' in e && e.touches[0] ? e.touches[0].clientX : (e as MouseEvent).clientX
  const clientY = 'touches' in e && e.touches[0] ? e.touches[0].clientY : (e as MouseEvent).clientY

  const cellWidth = getCellWidth()
  const deltaX = clientX - resizeStartX.value
  const deltaY = clientY - resizeStartY.value

  const handle = resizeHandle.value

  // Handle horizontal resize
  if (handle.includes('e')) {
    const deltaCols = Math.round(deltaX / (cellWidth + gap.value))
    const newWidth = Math.max(
      minWidth.value,
      Math.min(maxWidth.value, resizeStartWidth.value + deltaCols),
    )
    // Ensure we don't exceed grid bounds
    tempWidth.value = Math.min(newWidth, gridCols.value - props.x)
  }

  if (handle.includes('w')) {
    const deltaCols = Math.round(deltaX / (cellWidth + gap.value))
    const newWidth = Math.max(
      minWidth.value,
      Math.min(maxWidth.value, resizeStartWidth.value - deltaCols),
    )
    const widthDiff = newWidth - resizeStartWidth.value
    const newX = Math.max(0, props.x - widthDiff)
    tempWidth.value = newWidth
    tempX.value = newX
  }

  // Handle vertical resize
  if (handle.includes('s')) {
    const deltaRows = Math.round(deltaY / (rowHeight.value + gap.value))
    const newHeight = Math.max(
      minHeight.value,
      Math.min(maxHeight.value, resizeStartHeight.value + deltaRows),
    )
    tempHeight.value = newHeight
  }

  if (handle.includes('n')) {
    const deltaRows = Math.round(deltaY / (rowHeight.value + gap.value))
    const newHeight = Math.max(
      minHeight.value,
      Math.min(maxHeight.value, resizeStartHeight.value - deltaRows),
    )
    const heightDiff = newHeight - resizeStartHeight.value
    const newY = Math.max(0, props.y - heightDiff)
    tempHeight.value = newHeight
    tempY.value = newY
  }
}

function endResize() {
  if (!isResizing.value) return

  isResizing.value = false
  resizeHandle.value = ''
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', endResize)
  document.removeEventListener('touchmove', onResize)
  document.removeEventListener('touchend', endResize)

  const changed =
    tempWidth.value !== props.width ||
    tempHeight.value !== props.height ||
    tempX.value !== props.x ||
    tempY.value !== props.y

  if (changed) {
    emit('update:size', tempWidth.value, tempHeight.value)
    emit('change', tempX.value, tempY.value, tempWidth.value, tempHeight.value)
  }
  emit('resizeEnd')
}

// Computed style for the grid item
const itemStyle = computed(() => {
  const x = isDragging.value || isResizing.value ? tempX.value : props.x
  const y = isDragging.value || isResizing.value ? tempY.value : props.y
  const w = isResizing.value ? tempWidth.value : props.width
  const h = isResizing.value ? tempHeight.value : props.height

  return {
    gridColumn: `${x + 1} / span ${w}`,
    gridRow: `${y + 1} / span ${h}`,
    minHeight: `${h * rowHeight.value + (h - 1) * gap.value}px`,
  }
})

// Cleanup on unmount
onUnmounted(() => {
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', endDrag)
  document.removeEventListener('touchmove', onDrag)
  document.removeEventListener('touchend', endDrag)
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', endResize)
  document.removeEventListener('touchmove', onResize)
  document.removeEventListener('touchend', endResize)
})
</script>

<template>
  <div
    ref="containerRef"
    class="relative group"
    :class="{
      'z-50': isDragging || isResizing,
      'transition-all duration-150': !isDragging && !isResizing,
      'cursor-move': isDragging,
    }"
    :style="itemStyle"
  >
    <!-- Drag handle -->
    <div
      v-if="!disabled"
      class="absolute top-2 left-2 z-20 p-1 rounded bg-surface/80 cursor-move opacity-0 group-hover:opacity-100 transition-opacity hover:bg-surface-sunken"
      @mousedown="startDrag"
      @touchstart="startDrag"
    >
      <GripVertical class="h-4 w-4 text-text-muted" />
    </div>

    <!-- Resize handles -->
    <template v-if="!disabled">
      <!-- Right edge -->
      <div
        class="absolute top-0 right-0 w-2 h-full cursor-ew-resize opacity-0 group-hover:opacity-100 hover:bg-primary-500/30 transition-opacity z-20"
        @mousedown="(e) => startResize(e, 'e')"
        @touchstart="(e) => startResize(e, 'e')"
      />
      <!-- Bottom edge -->
      <div
        class="absolute bottom-0 left-0 w-full h-2 cursor-ns-resize opacity-0 group-hover:opacity-100 hover:bg-primary-500/30 transition-opacity z-20"
        @mousedown="(e) => startResize(e, 's')"
        @touchstart="(e) => startResize(e, 's')"
      />
      <!-- Bottom-right corner -->
      <div
        class="absolute bottom-0 right-0 w-4 h-4 cursor-nwse-resize opacity-0 group-hover:opacity-100 z-20"
        @mousedown="(e) => startResize(e, 'se')"
        @touchstart="(e) => startResize(e, 'se')"
      >
        <div
          class="absolute bottom-1 right-1 w-2 h-2 border-r-2 border-b-2 border-primary-500 rounded-br"
        />
      </div>
    </template>

    <!-- Content slot -->
    <slot />

    <!-- Drag/resize overlay -->
    <div
      v-if="isDragging || isResizing"
      class="absolute inset-0 bg-primary-500/10 border-2 border-primary-500 border-dashed rounded-lg pointer-events-none z-10"
    />
  </div>
</template>
