<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { datasourcesApi } from '@/services/api/datasources'
import { queriesApi, runsApi } from '@/services/api/queries'
import { timePresetToDateRange } from '@/types/canvas'
import type { Datasource, ChartType, VisualizationConfig } from '@/types/api'
import type { CanvasEdge } from '@/types/canvas'
import { AppLayout } from '@/components/layout'
import {
  CanvasToolbar,
  CanvasTimeline,
  CanvasWorkspace,
  CanvasInspector,
  EdgeEditModal,
} from '@/components/canvas'

const canvasStore = useCanvasStore()

// Panel resize
const inspectorWidth = ref(520)
const isResizing = ref(false)
const resizeStartX = ref(0)
const resizeStartWidth = ref(0)

function startResize(e: PointerEvent) {
  isResizing.value = true
  resizeStartX.value = e.clientX
  resizeStartWidth.value = inspectorWidth.value
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onResizeMove(e: PointerEvent) {
  if (!isResizing.value) return
  const dx = resizeStartX.value - e.clientX
  const newWidth = Math.max(320, Math.min(800, resizeStartWidth.value + dx))
  inspectorWidth.value = newWidth
}

function endResize() {
  isResizing.value = false
}

onMounted(() => {
  loadDatasources()
  window.addEventListener('pointermove', onResizeMove)
  window.addEventListener('pointerup', endResize)
})

onUnmounted(() => {
  window.removeEventListener('pointermove', onResizeMove)
  window.removeEventListener('pointerup', endResize)
  stopLiveRefresh()
})

// Datasources
const datasources = ref<Datasource[]>([])

async function loadDatasources() {
  try {
    datasources.value = await datasourcesApi.list()
  } catch (e) {
    console.error('Failed to load datasources:', e)
  }
}

// Canvas state
const showGrid = ref(true)
const selectedId = ref<string | null>(null)
const workspaceRef = ref<InstanceType<typeof CanvasWorkspace> | null>(null)

const selectedNode = computed(
  () => canvasStore.nodes.find((n) => n.id === selectedId.value) ?? null,
)

// Query execution state
const isRunning = ref(false)

// Live refresh
const LIVE_REFRESH_INTERVAL = 30000 // 30 seconds
let liveRefreshTimer: ReturnType<typeof setInterval> | null = null

async function runAllQueries() {
  const queryNodes = canvasStore.nodes.filter(
    (n) => n.type === 'query' && n.meta.datasourceId && n.meta.sql?.trim()
  )

  // Run all queries in parallel
  await Promise.all(queryNodes.map((node) => runQueryNode(node.id)))
}

async function runQueryNode(nodeId: string) {
  const node = canvasStore.nodes.find((n) => n.id === nodeId)
  if (!node || node.type !== 'query') return
  if (!node.meta.datasourceId || !node.meta.sql?.trim()) return

  canvasStore.updateNodeMeta(node.id, {
    status: 'running',
    error: null,
  })

  try {
    const { start, end } = timePresetToDateRange(canvasStore.activeCanvas?.timeRange.preset ?? '7d')

    let queryId = node.meta.queryId
    const queryData = {
      datasource_id: node.meta.datasourceId,
      name: `[Canvas] ${node.title}`,
      sql: node.meta.sql,
      parameters: [
        { name: 'start', param_type: 'datetime' as const, required: false, default: start.toISOString() },
        { name: 'end', param_type: 'datetime' as const, required: false, default: end.toISOString() },
      ],
      tags: ['canvas'],
    }

    if (queryId) {
      await queriesApi.update(queryId, queryData)
    } else {
      const created = await queriesApi.create(queryData)
      queryId = created.id
      canvasStore.updateNodeMeta(node.id, { queryId })
    }

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
          lastRun: new Date().toLocaleTimeString(),
          rows: result.row_count,
          runtime: `${result.execution_time_ms}ms`,
          result,
          error: null,
        })
        return
      } else if (['failed', 'cancelled', 'timeout'].includes(status.status)) {
        canvasStore.updateNodeMeta(node.id, {
          status: 'error',
          error: status.error_message || `Query ${status.status}`,
        })
        return
      }

      await new Promise((r) => setTimeout(r, 500))
      attempts++
    }

    canvasStore.updateNodeMeta(node.id, {
      status: 'error',
      error: 'Query timed out',
    })
  } catch (e) {
    canvasStore.updateNodeMeta(node.id, {
      status: 'error',
      error: e instanceof Error ? e.message : 'Failed to execute query',
    })
  }
}

