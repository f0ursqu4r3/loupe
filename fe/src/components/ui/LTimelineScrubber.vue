<script setup lang="ts">
import { computed, ref, type HTMLAttributes } from 'vue'

interface Marker {
  position: number
  label?: string
  color?: 'primary' | 'success' | 'warning' | 'error' | 'info'
}

interface Props {
  /** Current position value (v-model) */
  modelValue?: number
  /** Minimum value */
  min?: number
  /** Maximum value */
  max?: number
  /** Step increment */
  step?: number
  /** Show tick marks at step intervals */
  showTicks?: boolean
  /** Markers/keyframes to display on the timeline */
  markers?: Marker[]
  /** Whether to show labels on markers */
  markerLabels?: boolean
  /** Whether the scrubber is disabled */
  disabled?: boolean
  /** Size variant */
  size?: 'sm' | 'md' | 'lg'
  /** Show current value tooltip while dragging */
  showTooltip?: boolean
  /** Format function for the tooltip value */
  formatValue?: (value: number) => string
  /** Snap to nearest marker when dragging */
  markerSnap?: boolean
  /** Additional CSS classes */
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: 0,
  min: 0,
  max: 100,
  step: 1,
  showTicks: false,
  markers: () => [],
  markerLabels: false,
  disabled: false,
  size: 'md',
  showTooltip: true,
  formatValue: (v: number) => v.toFixed(0),
  markerSnap: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
  scrubStart: []
  scrubEnd: [value: number]
  markerClick: [marker: Marker]
}>()

const trackRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const isHovering = ref(false)
const dragValue = ref(0) // Raw value during drag (before snapping)

// Calculate the percentage position of the playhead (thumb)
const thumbPercent = computed(() => {
  const range = props.max - props.min
  if (range === 0) return 0
  // During drag, show thumb at raw drag position
  const value = isDragging.value ? dragValue.value : props.modelValue
  return ((value - props.min) / range) * 100
})

// Calculate the percentage for the filled progress bar (always snapped when markerSnap is on)
const progressPercent = computed(() => {
  const range = props.max - props.min
  if (range === 0) return 0
  // During drag with markerSnap, show progress at snapped position
  if (isDragging.value && props.markerSnap && props.markers.length > 0) {
    const snappedValue = findNearestSnapPoint(dragValue.value)
    return ((snappedValue - props.min) / range) * 100
  }
  return ((props.modelValue - props.min) / range) * 100
})

// Size-based classes
const sizeClasses = {
  sm: {
    track: 'h-1',
    thumb: 'w-3 h-3',
    marker: 'w-1.5 h-1.5',
  },
  md: {
    track: 'h-1.5',
    thumb: 'w-4 h-4',
    marker: 'w-2 h-2',
  },
  lg: {
    track: 'h-2',
    thumb: 'w-5 h-5',
    marker: 'w-2.5 h-2.5',
  },
}

const currentSize = computed(() => sizeClasses[props.size])

// Marker color classes
const markerColors = {
  primary: 'bg-primary-500',
  success: 'bg-success',
  warning: 'bg-warning',
  error: 'bg-error',
  info: 'bg-info',
}

// Convert a position value to percentage
function positionToPercent(position: number): number {
  const range = props.max - props.min
  if (range === 0) return 0
  return ((position - props.min) / range) * 100
}

// Calculate tick positions
const ticks = computed(() => {
  if (!props.showTicks) return []
  const tickArray: number[] = []
  for (let i = props.min; i <= props.max; i += props.step) {
    tickArray.push(positionToPercent(i))
  }
  return tickArray
})

// Get all snap points (markers + min/max endpoints)
const snapPoints = computed(() => {
  const points = [props.min, props.max, ...props.markers.map((m) => m.position)]
  return [...new Set(points)].sort((a, b) => a - b)
})

