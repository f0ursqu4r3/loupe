<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import * as monaco from 'monaco-editor'
import { useCanvasStore } from '@/stores/canvas'
import { datasourcesApi } from '@/services/api/datasources'
import { queriesApi, runsApi } from '@/services/api/queries'
import type { Datasource } from '@/types/api'
import type { CanvasNode, CanvasEdge, TimePreset, EdgeRelationship } from '@/types/canvas'
import { timePresetToDateRange } from '@/types/canvas'
import { AppLayout } from '@/components/layout'
import { LButton, LSelect, LModal, LBadge, LCheckbox, LEmptyState } from '@/components/ui'
import { Play, Crosshair } from 'lucide-vue-next'

const canvasStore = useCanvasStore()

// Datasources
const datasources = ref<Datasource[]>([])

async function loadDatasources() {
  try {
    datasources.value = await datasourcesApi.list()
  } catch (e) {
    console.error('Failed to load datasources:', e)
  }
}

function getDatasourceName(id?: string): string {
  if (!id) return '(none)'
  const ds = datasources.value.find((d) => d.id === id)
  return ds?.name ?? '(unknown)'
}

// Canvas state
const canvasEl = ref<HTMLElement | null>(null)
const monacoHost = ref<HTMLElement | null>(null)
const showGrid = ref(true)
const canvasSize = reactive({ w: 3000, h: 2000 })

// Selection
const selectedId = ref<string | null>(null)
const selectedNode = computed(
  () => canvasStore.nodes.find((n) => n.id === selectedId.value) ?? null,
)

// UI state
const splitCollapsed = reactive({ sql: false, viz: false })
const isRunning = ref(false)

// Time presets
const timePresets: { value: TimePreset; label: string }[] = [
  { value: 'now', label: 'Now' },
  { value: '24h', label: '24h' },
  { value: '7d', label: '7d' },
  { value: '30d', label: '30d' },
  { value: '90d', label: '90d' },
]

const timeRangeLabel = computed(() => {
  const preset = canvasStore.activeCanvas?.timeRange.preset ?? '7d'
  const live = canvasStore.activeCanvas?.live ? ' (live)' : ''
  return `Last ${preset}${live}`
})

// Canvas options for LSelect
const canvasOptions = computed(() =>
  canvasStore.canvases.map((c) => ({ value: c.id, label: c.name })),
)

// Datasource options for LSelect
const datasourceOptions = computed(() => [
  { value: '', label: 'Select datasource...' },
  ...datasources.value.map((ds) => ({ value: ds.id, label: ds.name })),
])

// Viz options for LSelect
const vizOptions = [
  { value: 'table', label: 'Table' },
  { value: 'line', label: 'Line' },
  { value: 'bar', label: 'Bar' },
  { value: 'stat', label: 'Stat' },
]

// Edge relationship options for LSelect
const edgeRelationshipOptions = computed(() =>
  edgeRelationships.map((rel) => ({ value: rel, label: rel })),
)

function setTimePreset(preset: TimePreset) {
  canvasStore.setTimePreset(preset)
}

function toggleLive(value: boolean) {
  canvasStore.setLive(value)
}

// Edge relationships
const edgeRelationships: EdgeRelationship[] = [
  'motivates',
  'explains',
  'contradicts',
  'supports',
  'derives_from',
  'questions',
]

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

// Edge editing
const editingEdge = ref<CanvasEdge | null>(null)

function editEdgeLabel(edge: CanvasEdge) {
  editingEdge.value = { ...edge }
}

function saveEditingEdge() {
  if (editingEdge.value) {
    canvasStore.updateEdge(editingEdge.value.id, { label: editingEdge.value.label })
    editingEdge.value = null
  }
}

function deleteEditingEdge() {
  if (editingEdge.value) {
    canvasStore.deleteEdge(editingEdge.value.id)
    editingEdge.value = null
  }
}

// Canvas operations
function onCanvasChange(id: string | number) {
  canvasStore.setActiveCanvas(String(id))
  selectedId.value = null
}

