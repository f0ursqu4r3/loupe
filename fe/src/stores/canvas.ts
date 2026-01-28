import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  CanvasState,
  CanvasNode,
  CanvasEdge,
  CanvasStorage,
  TimePreset,
  CanvasNodeType,
  EdgeRelationship,
} from '@/types/canvas'
import { createDefaultNode, createDefaultEdge } from '@/types/canvas'
import type { Datasource } from '@/types/api'

const STORAGE_KEY = 'loupe-canvas-storage'

function generateCanvasId(): string {
  return `canvas-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`
}

function createEmptyCanvas(name: string = 'Untitled Canvas'): CanvasState {
  const now = new Date().toISOString()
  return {
    id: generateCanvasId(),
    name,
    nodes: [],
    edges: [],
    timeRange: { preset: 'now', offset: 0 },
    live: false,
    createdAt: now,
    updatedAt: now,
  }
}

export const useCanvasStore = defineStore('canvas', () => {
  // State
  const canvases = ref<CanvasState[]>([])
  const activeCanvasId = ref<string | null>(null)

  // Getters
  const activeCanvas = computed(() =>
    canvases.value.find((c) => c.id === activeCanvasId.value) ?? null
  )

  const nodes = computed(() => activeCanvas.value?.nodes ?? [])
  const edges = computed(() => activeCanvas.value?.edges ?? [])

  // Load from localStorage
  function load(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const data: CanvasStorage = JSON.parse(stored)
        canvases.value = data.canvases ?? []
        activeCanvasId.value = data.activeCanvasId

        // Migrate: ensure all canvases have offset in timeRange
        for (const canvas of canvases.value) {
          if (canvas.timeRange.offset === undefined) {
            canvas.timeRange.offset = 0
          }
        }

        // Validate active canvas exists
        if (activeCanvasId.value && !canvases.value.find((c) => c.id === activeCanvasId.value)) {
          activeCanvasId.value = canvases.value[0]?.id ?? null
        }
      }

      // Create default canvas if none exist
      if (canvases.value.length === 0) {
        const defaultCanvas = createEmptyCanvas('My First Canvas')
        canvases.value.push(defaultCanvas)
        activeCanvasId.value = defaultCanvas.id
        save()
      }
    } catch (e) {
      console.error('Failed to load canvas state:', e)
      // Reset to default state
      const defaultCanvas = createEmptyCanvas('My First Canvas')
      canvases.value = [defaultCanvas]
      activeCanvasId.value = defaultCanvas.id
      save()
    }
  }

  // Save to localStorage
  function save(): void {
    try {
      const data: CanvasStorage = {
        canvases: canvases.value,
        activeCanvasId: activeCanvasId.value,
      }
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
    } catch (e) {
      console.error('Failed to save canvas state:', e)
    }
  }

  // Update active canvas timestamp
  function touch(): void {
    if (activeCanvas.value) {
      activeCanvas.value.updatedAt = new Date().toISOString()
    }
  }

  // Canvas CRUD
  function createCanvas(name: string = 'Untitled Canvas'): CanvasState {
    const canvas = createEmptyCanvas(name)
    canvases.value.push(canvas)
    activeCanvasId.value = canvas.id
    save()
    return canvas
  }

  function setActiveCanvas(id: string): void {
    if (canvases.value.find((c) => c.id === id)) {
      activeCanvasId.value = id
      save()
    }
  }

  function deleteCanvas(id: string): void {
    const idx = canvases.value.findIndex((c) => c.id === id)
    if (idx >= 0) {
      canvases.value.splice(idx, 1)

      // Handle active canvas deletion
      if (activeCanvasId.value === id) {
        activeCanvasId.value = canvases.value[0]?.id ?? null
        if (canvases.value.length === 0) {
          createCanvas()
        }
      }
      save()
    }
  }

  function renameCanvas(id: string, name: string): void {
    const canvas = canvases.value.find((c) => c.id === id)
    if (canvas) {
      canvas.name = name
      canvas.updatedAt = new Date().toISOString()
      save()
    }
  }

  // Node operations
  function addNode(
    type: CanvasNodeType,
    position: { x: number; y: number },
    datasource?: Datasource
  ): CanvasNode | null {
    if (!activeCanvas.value) return null

    const node = createDefaultNode(type, position, datasource)
    activeCanvas.value.nodes.push(node)
    touch()
    save()
    return node
  }

  function updateNode(id: string, updates: Partial<CanvasNode>): void {
    if (!activeCanvas.value) return

    const node = activeCanvas.value.nodes.find((n) => n.id === id)
    if (node) {
      Object.assign(node, updates)
      touch()
      save()
    }
  }

  function updateNodeMeta(id: string, meta: Partial<CanvasNode['meta']>): void {
    if (!activeCanvas.value) return

    const node = activeCanvas.value.nodes.find((n) => n.id === id)
    if (node) {
      Object.assign(node.meta, meta)
      touch()
      save()
    }
  }

  function deleteNode(id: string): void {
    if (!activeCanvas.value) return

    const idx = activeCanvas.value.nodes.findIndex((n) => n.id === id)
    if (idx >= 0) {
      activeCanvas.value.nodes.splice(idx, 1)

      // Remove connected edges
      activeCanvas.value.edges = activeCanvas.value.edges.filter(
        (e) => e.from !== id && e.to !== id
      )

      touch()
      save()
    }
  }

  function duplicateNode(id: string): CanvasNode | null {
    if (!activeCanvas.value) return null

    const node = activeCanvas.value.nodes.find((n) => n.id === id)
    if (!node) return null

    const copy: CanvasNode = JSON.parse(JSON.stringify(node))
    copy.id = `${node.type[0]}-${Math.random().toString(16).slice(2, 8)}`
    copy.title = `${node.title} (copy)`
    copy.x += 30
    copy.y += 30
    // Clear runtime state
    if (copy.meta) {
      copy.meta.queryId = undefined
      copy.meta.result = null
      copy.meta.error = null
      copy.meta.status = 'idle'
      copy.meta.lastRun = 'never'
    }

    activeCanvas.value.nodes.push(copy)
    touch()
    save()
    return copy
  }

  // Edge operations
  function addEdge(from: string, to: string, label: EdgeRelationship = 'motivates'): CanvasEdge | null {
    if (!activeCanvas.value) return null

    // Prevent self-loops
    if (from === to) return null

    // Prevent duplicate edges
    const exists = activeCanvas.value.edges.some((e) => e.from === from && e.to === to)
    if (exists) return null

    const edge = createDefaultEdge(from, to)
    edge.label = label
    activeCanvas.value.edges.push(edge)
    touch()
    save()
    return edge
  }

  function updateEdge(id: string, updates: Partial<CanvasEdge>): void {
    if (!activeCanvas.value) return

    const edge = activeCanvas.value.edges.find((e) => e.id === id)
    if (edge) {
      Object.assign(edge, updates)
      touch()
      save()
    }
  }

  function deleteEdge(id: string): void {
    if (!activeCanvas.value) return

    const idx = activeCanvas.value.edges.findIndex((e) => e.id === id)
    if (idx >= 0) {
      activeCanvas.value.edges.splice(idx, 1)
      touch()
      save()
    }
  }

  // Time range
  function setTimePreset(preset: TimePreset): void {
    if (!activeCanvas.value) return
    activeCanvas.value.timeRange = { ...activeCanvas.value.timeRange, preset, offset: 0 }
    touch()
    save()
  }

  function setTimeOffset(offsetMs: number): void {
    if (!activeCanvas.value) return
    activeCanvas.value.timeRange = {
      ...activeCanvas.value.timeRange,
      preset: 'custom',
      offset: offsetMs,
    }
    touch()
    save()
  }

  function setLive(live: boolean): void {
    if (!activeCanvas.value) return
    activeCanvas.value.live = live
    if (live) {
      // Reset offset when going live
      activeCanvas.value.timeRange = { ...activeCanvas.value.timeRange, offset: 0 }
    }
    touch()
    save()
  }

  // Initialize on store creation
  load()

  return {
    // State
    canvases,
    activeCanvasId,

    // Getters
    activeCanvas,
    nodes,
    edges,

    // Canvas operations
    load,
    save,
    createCanvas,
    setActiveCanvas,
    deleteCanvas,
    renameCanvas,

    // Node operations
    addNode,
    updateNode,
    updateNodeMeta,
    deleteNode,
    duplicateNode,

    // Edge operations
    addEdge,
    updateEdge,
    deleteEdge,

    // Time range
    setTimePreset,
    setTimeOffset,
    setLive,
  }
})
