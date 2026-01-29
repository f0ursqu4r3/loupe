<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Save,
  Loader2,
  AlertCircle,
  CheckCircle,
  X,
  Plus,
  Trash2,
  Settings,
  GripVertical,
} from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LInput, LCard, LSpinner, LModal, LEmptyState, LTagsInput } from '@/components/ui'
import { GridItem } from '@/components/dashboard'
import { VisualizationRenderer } from '@/components/charts'
import { dashboardsApi, visualizationsApi, runsApi } from '@/services/api'
import { setLastDashboardId } from '@/utils/dashboardHistory'
import type {
  Dashboard,
  Tile,
  Visualization,
  QueryResult,
  CreateDashboardRequest,
  CreateTileRequest,
} from '@/types'

const route = useRoute()
const router = useRouter()

// Route params
const dashboardId = computed(() => route.params.id as string | undefined)
const isNew = computed(() => !dashboardId.value || dashboardId.value === 'new')

function createEmptyDashboard(): Partial<Dashboard> {
  return {
    name: '',
    description: '',
    parameters: [],
    tags: [],
    tiles: [],
  }
}

// Dashboard state
const dashboard = ref<Partial<Dashboard>>(createEmptyDashboard())

// UI state
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)

// Add tile modal
const showAddTileModal = ref(false)
const visualizations = ref<Visualization[]>([])
const selectedVisualizationId = ref<string | null>(null)
const addingTile = ref(false)

// Settings panel collapsed state (default collapsed for existing dashboards)
const settingsCollapsed = ref(true)

// Tile data cache - stores query results for each visualization
const tileData = ref<Record<string, QueryResult | null>>({})
const tileLoading = ref<Record<string, boolean>>({})

// Refs to GridItem components for triggering drag
const tileRefs = ref<Record<string, InstanceType<typeof GridItem>>>({})

const tilePollToken = ref(0)
const isActive = ref(true)

// Grid settings
const GRID_COLS = 12
const GRID_ROW_HEIGHT = 80

function resetDashboardState() {
  dashboard.value = createEmptyDashboard()
  tileData.value = {}
  tileLoading.value = {}
  tileRefs.value = {}
  vizCache.value = {}
  error.value = null
  saveSuccess.value = false
  showAddTileModal.value = false
  selectedVisualizationId.value = null
  settingsCollapsed.value = true
  loading.value = false
  saving.value = false
  addingTile.value = false
}