function createNewCanvas() {
  const name = prompt('Canvas name:', 'New Canvas')
  if (name) {
    canvasStore.createCanvas(name)
    selectedId.value = null
  }
}

function toggleGrid() {
  showGrid.value = !showGrid.value
}

// Node operations
function addNode() {
  const defaultDs = datasources.value[0]
  const node = canvasStore.addNode(
    'query',
    { x: 150 + Math.random() * 200, y: 150 + Math.random() * 200 },
    defaultDs,
  )
  if (node) {
    selectedId.value = node.id
  }
}

function addNote() {
  const node = canvasStore.addNode('note', {
    x: 150 + Math.random() * 200,
    y: 150 + Math.random() * 200,
  })
  if (node) {
    selectedId.value = node.id
  }
}

function selectNode(id: string) {
  // Handle connect mode
  if (connectMode.active && connectMode.from && connectMode.from !== id) {
    canvasStore.addEdge(connectMode.from, id, connectMode.label)
    cancelConnect()
  }
  selectedId.value = id
}

function duplicateNode(id: string) {
  const copy = canvasStore.duplicateNode(id)
  if (copy) {
    selectedId.value = copy.id
  }
}

function deleteNode(id: string) {
  canvasStore.deleteNode(id)
  if (selectedId.value === id) {
    selectedId.value = null
  }
  if (connectMode.from === id) {
    cancelConnect()
  }
}

// Node updates
function updateNodeTitle(title: string | number) {
  if (selectedNode.value) {
    canvasStore.updateNode(selectedNode.value.id, { title: String(title) })
  }
}

function updateNodeDatasource(id: string | number) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, {
      datasourceId: id ? String(id) : undefined,
    })
  }
}

function updateNodeViz(viz: string | number) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, {
      viz: String(viz) as 'table' | 'line' | 'bar' | 'stat',
    })
  }
}

function updateNoteText(text: string) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, { text })
  }
}

// Query execution
async function runSelectedQuery() {
  const node = selectedNode.value
  if (!node || node.type !== 'query') return
  if (!node.meta.datasourceId) {
    canvasStore.updateNodeMeta(node.id, { error: 'Please select a datasource' })
    return
  }
  if (!node.meta.sql?.trim()) {
    canvasStore.updateNodeMeta(node.id, { error: 'No SQL to execute' })
    return
  }

  isRunning.value = true
  canvasStore.updateNodeMeta(node.id, {
    status: 'running',
    error: null,
    result: null,
  })

  try {
    // Get time range parameters
    const { start, end } = timePresetToDateRange(canvasStore.activeCanvas?.timeRange.preset ?? '7d')

    // Create or update the query in the backend
    let queryId = node.meta.queryId
    const queryData = {
      datasource_id: node.meta.datasourceId,
      name: `[Canvas] ${node.title}`,
      sql: node.meta.sql,
      parameters: [],
      tags: ['canvas'],
    }

    if (queryId) {
      await queriesApi.update(queryId, queryData)
    } else {
      const created = await queriesApi.create(queryData)
      queryId = created.id
      canvasStore.updateNodeMeta(node.id, { queryId })
    }

    // Execute the query
    const run = await runsApi.create({
      query_id: queryId,
      parameters: {
        start: start.toISOString(),
        end: end.toISOString(),
      },
    })

    // Poll for completion
    let attempts = 0
    const maxAttempts = 60

    while (attempts < maxAttempts) {
      const status = await runsApi.get(run.id)

      if (status.status === 'completed') {
        const result = await runsApi.getResult(run.id)
        canvasStore.updateNodeMeta(node.id, {
          status: 'ok',
          lastRun: 'just now',
          rows: result.row_count,
          runtime: `${result.execution_time_ms}ms`,
          result,
          error: null,
        })
        break
      } else if (['failed', 'cancelled', 'timeout'].includes(status.status)) {
        canvasStore.updateNodeMeta(node.id, {
          status: 'error',
          error: status.error_message || `Query ${status.status}`,
        })
        break
      }

      await new Promise((r) => setTimeout(r, 500))
      attempts++
    }

    if (attempts >= maxAttempts) {
      canvasStore.updateNodeMeta(node.id, {
        status: 'error',
        error: 'Query timed out',
      })
    }
  } catch (e) {
    canvasStore.updateNodeMeta(node.id, {
      status: 'error',
      error: e instanceof Error ? e.message : 'Failed to execute query',
    })
  } finally {
    isRunning.value = false
  }
}