// Find the nearest snap point to a given value
function findNearestSnapPoint(value: number): number {
  const points = snapPoints.value
  if (points.length === 0) return value

  let nearest = points[0]!
  let minDistance = Math.abs(value - nearest)

  for (const point of points) {
    const distance = Math.abs(value - point)
    if (distance < minDistance) {
      minDistance = distance
      nearest = point
    }
  }

  return nearest
}

// Check if click is on/near the thumb
function isClickOnThumb(e: PointerEvent): boolean {
  const rect = trackRef.value?.getBoundingClientRect()
  if (!rect) return false

  const clickPercent = (e.clientX - rect.left) / rect.width
  const thumbPercent = (props.modelValue - props.min) / (props.max - props.min)

  // Threshold: ~20px equivalent in percentage terms
  const threshold = 20 / rect.width

  return Math.abs(clickPercent - thumbPercent) < threshold
}

// Handle pointer events for scrubbing
function handlePointerDown(e: PointerEvent) {
  if (props.disabled) return
  isDragging.value = true
  dragValue.value = props.modelValue
  emit('scrubStart')

  // Only update position if click is not on the thumb
  if (!isClickOnThumb(e)) {
    updateValueFromEvent(e)
  }

  // Capture pointer on the track element for reliable dragging
  trackRef.value?.setPointerCapture(e.pointerId)
}

// Handle thumb-specific pointer down (no repositioning)
function handleThumbPointerDown(e: PointerEvent) {
  if (props.disabled) return
  isDragging.value = true
  dragValue.value = props.modelValue
  emit('scrubStart')
  // Capture on track for consistent move/up handling
  trackRef.value?.setPointerCapture(e.pointerId)
}

function handlePointerMove(e: PointerEvent) {
  if (!isDragging.value) return
  updateValueFromEvent(e)
}

function handlePointerUp(e: PointerEvent) {
  if (isDragging.value) {
    // Snap to marker on release if enabled
    if (props.markerSnap && props.markers.length > 0) {
      const snappedValue = findNearestSnapPoint(props.modelValue)
      emit('update:modelValue', snappedValue)
    }
    isDragging.value = false
    emit('scrubEnd', props.modelValue)
    trackRef.value?.releasePointerCapture(e.pointerId)
  }
}

function updateValueFromEvent(e: PointerEvent) {
  const rect = trackRef.value?.getBoundingClientRect()
  if (!rect) return

  // Calculate position as percentage
  const x = e.clientX - rect.left
  let percent = x / rect.width

  // Clamp to 0-1
  percent = Math.max(0, Math.min(1, percent))

  // Convert to value
  const range = props.max - props.min
  let value = props.min + percent * range

  // Apply step
  value = Math.round(value / props.step) * props.step

  // Clamp to min/max
  value = Math.max(props.min, Math.min(props.max, value))

  // Store raw drag value for thumb position
  dragValue.value = value

  // Emit the value (snapping happens on release)
  emit('update:modelValue', value)
}

function handleMarkerClick(marker: Marker, e: PointerEvent) {
  e.stopPropagation()
  e.preventDefault()
  if (props.disabled) return
  emit('markerClick', marker)
  emit('update:modelValue', marker.position)
}

// Keyboard navigation
function handleKeyDown(e: KeyboardEvent) {
  if (props.disabled) return

  let newValue = props.modelValue
  const largeStep = props.step * 10

  switch (e.key) {
    case 'ArrowLeft':
    case 'ArrowDown':
      newValue = props.modelValue - props.step
      e.preventDefault()
      break
    case 'ArrowRight':
    case 'ArrowUp':
      newValue = props.modelValue + props.step
      e.preventDefault()
      break
    case 'PageDown':
      newValue = props.modelValue - largeStep
      e.preventDefault()
      break
    case 'PageUp':
      newValue = props.modelValue + largeStep
      e.preventDefault()
      break
    case 'Home':
      newValue = props.min
      e.preventDefault()
      break
    case 'End':
      newValue = props.max
      e.preventDefault()
      break
    default:
      return
  }

  // Clamp and emit
  newValue = Math.max(props.min, Math.min(props.max, newValue))
  emit('update:modelValue', newValue)
}
</script>