// Load dashboard
async function loadDashboard() {
  const pollToken = ++tilePollToken.value

  if (isNew.value) {
    // Show settings for new dashboards so user can enter name
    settingsCollapsed.value = false
    return
  }

  try {
    loading.value = true
    const data = await dashboardsApi.get(dashboardId.value!)
    if (!isActive.value || pollToken !== tilePollToken.value) return
    dashboard.value = data
    setLastDashboardId(data.id)

    // Load tile data
    for (const tile of data.tiles) {
      loadTileData(tile, pollToken)
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load dashboard'
  } finally {
    if (isActive.value && pollToken === tilePollToken.value) {
      loading.value = false
    }
  }
}

// Load data for a tile
async function loadTileData(tile: Tile, pollToken: number) {
  if (!isActive.value || pollToken !== tilePollToken.value) return
  tileLoading.value[tile.id] = true
  try {
    // Get the visualization to find the query
    const viz = await visualizationsApi.get(tile.visualization_id)
    if (!isActive.value || pollToken !== tilePollToken.value) return

    // Run the query
    const run = await runsApi.create({
      query_id: viz.query_id,
      parameters: {},
    })
    if (!isActive.value || pollToken !== tilePollToken.value) return

    // Poll for completion
    const maxAttempts = 60
    let attempts = 0
    while (attempts < maxAttempts) {
      if (!isActive.value || pollToken !== tilePollToken.value) return
      const status = await runsApi.get(run.id)
      if (!isActive.value || pollToken !== tilePollToken.value) return
      if (status.status === 'completed') {
        const data = await runsApi.getResult(run.id)
        if (!isActive.value || pollToken !== tilePollToken.value) return
        tileData.value[tile.id] = data
        break
      } else if (
        status.status === 'failed' ||
        status.status === 'cancelled' ||
        status.status === 'timeout'
      ) {
        tileData.value[tile.id] = null
        break
      }
      await new Promise((r) => setTimeout(r, 500))
      attempts++
    }
  } catch (e) {
    console.error('Failed to load tile data:', e)
    if (isActive.value && pollToken === tilePollToken.value) {
      tileData.value[tile.id] = null
    }
  } finally {
    if (isActive.value && pollToken === tilePollToken.value) {
      tileLoading.value[tile.id] = false
    }
  }
}

// Save dashboard
async function saveDashboard() {
  if (!dashboard.value.name?.trim()) {
    error.value = 'Dashboard name is required'
    return
  }

  try {
    saving.value = true
    error.value = null

    const payload: CreateDashboardRequest = {
      name: dashboard.value.name!,
      description: dashboard.value.description,
      parameters: dashboard.value.parameters,
      tags: dashboard.value.tags,
    }

    if (isNew.value) {
      const created = await dashboardsApi.create(payload)
      router.replace({ name: 'dashboard-editor', params: { id: created.id } })
      dashboard.value = created
      setLastDashboardId(created.id)
    } else {
      const updated = await dashboardsApi.update(dashboardId.value!, payload)
      dashboard.value = updated
      setLastDashboardId(updated.id)
    }

    saveSuccess.value = true
    setTimeout(() => (saveSuccess.value = false), 2000)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save dashboard'
  } finally {
    saving.value = false
  }
}

// Load visualizations for add tile modal
async function openAddTileModal() {
  showAddTileModal.value = true
  try {
    visualizations.value = await visualizationsApi.list()
  } catch (e) {
    console.error('Failed to load visualizations:', e)
  }
}

// Add a tile
async function addTile() {
  if (!selectedVisualizationId.value || isNew.value) return

  try {
    addingTile.value = true

    // Find next available position
    const tiles = dashboard.value.tiles || []
    const maxY = tiles.reduce((max, t) => Math.max(max, t.pos_y + t.height), 0)

    const payload: CreateTileRequest = {
      visualization_id: selectedVisualizationId.value,
      pos_x: 0,
      pos_y: maxY,
      width: 6,
      height: 4,
    }

    const tile = await dashboardsApi.createTile(dashboardId.value!, payload)
    dashboard.value.tiles = [...(dashboard.value.tiles || []), tile]
    loadTileData(tile, tilePollToken.value)

    showAddTileModal.value = false
    selectedVisualizationId.value = null
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to add tile'
  } finally {
    addingTile.value = false
  }
}

// Delete a tile
async function deleteTile(tileId: string) {
  if (!confirm('Remove this tile from the dashboard?')) return

  try {
    await dashboardsApi.deleteTile(dashboardId.value!, tileId)
    dashboard.value.tiles = dashboard.value.tiles?.filter((t) => t.id !== tileId)
    delete tileData.value[tileId]
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to delete tile'
  }
}

// Update tile position/size
async function updateTile(tile: Tile, updates: Partial<Tile>) {
  try {
    // Apply collision resolution before saving
    const resolvedTiles = resolveCollisions(tile.id, updates)

    // Update all affected tiles
    for (const resolved of resolvedTiles) {
      if (resolved.id === tile.id) {
        const updated = await dashboardsApi.updateTile(dashboardId.value!, tile.id, updates)
        const idx = dashboard.value.tiles?.findIndex((t) => t.id === tile.id)
        if (idx !== undefined && idx >= 0 && dashboard.value.tiles) {
          dashboard.value.tiles[idx] = updated
        }
      } else {
        // Update other tiles that were pushed
        const updated = await dashboardsApi.updateTile(dashboardId.value!, resolved.id, {
          pos_x: resolved.pos_x,
          pos_y: resolved.pos_y,
        })
        const idx = dashboard.value.tiles?.findIndex((t) => t.id === resolved.id)
        if (idx !== undefined && idx >= 0 && dashboard.value.tiles) {
          dashboard.value.tiles[idx] = updated
        }
      }
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to update tile'
  }
}

// Track tile being dragged for collision preview
const draggingTileId = ref<string | null>(null)
const tileOffsets = ref<Record<string, { x: number; y: number }>>({})

// Check if two tiles overlap
function tilesOverlap(
  a: { pos_x: number; pos_y: number; width: number; height: number },
  b: { pos_x: number; pos_y: number; width: number; height: number },
): boolean {
  return !(
    a.pos_x + a.width <= b.pos_x ||
    b.pos_x + b.width <= a.pos_x ||
    a.pos_y + a.height <= b.pos_y ||
    b.pos_y + b.height <= a.pos_y
  )
}

// Preview collision handling during drag/resize
function handleTilePreview(
  tileId: string,
  newX: number,
  newY: number,
  newWidth: number,
  newHeight: number,
) {
  draggingTileId.value = tileId
  const tiles = dashboard.value.tiles || []

  const movingTile = {
    pos_x: newX,
    pos_y: newY,
    width: newWidth,
    height: newHeight,
  }

  // Calculate offsets for tiles that would collide
  const newOffsets: Record<string, { x: number; y: number }> = {}

  for (const tile of tiles) {
    if (tile.id === tileId) continue

    const staticTile = {
      pos_x: tile.pos_x,
      pos_y: tile.pos_y,
      width: tile.width,
      height: tile.height,
    }

    if (tilesOverlap(movingTile, staticTile)) {
      // Push tile down below the moving tile
      const pushY = movingTile.pos_y + movingTile.height - tile.pos_y
      newOffsets[tile.id] = { x: 0, y: pushY }
    }
  }

  tileOffsets.value = newOffsets
}

// Clear preview offsets when drag ends
function clearTilePreview() {
  draggingTileId.value = null
  tileOffsets.value = {}
}

// Resolve collisions and return all tiles that need updates
function resolveCollisions(
  movedTileId: string,
  updates: Partial<Tile>,
): Array<{ id: string; pos_x: number; pos_y: number }> {
  const tiles = dashboard.value.tiles || []
  const result: Array<{ id: string; pos_x: number; pos_y: number }> = []

  const movedTile = tiles.find((t) => t.id === movedTileId)
  if (!movedTile) return result

  const moving = {
    pos_x: updates.pos_x ?? movedTile.pos_x,
    pos_y: updates.pos_y ?? movedTile.pos_y,
    width: updates.width ?? movedTile.width,
    height: updates.height ?? movedTile.height,
  }

  result.push({ id: movedTileId, pos_x: moving.pos_x, pos_y: moving.pos_y })

  // Find and push colliding tiles down
  for (const tile of tiles) {
    if (tile.id === movedTileId) continue

    const staticTile = {
      pos_x: tile.pos_x,
      pos_y: tile.pos_y,
      width: tile.width,
      height: tile.height,
    }

    if (tilesOverlap(moving, staticTile)) {
      // Push tile down
      const newY = moving.pos_y + moving.height
      result.push({ id: tile.id, pos_x: tile.pos_x, pos_y: newY })
    }
  }

  return result
}

// Get effective tile position (with preview offset applied)
function getEffectiveTilePosition(tile: Tile) {
  const offset = tileOffsets.value[tile.id]
  if (offset && draggingTileId.value && tile.id !== draggingTileId.value) {
    return {
      x: tile.pos_x + offset.x,
      y: tile.pos_y + offset.y,
    }
  }
  return { x: tile.pos_x, y: tile.pos_y }
}

// Get visualization info for a tile
const vizCache = ref<Record<string, Visualization>>({})
async function getVisualization(vizId: string): Promise<Visualization | null> {
  if (vizCache.value[vizId]) return vizCache.value[vizId]
  try {
    const viz = await visualizationsApi.get(vizId)
    vizCache.value[vizId] = viz
    return viz
  } catch {
    return null
  }
}

onMounted(() => {
  loadDashboard()
})

onBeforeUnmount(() => {
  isActive.value = false
  tilePollToken.value += 1
})

watch(
  () => dashboardId.value,
  (next, prev) => {
    if (next === prev) return
    resetDashboardState()
    loadDashboard()
  },
)

// Clear error on changes
watch(
  () => dashboard.value.name,
  () => {
    error.value = null
  },
)
</script>

<template>
  <AppLayout :title="isNew ? 'New Dashboard' : dashboard.name || 'Dashboard'" back="dashboards">
    <template #header-actions>
      <LButton
        variant="ghost"
        size="sm"
        @click="settingsCollapsed = !settingsCollapsed"
        :class="{ 'text-primary-500': !settingsCollapsed }"
        title="Toggle settings"
      >
        <Settings class="h-4 w-4" />
      </LButton>
      <LButton v-if="!isNew" variant="secondary" @click="openAddTileModal">
        <Plus class="h-4 w-4" />
        Add Tile
      </LButton>
      <LButton :disabled="saving" @click="saveDashboard">
        <Loader2 v-if="saving" class="h-4 w-4 animate-spin" />
        <Save v-else class="h-4 w-4" />
        {{ saving ? 'Saving...' : 'Save' }}
      </LButton>
    </template>

    <!-- Loading state -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <LSpinner size="lg" />
    </div>

    <div v-else class="space-y-4">
      <!-- Error banner -->
      <div
        v-if="error"
        class="flex items-center gap-3 p-3 bg-error-muted text-error rounded-lg text-sm"
      >
        <AlertCircle class="h-5 w-5 shrink-0" />
        <span class="flex-1">{{ error }}</span>
        <button @click="error = null" class="p-1 hover:bg-error/20 rounded">
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Success banner -->
      <div
        v-if="saveSuccess"
        class="flex items-center gap-3 p-3 bg-success-muted text-success rounded-lg text-sm"
      >
        <CheckCircle class="h-5 w-5 shrink-0" />
        <span>Dashboard saved successfully</span>
      </div>

      <!-- Dashboard metadata (only shown when settings expanded) -->
      <LCard v-if="!settingsCollapsed" padding="sm">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Name</label>
            <LInput v-model="dashboard.name" placeholder="My Dashboard" />
          </div>
          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Description (optional)</label>
            <LInput v-model="dashboard.description" placeholder="Dashboard description..." />
          </div>
        </div>
        <div class="mt-4">
          <label class="block text-sm font-medium text-text mb-1.5">Tags</label>
          <LTagsInput
            :model-value="dashboard.tags || []"
            @update:model-value="dashboard.tags = $event"
            placeholder="Add tags for filtering..."
          />
        </div>
      </LCard>

      <!-- Tiles grid -->
      <div v-if="isNew" class="py-12">
        <LEmptyState
          title="Save to add tiles"
          description="Save the dashboard first, then you can add visualization tiles."
        >
          <template #action>
            <LButton @click="saveDashboard" :disabled="saving">
              <Save class="h-4 w-4" />
              Save Dashboard
            </LButton>
          </template>
        </LEmptyState>
      </div>

      <div v-else-if="!dashboard.tiles?.length" class="py-12">
        <LEmptyState
          title="No tiles yet"
          description="Add visualization tiles to build your dashboard."
        >
          <template #action>
            <LButton @click="openAddTileModal">
              <Plus class="h-4 w-4" />
              Add Tile
            </LButton>
          </template>
        </LEmptyState>
      </div>

      <div
        v-else
        class="grid gap-4"
        :style="{
          gridTemplateColumns: `repeat(${GRID_COLS}, 1fr)`,
          gridAutoRows: `${GRID_ROW_HEIGHT}px`,
        }"
      >
        <GridItem
          v-for="tile in dashboard.tiles"
          :key="tile.id"
          :ref="
            (el) => {
              if (el) tileRefs[tile.id] = el
            }
          "
          :x="getEffectiveTilePosition(tile).x"
          :y="getEffectiveTilePosition(tile).y"
          :width="tile.width"
          :height="tile.height"
          :grid-cols="GRID_COLS"
          :row-height="GRID_ROW_HEIGHT"
          :gap="16"
          :min-width="2"
          :min-height="2"
          :show-handle="false"
          :class="{ 'transition-all duration-200': tileOffsets[tile.id] }"
          @preview="(x, y, w, h) => handleTilePreview(tile.id, x, y, w, h)"
          @change="
            (x, y, w, h) => {
              clearTilePreview()
              updateTile(tile, { pos_x: x, pos_y: y, width: w, height: h })
            }
          "
          @drag-end="clearTilePreview"
          @resize-end="clearTilePreview"
        >
          <LCard padding="none" class="h-full overflow-hidden">
            <!-- Tile header with integrated drag handle -->
            <div
              class="absolute top-0 left-0 right-0 flex items-center justify-between px-2 py-1.5 bg-surface/95 backdrop-blur-sm border-b border-border/50 z-10 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <div class="flex items-center gap-1 min-w-0">
                <button
                  type="button"
                  class="p-1 rounded cursor-move hover:bg-surface-sunken text-text-muted transition-colors shrink-0"
                  @mousedown="tileRefs[tile.id]?.startDrag($event)"
                  @touchstart="tileRefs[tile.id]?.startDrag($event)"
                >
                  <GripVertical class="h-4 w-4" />
                </button>
                <span class="text-sm font-medium text-text truncate">
                  {{ tile.title || vizCache[tile.visualization_id]?.name || 'Untitled' }}
                </span>
              </div>
              <button
                type="button"
                class="p-1 rounded hover:bg-surface-sunken text-text-muted hover:text-error transition-colors shrink-0"
                @click="deleteTile(tile.id)"
              >
                <Trash2 class="h-4 w-4" />
              </button>
            </div>

            <!-- Tile content -->
            <div class="h-full p-3 pt-2">
              <VisualizationRenderer
                :chart-type="vizCache[tile.visualization_id]?.chart_type || 'table'"
                :data="tileData[tile.id] || null"
                :config="vizCache[tile.visualization_id]?.config || {}"
                :loading="tileLoading[tile.id]"
                height="100%"
                @vue:mounted="getVisualization(tile.visualization_id)"
              />
            </div>
          </LCard>
        </GridItem>
      </div>
    </div>

    <!-- Add tile modal -->
    <LModal :open="showAddTileModal" title="Add Tile" @close="showAddTileModal = false">
      <div class="space-y-4">
        <p class="text-sm text-text-muted">Select a visualization to add to this dashboard.</p>

        <div v-if="visualizations.length === 0" class="text-center py-4">
          <p class="text-text-muted mb-3">No visualizations found.</p>
          <LButton variant="secondary" @click="router.push({ name: 'visualizations' })">
            Create a Visualization First
          </LButton>
        </div>

        <div v-else class="space-y-2 max-h-64 overflow-y-auto">
          <button
            v-for="viz in visualizations"
            :key="viz.id"
            type="button"
            class="w-full text-left p-3 rounded-lg border transition-colors"
            :class="
              selectedVisualizationId === viz.id
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
                : 'border-border hover:border-border-hover'
            "
            @click="selectedVisualizationId = viz.id"
          >
            <div class="font-medium text-text">{{ viz.name }}</div>
            <div class="text-xs text-text-muted mt-0.5 capitalize">{{ viz.chart_type }}</div>
          </button>
        </div>
      </div>

      <template #footer>
        <LButton variant="secondary" @click="showAddTileModal = false">Cancel</LButton>
        <LButton :disabled="!selectedVisualizationId || addingTile" @click="addTile">
          <Loader2 v-if="addingTile" class="h-4 w-4 animate-spin" />
          Add Tile
        </LButton>
      </template>
    </LModal>
  </AppLayout>
</template>
