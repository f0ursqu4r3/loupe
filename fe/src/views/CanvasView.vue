<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { datasourcesApi } from '@/services/api/datasources'
import { queriesApi, runsApi } from '@/services/api/queries'
import { timePresetToDateRange } from '@/types/canvas'
import type { Datasource } from '@/types/api'
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
    canvasStore.updateNodeMeta(selectedNode.value.id, {
      viz: viz as 'table' | 'line' | 'bar' | 'stat',
    })
  }
}

function updateNoteText(text: string) {
  if (selectedNode.value) {
    canvasStore.updateNodeMeta(selectedNode.value.id, { text })
  }
}

function centerOnSelected() {
  // TODO: Implement canvas pan to center on selected node
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

onMounted(() => {
  loadDatasources()
})
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
    <div class="h-full grid grid-rows-[64px_1fr] bg-surface text-text">
      <!-- Timeline scrubber -->
      <CanvasTimeline />

      <!-- Main -->
      <main class="grid grid-cols-[1fr_520px] h-full min-h-0">
        <!-- Canvas workspace -->
        <CanvasWorkspace
          ref="workspaceRef"
          :show-grid="showGrid"
          :selected-id="selectedId"
          :datasources="datasources"
          @update:selected-id="selectedId = $event"
          @edit-edge="handleEditEdge"
        />

        <!-- Inspector -->
        <CanvasInspector
          :node="selectedNode"
          :datasources="datasources"
          :is-running="isRunning"
          @update:title="updateNodeTitle"
          @update:datasource="updateNodeDatasource"
          @update:sql="updateNodeSql"
          @update:viz="updateNodeViz"
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