<template>
  <div
    :class="[
      'relative select-none',
      disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer',
      props.class,
    ]"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
  >
    <!-- Clickable area - extends beyond the track for easier interaction -->
    <div
      ref="trackRef"
      class="relative py-3"
      style="touch-action: none"
      role="slider"
      tabindex="0"
      :aria-valuemin="min"
      :aria-valuemax="max"
      :aria-valuenow="modelValue"
      :aria-disabled="disabled"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
      @pointercancel="handlePointerUp"
      @keydown="handleKeyDown"
    >
      <!-- Track background -->
      <div
        :class="[
          'relative w-full rounded-full bg-surface-sunken border border-border transition-colors pointer-events-none',
          currentSize.track,
          !disabled && isHovering && 'border-border-strong',
        ]"
      >
        <!-- Filled progress (shows snapped position) -->
        <div
          :class="[
            'absolute inset-y-0 left-0 rounded-full bg-primary-500 pointer-events-none',
            !isDragging && 'transition-all',
          ]"
          :style="{ width: `${progressPercent}%` }"
        />

        <!-- Tick marks -->
        <template v-if="showTicks">
          <div
            v-for="(tick, index) in ticks"
            :key="index"
            class="absolute top-1/2 -translate-y-1/2 w-px h-full bg-border-strong opacity-50 pointer-events-none"
            :style="{ left: `${tick}%` }"
          />
        </template>
      </div>

      <!-- Playhead/Thumb (outside track, higher z-index, has pointer events) -->
      <div
        :class="[
          'absolute top-1/2 -translate-y-1/2 -translate-x-1/2 rounded-full bg-white border-2 border-primary-500 shadow-md z-30',
          currentSize.thumb,
          !disabled && 'cursor-grab',
          (isDragging || isHovering) && !disabled && 'scale-110',
          isDragging && 'shadow-lg cursor-grabbing',
          !isDragging && 'transition-transform',
        ]"
        :style="{ left: `${thumbPercent}%` }"
        @pointerdown.stop="handleThumbPointerDown"
      />

      <!-- Markers (positioned absolutely, clickable) -->
      <template v-for="(marker, index) in markers" :key="`marker-${index}`">
        <!-- Marker dot (clickable) -->
        <div
          :class="[
            'absolute top-1/2 -translate-y-1/2 -translate-x-1/2 rounded-full transition-transform z-10',
            currentSize.marker,
            markerColors[marker.color || 'primary'],
            !disabled && 'hover:scale-150 cursor-pointer',
          ]"
          :style="{ left: `${positionToPercent(marker.position)}%` }"
          @pointerdown.stop="handleMarkerClick(marker, $event)"
        />
        <!-- Marker label -->
        <div
          v-if="markerLabels && marker.label"
          class="absolute -bottom-3 -translate-x-1/2 px-1.5 py-0.5 text-xs text-text whitespace-nowrap pointer-events-none z-10"
          :style="{ left: `${positionToPercent(marker.position)}%` }"
          :title="marker.label"
        >
          {{ marker.label }}
        </div>
      </template>

      <!-- Tooltip -->
      <Transition
        enter-active-class="transition-opacity duration-150"
        leave-active-class="transition-opacity duration-100"
        enter-from-class="opacity-0"
        leave-to-class="opacity-0"
      >
        <div
          v-if="showTooltip && isDragging"
          class="absolute -top-4 -translate-x-1/2 px-2 py-0.5 rounded bg-surface-overlay border border-border shadow-md text-xs text-text whitespace-nowrap pointer-events-none z-20"
          :style="{ left: `${progressPercent}%` }"
        >
          {{
            formatValue(
              markerSnap && markers.length > 0 ? findNearestSnapPoint(dragValue) : dragValue,
            )
          }}
        </div>
      </Transition>
    </div>
  </div>
</template>