function startLiveRefresh() {
  if (liveRefreshTimer) return

  // Run immediately, then on interval
  runAllQueries()
  liveRefreshTimer = setInterval(runAllQueries, LIVE_REFRESH_INTERVAL)
}

function stopLiveRefresh() {
  if (liveRefreshTimer) {
    clearInterval(liveRefreshTimer)
    liveRefreshTimer = null
  }
}

// Watch for live mode changes
watch(
  () => canvasStore.activeCanvas?.live,
  (isLive) => {
    if (isLive) {
      startLiveRefresh()
    } else {
      stopLiveRefresh()
    }
  },
)

// Edge editing
const editingEdge = ref<CanvasEdge | null>(null)

function handleEditEdge(edge: CanvasEdge) {
  editingEdge.value = { ...edge }
}

function handleSaveEdge(edge: CanvasEdge) {
  canvasStore.updateEdge(edge.id, { label: edge.label })
}

function handleDeleteEdge(edge: CanvasEdge) {
  canvasStore.deleteEdge(edge.id)
}

// Toolbar actions
function handleAddQuery() {
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

function handleAddNote() {
  const node = canvasStore.addNode('note', {
    x: 150 + Math.random() * 200,
    y: 150 + Math.random() * 200,
  })
  if (node) {
    selectedId.value = node.id
  }
}

function handleToggleGrid() {
  showGrid.value = !showGrid.value
}

function handleNewCanvas() {
  selectedId.value = null
}

// Inspector actions
function updateNodeTitle(title: string) {
  if (selectedNode.value) {
    canvasStore.updateNode(selectedNode.value.id, { title })
  }
}

function updateNodeDatasource(id: string) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, {
      datasourceId: id || undefined,
    })
  }
}

function updateNodeSql(sql: string) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, { sql })
  }
}

function updateNodeViz(viz: string) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, { viz: viz as ChartType })
  }
}

function updateNodeVizConfig(config: VisualizationConfig) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, { vizConfig: config })
  }
}

function updateNoteText(text: string) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, { text })
  }
}

function centerOnSelected() {
  if (selectedId.value && workspaceRef.value) {
    workspaceRef.value.centerOnNode(selectedId.value)
  }
}

// Query execution (for selected node)
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
  try {
    await runQueryNode(node.id)
  } finally {
    isRunning.value = false
  }
}
</script>

<template>
  <AppLayout title="Canvas" no-padding>
    <template #header-left>
      <CanvasToolbar
        :show-grid="showGrid"
        :selected-id="selectedId"
        @add-query="handleAddQuery"
        @add-note="handleAddNote"
        @toggle-grid="handleToggleGrid"
        @new-canvas="handleNewCanvas"
      />
    </template>
    <div
      class="h-full grid grid-rows-[64px_1fr] bg-surface text-text"
      :class="{ 'select-none': isResizing }"
    >
      <!-- Timeline scrubber -->
      <CanvasTimeline />

      <!-- Main -->
      <main class="flex h-full min-h-0">
        <!-- Canvas workspace -->
        <CanvasWorkspace
          ref="workspaceRef"
          class="flex-1 min-w-0"
          :show-grid="showGrid"
          :selected-id="selectedId"
          :datasources="datasources"
          @update:selected-id="selectedId = $event"
          @edit-edge="handleEditEdge"
        />

        <!-- Resize handle -->
        <div
          class="w-1.5 bg-border hover:bg-primary-500 cursor-col-resize transition-colors shrink-0"
          :class="{ 'bg-primary-500': isResizing }"
          @pointerdown="startResize"
        />

        <!-- Inspector -->
        <CanvasInspector
          :style="{ width: `${inspectorWidth}px` }"
          class="shrink-0"
          :node="selectedNode"
          :datasources="datasources"
          :is-running="isRunning"
          @update:title="updateNodeTitle"
          @update:datasource="updateNodeDatasource"
          @update:sql="updateNodeSql"
          @update:viz="updateNodeViz"
          @update:vizConfig="updateNodeVizConfig"
          @update:note-text="updateNoteText"
          @run="runSelectedQuery"
          @center="centerOnSelected"
        />
      </main>

      <!-- Edge edit modal -->
      <EdgeEditModal
        :edge="editingEdge"
        @update:edge="editingEdge = $event"
        @save="handleSaveEdge"
        @delete="handleDeleteEdge"
      />
    </div>
  </AppLayout>
</template>
