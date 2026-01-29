<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { Splitpanes, Pane } from 'splitpanes'
import 'splitpanes/dist/splitpanes.css'
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

// Panel layout - load from localStorage
const STORAGE_KEY = 'loupe:canvas:layout'

function loadLayoutFromStorage() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) return JSON.parse(stored)
  } catch {
    // ignore
  }
  return null
}

const storedLayout = loadLayoutFromStorage()
const splitDirection = ref<'vertical' | 'horizontal'>(storedLayout?.splitDirection ?? 'vertical')
const inspectorPct = ref(storedLayout?.inspectorPct ?? 30)

// Persist layout changes
watch([splitDirection, inspectorPct], () => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify({
    splitDirection: splitDirection.value,
    inspectorPct: inspectorPct.value,
  }))
})

function onPaneResized(panes: { size: number }[]) {
  if (panes[1]) {
    inspectorPct.value = panes[1].size
  }
}

onMounted(() => {
  loadDatasources()
})

onUnmounted(() => {
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
    (n) => n.type === 'query' && n.meta.datasourceId && n.meta.sql?.trim(),
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
        {
          name: 'start',
          param_type: 'datetime' as const,
          required: false,
          default: start.toISOString(),
        },
        {
          name: 'end',
          param_type: 'datetime' as const,
          required: false,
          default: end.toISOString(),
        },
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
      <CanvasToolbar @new-canvas="handleNewCanvas" />
    </template>
    <div class="h-full grid grid-rows-[64px_1fr] bg-surface text-text">
      <!-- Timeline scrubber -->
      <CanvasTimeline />

      <!-- Main -->
      <Splitpanes
        class="default-theme h-full"
        :horizontal="splitDirection === 'horizontal'"
        @resized="onPaneResized"
      >
        <Pane :size="100 - inspectorPct" :min-size="20">
          <CanvasWorkspace
            ref="workspaceRef"
            class="h-full w-full"
            :show-grid="showGrid"
            :selected-id="selectedId"
            :datasources="datasources"
            :split-direction="splitDirection"
            @update:selected-id="selectedId = $event"
            @update:show-grid="showGrid = $event"
            @update:split-direction="splitDirection = $event"
            @add-query="handleAddQuery"
            @add-note="handleAddNote"
            @edit-edge="handleEditEdge"
          />
        </Pane>

        <Pane :size="inspectorPct" :min-size="20" :max-size="60">
          <CanvasInspector
            class="h-full w-full"
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
        </Pane>
      </Splitpanes>

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