function formatCell(value: unknown): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

function vizTitle(v?: string) {
  switch (v) {
    case 'line':
      return 'Line Chart'
    case 'bar':
      return 'Bar Chart'
    case 'stat':
      return 'Stat'
    default:
      return 'Table'
  }
}

function formatSql() {
  const n = selectedNode.value
  if (!n || n.type !== 'query') return
  // Simple format: just trim and ensure newline
  const formatted = (n.meta.sql ?? '').trim() + '\n'
  canvasStore.updateNodeMeta(n.id, { sql: formatted })
  editor?.setValue(formatted)
}

function togglePane(which: 'sql' | 'viz') {
  splitCollapsed[which] = !splitCollapsed[which]
}

function centerOnSelected() {
  // TODO: Implement canvas pan to center on selected node
}

// Node styling
function nodeStyle(n: CanvasNode) {
  return {
    left: `${n.x}px`,
    top: `${n.y}px`,
    width: `${n.w}px`,
    height: `${n.h}px`,
  }
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
    selectedId.value = null
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

// Split resize
let resizing = false
const split = reactive({ leftPct: 50 })

function startResize(e: PointerEvent) {
  resizing = true
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onResizeMove(e: PointerEvent) {
  if (!resizing) return
  const inspector = document.querySelector('.inspector') as HTMLElement | null
  const splitEl = inspector?.querySelector('.split-pane') as HTMLElement | null
  if (!splitEl) return

  const rect = splitEl.getBoundingClientRect()
  const pct = ((e.clientX - rect.left) / rect.width) * 100
  split.leftPct = Math.min(80, Math.max(20, pct))
  splitEl.style.setProperty('--leftPct', `${split.leftPct}%`)
}

function endResize() {
  resizing = false
}

// Monaco integration
let editor: monaco.editor.IStandaloneCodeEditor | null = null

async function ensureMonaco() {
  await nextTick()
  const host = monacoHost.value
  const n = selectedNode.value
  if (!host || !n || n.type !== 'query') return

  if (!editor) {
    editor = monaco.editor.create(host, {
      value: n.meta.sql ?? '',
      language: 'sql',
      theme: 'vs-dark',
      minimap: { enabled: false },
      fontSize: 13,
      automaticLayout: true,
      scrollBeyondLastLine: false,
    })

    editor.onDidChangeModelContent(() => {
      const cur = selectedNode.value
      if (cur && cur.type === 'query') {
        canvasStore.updateNodeMeta(cur.id, { sql: editor!.getValue() })
      }
    })
  } else {
    editor.setValue(n.meta.sql ?? '')
  }
}

watch(
  () => selectedId.value,
  async () => {
    if (editor) {
      editor.getModel()?.dispose()
      editor.dispose()
      editor = null
    }
    await ensureMonaco()
  },
)

watch(
  () => selectedNode.value?.meta?.sql,
  (v) => {
    if (!editor) return
    if (typeof v === 'string' && v !== editor.getValue()) {
      editor.setValue(v)
    }
  },
)

onMounted(async () => {
  loadDatasources()
  window.addEventListener('pointermove', onResizeMove)
  window.addEventListener('pointerup', endResize)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onResizeMove)
  window.removeEventListener('pointerup', endResize)
  if (editor) {
    editor.dispose()
    editor = null
  }
})
</script>

<template>
  <AppLayout title="Canvas" no-padding>
    <div class="h-full grid grid-rows-[56px_64px_1fr] bg-surface text-text">
      <!-- Top bar -->
      <header class="flex items-center gap-3 px-4 border-b border-border bg-surface-raised">
        <div class="flex items-center gap-2 font-semibold">
          <LSelect
            :model-value="canvasStore.activeCanvasId ?? ''"
            :options="canvasOptions"
            class="min-w-40"
            @update:model-value="onCanvasChange"
          />
          <LButton variant="secondary" size="sm" @click="createNewCanvas" title="New Canvas">
            +
          </LButton>
        </div>

        <div class="flex gap-2">
          <LButton variant="secondary" @click="addNode()">New Query Node</LButton>
          <LButton variant="secondary" @click="addNote()">New Note</LButton>
          <LButton variant="secondary" @click="toggleGrid()">
            {{ showGrid ? 'Hide Grid' : 'Show Grid' }}
          </LButton>
        </div>

        <div class="flex-1"></div>

        <div class="flex gap-2 items-center">
          <LBadge>Time: {{ timeRangeLabel }}</LBadge>
          <LBadge v-if="selectedId">Selected: {{ selectedId }}</LBadge>
        </div>
      </header>

      <!-- Timeline scrubber -->
      <section
        class="grid grid-cols-[240px_1fr_120px] items-center gap-3 px-4 border-b border-border bg-surface-sunken"
      >
        <div class="flex items-center gap-2">
          <div class="flex flex-col gap-0.5">
            <div class="font-semibold text-sm">Timeline</div>
            <div class="text-xs text-text-muted">Global time window for all queries</div>
          </div>
        </div>

        <div class="flex justify-center">
          <div class="flex gap-1.5">
            <LButton
              v-for="p in timePresets"
              :key="p.value"
              :variant="canvasStore.activeCanvas?.timeRange.preset === p.value ? 'primary' : 'secondary'"
              size="sm"
              @click="setTimePreset(p.value)"
            >
              {{ p.label }}
            </LButton>
          </div>
        </div>

        <div class="flex justify-end items-center">
          <label class="inline-flex gap-2 items-center text-xs text-text-muted cursor-pointer">
            <LCheckbox
              :model-value="canvasStore.activeCanvas?.live ?? false"
              @update:model-value="toggleLive"
            />
            Live
          </label>
        </div>
      </section>

      <!-- Main -->
      <main class="grid grid-cols-[1fr_520px] h-full min-h-0">
        <!-- Canvas -->
        <section
          ref="canvasEl"
          class="relative overflow-auto min-h-0 bg-surface-sunken"
          :class="{ 'canvas-grid': showGrid }"
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
                @click.stop="editEdgeLabel(e)"
              >
                {{ e.label }}
              </text>
            </g>
          </svg>

          <!-- Nodes -->
          <div
            v-for="n in canvasStore.nodes"
            :key="n.id"
            class="absolute rounded-xl border bg-surface-raised shadow-lg grid grid-rows-[36px_1fr_32px] select-none cursor-grab"
            :class="{
              'border-border-strong ring-2 ring-border': n.id === selectedId,
              'border-border': n.id !== selectedId,
            }"
            :style="nodeStyle(n)"
            @pointerdown.stop="onNodePointerDown($event, n.id)"
            @dblclick.stop="selectNode(n.id)"
          >
            <div class="flex items-center justify-between px-2.5 py-2 border-b border-border">
              <div
                class="flex items-center gap-2 font-semibold text-sm overflow-hidden text-ellipsis whitespace-nowrap"
              >
                <span
                  class="w-2.5 h-2.5 rounded-full shrink-0"
                  :class="n.type === 'query' ? 'bg-info' : 'bg-warning'"
                ></span>
                <span>{{ n.title }}</span>
              </div>
              <div class="flex gap-1.5">
                <button
                  class="w-6 h-6 rounded-lg border border-border bg-surface hover:bg-surface-sunken text-xs flex items-center justify-center"
                  title="Connect"
                  @click.stop="startConnect(n.id)"
                >
                  &#x26d3;
                </button>
                <button
                  class="w-6 h-6 rounded-lg border border-border bg-surface hover:bg-surface-sunken text-xs flex items-center justify-center"
                  title="Duplicate"
                  @click.stop="duplicateNode(n.id)"
                >
                  &#x29c9;
                </button>
                <button
                  class="w-6 h-6 rounded-lg border border-border bg-surface hover:bg-surface-sunken text-xs flex items-center justify-center"
                  title="Delete"
                  @click.stop="deleteNode(n.id)"
                >
                  &#x2715;
                </button>
              </div>
            </div>

            <div class="p-2.5 min-h-0 flex flex-col gap-2.5 overflow-hidden">
              <template v-if="n.type === 'query'">
                <div class="grid gap-1.5 text-xs">
                  <div class="flex justify-between gap-2.5">
                    <span class="text-text-muted">Data source</span>
                    <span>{{ getDatasourceName(n.meta.datasourceId) }}</span>
                  </div>
                  <div class="flex justify-between gap-2.5">
                    <span class="text-text-muted">Last run</span>
                    <span>{{ n.meta.lastRun || 'never' }}</span>
                  </div>
                  <div class="flex justify-between gap-2.5">
                    <span class="text-text-muted">Status</span>
                    <span
                      :class="{
                        'text-success': n.meta.status === 'ok',
                        'text-warning': n.meta.status === 'warn',
                        'text-error': n.meta.status === 'error',
                        'text-info': n.meta.status === 'running',
                        'text-text-subtle': n.meta.status === 'idle' || !n.meta.status,
                      }"
                    >
                      {{ n.meta.status || 'idle' }}
                    </span>
                  </div>
                </div>
                <div class="text-xs text-text-muted">Double-click to open editor</div>
              </template>

              <template v-else>
                <div class="text-xs leading-snug whitespace-pre-wrap overflow-hidden">
                  {{ n.meta.text || '(empty note)' }}
                </div>
              </template>
            </div>

            <div class="flex gap-2 items-center px-2.5 py-2 border-t border-border overflow-hidden">
              <LBadge size="sm">{{ timeRangeLabel }}</LBadge>
              <LBadge v-if="n.type === 'query'" variant="info" size="sm">SQL</LBadge>
              <LBadge v-if="n.type === 'note'" variant="warning" size="sm">NOTE</LBadge>
            </div>
          </div>

          <!-- Connect mode cursor hint -->
          <div
            v-if="connectMode.active"
            class="sticky left-4 top-4 inline-flex gap-2.5 items-center px-3 py-2.5 rounded-xl border border-border bg-surface-overlay/80 backdrop-blur-sm m-4 w-fit"
          >
            Click a target node to connect from <b>{{ connectMode.from }}</b>
            <LSelect
              v-model="connectMode.label"
              :options="edgeRelationshipOptions"
              class="w-36"
            />
            <LButton variant="secondary" size="sm" @click="cancelConnect()">Cancel</LButton>
          </div>
        </section>

        <!-- Inspector -->
        <aside
          class="inspector border-l border-border bg-surface-raised grid grid-rows-[auto_auto_1fr] min-h-0 overflow-hidden"
        >
          <LEmptyState
            v-if="!selectedNode"
            title="No selection"
            description="Select a node to edit SQL, view results, or add notes."
            class="py-8"
          />

          <template v-else>
            <div class="p-3 flex items-center justify-between gap-3 border-b border-border">
              <div class="flex items-center gap-2.5 min-w-0 flex-1">
                <span
                  class="w-2.5 h-2.5 rounded-full shrink-0"
                  :class="selectedNode.type === 'query' ? 'bg-info' : 'bg-warning'"
                ></span>
                <input
                  class="w-full min-w-0 font-bold text-sm bg-transparent border-none outline-none"
                  :value="selectedNode.title"
                  @input="($event) => updateNodeTitle(($event.target as HTMLInputElement).value)"
                  spellcheck="false"
                />
              </div>

              <div class="flex gap-2">
                <LButton
                  v-if="selectedNode.type === 'query'"
                  variant="primary"
                  :loading="isRunning"
                  @click="runSelectedQuery()"
                >
                  <Play class="h-4 w-4" />
                  Run
                </LButton>
                <LButton variant="secondary" @click="centerOnSelected()">
                  <Crosshair class="h-4 w-4" />
                  Center
                </LButton>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-2.5 p-3 border-b border-border">
              <div class="grid gap-1">
                <div class="text-[11px] text-text-subtle">Global time</div>
                <div class="text-xs">
                  {{ timeRangeLabel }}
                  <span v-if="canvasStore.activeCanvas?.live" class="text-text-muted">(live)</span>
                </div>
              </div>
              <div v-if="selectedNode.type === 'query'" class="grid gap-1">
                <div class="text-[11px] text-text-subtle">Source</div>
                <LSelect
                  :model-value="selectedNode.meta.datasourceId ?? ''"
                  :options="datasourceOptions"
                  class="text-xs"
                  @update:model-value="updateNodeDatasource"
                />
              </div>
              <div class="grid gap-1">
                <div class="text-[11px] text-text-subtle">Node id</div>
                <div class="text-xs font-mono">{{ selectedNode.id }}</div>
              </div>
            </div>

            <template v-if="selectedNode.type === 'query'">
              <!-- Split pane: SQL / Viz -->
              <div
                class="split-pane grid h-full min-h-0"
                :style="{
                  gridTemplateColumns: `${split.leftPct}% 10px calc(100% - ${split.leftPct}% - 10px)`,
                }"
              >
                <div class="min-h-0 grid grid-rows-[40px_1fr] overflow-hidden">
                  <div
                    class="flex items-center justify-between px-2.5 py-2 border-b border-border font-semibold text-xs"
                  >
                    <span>SQL Editor</span>
                    <div class="flex gap-2 items-center">
                      <LButton variant="ghost" size="sm" @click="formatSql()">Format</LButton>
                      <LButton variant="ghost" size="sm" @click="togglePane('sql')">
                        {{ splitCollapsed.sql ? 'Show' : 'Hide' }}
                      </LButton>
                    </div>
                  </div>

                  <div v-show="!splitCollapsed.sql" class="min-h-0 overflow-hidden">
                    <div ref="monacoHost" class="h-full w-full"></div>
                  </div>
                </div>

                <div
                  class="cursor-col-resize bg-border-muted hover:bg-border"
                  @pointerdown="startResize"
                ></div>

                <div class="min-h-0 grid grid-rows-[40px_1fr] overflow-hidden">
                  <div
                    class="flex items-center justify-between px-2.5 py-2 border-b border-border font-semibold text-xs"
                  >
                    <span>Visualization</span>
                    <div class="flex gap-2 items-center">
                      <LSelect
                        :model-value="selectedNode.meta.viz ?? 'table'"
                        :options="vizOptions"
                        class="w-24"
                        @update:model-value="updateNodeViz"
                      />
                      <LButton variant="ghost" size="sm" @click="togglePane('viz')">
                        {{ splitCollapsed.viz ? 'Show' : 'Hide' }}
                      </LButton>
                    </div>
                  </div>

                  <div v-show="!splitCollapsed.viz" class="min-h-0 overflow-auto">
                    <!-- Error state -->
                    <div v-if="selectedNode.meta.error" class="p-5 text-center">
                      <div class="font-bold text-error mb-2">Error</div>
                      <div class="text-sm text-error/70">{{ selectedNode.meta.error }}</div>
                    </div>

                    <!-- No results yet -->
                    <div
                      v-else-if="!selectedNode.meta.result"
                      class="p-5 flex flex-col items-center justify-center gap-2 text-text-muted text-center"
                    >
                      <div class="font-bold">{{ vizTitle(selectedNode.meta.viz) }}</div>
                      <div class="text-sm">Run the query to see results</div>
                    </div>

                    <!-- Results -->
                    <div v-else class="p-3 flex flex-col gap-3">
                      <div class="flex gap-2 flex-wrap">
                        <LBadge>Rows: {{ selectedNode.meta.result.row_count }}</LBadge>
                        <LBadge>Runtime: {{ selectedNode.meta.result.execution_time_ms }}ms</LBadge>
                        <LBadge v-if="selectedNode.meta.result.truncated" variant="warning">
                          Truncated
                        </LBadge>
                      </div>

                      <!-- Table visualization -->
                      <div
                        v-if="selectedNode.meta.viz === 'table'"
                        class="border border-border rounded-xl overflow-hidden"
                      >
                        <div
                          class="grid gap-2.5 p-2.5 font-bold bg-surface-sunken border-b border-border text-xs"
                          :style="{
                            gridTemplateColumns: `repeat(${selectedNode.meta.result.columns.length}, minmax(80px, 1fr))`,
                          }"
                        >
                          <span
                            v-for="col in selectedNode.meta.result.columns"
                            :key="col.name"
                            class="font-mono"
                          >
                            {{ col.name }}
                          </span>
                        </div>
                        <div
                          v-for="(row, idx) in selectedNode.meta.result.rows.slice(0, 50)"
                          :key="idx"
                          class="grid gap-2.5 p-2.5 border-b border-border-muted last:border-b-0 text-xs"
                          :style="{
                            gridTemplateColumns: `repeat(${selectedNode.meta.result.columns.length}, minmax(80px, 1fr))`,
                          }"
                        >
                          <span v-for="(cell, cidx) in row" :key="cidx" class="font-mono truncate">
                            {{ formatCell(cell) }}
                          </span>
                        </div>
                        <div
                          v-if="selectedNode.meta.result.rows.length > 50"
                          class="p-2.5 text-xs text-text-muted text-center"
                        >
                          ... and {{ selectedNode.meta.result.rows.length - 50 }} more rows
                        </div>
                      </div>

                      <!-- Chart placeholder for non-table -->
                      <div
                        v-else
                        class="p-5 flex flex-col items-center justify-center gap-2 text-text-muted text-center"
                      >
                        <div class="font-bold">{{ vizTitle(selectedNode.meta.viz) }}</div>
                        <div class="text-sm">
                          Chart rendering coming soon. Data available:
                          {{ selectedNode.meta.result.row_count }} rows
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </template>

            <template v-else>
              <!-- Note editor -->
              <div class="h-full min-h-0 grid grid-rows-[40px_1fr]">
                <div
                  class="flex items-center px-2.5 py-2 border-b border-border font-semibold text-xs"
                >
                  <span>Note</span>
                </div>
                <textarea
                  class="w-full h-full p-3 border-none outline-none resize-none bg-transparent text-sm leading-relaxed"
                  :value="selectedNode.meta.text"
                  @input="($event) => updateNoteText(($event.target as HTMLTextAreaElement).value)"
                  placeholder="Write context, assumptions, conclusions..."
                ></textarea>
              </div>
            </template>
          </template>
        </aside>
      </main>

      <!-- Edge label edit modal -->
      <LModal
        :open="!!editingEdge"
        title="Edge Relationship"
        size="sm"
        @close="editingEdge = null"
      >
        <LSelect
          v-if="editingEdge"
          v-model="editingEdge.label"
          :options="edgeRelationshipOptions"
        />
        <template #footer>
          <LButton variant="danger" @click="deleteEditingEdge()">Delete Edge</LButton>
          <LButton variant="primary" @click="saveEditingEdge()">Save</LButton>
        </template>
      </LModal>
    </div>
  </AppLayout>
</template>

<style scoped>
/* Canvas grid pattern - keeping as scoped since it's unique to this component */
.canvas-grid {
  background-image:
    linear-gradient(var(--loupe-border-muted) 1px, transparent 1px),
    linear-gradient(90deg, var(--loupe-border-muted) 1px, transparent 1px);
  background-size: 24px 24px;
}
</style>
