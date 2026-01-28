<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { LButton, LSelect, LBadge, LInput } from '@/components/ui'
import { VisualizationRenderer } from '@/components/charts'
import SqlEditor from '@/components/editor/SqlEditor.vue'
import type { CanvasNode, VisualizationConfig, ChartType } from '@/types'

interface Props {
  node: CanvasNode
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:sql': [sql: string]
  'update:viz': [viz: ChartType]
  'update:vizConfig': [config: VisualizationConfig]
  run: []
}>()

// Split pane state
const splitCollapsed = reactive({ sql: false, viz: false })
const split = reactive({ leftPct: 50 })
const isResizing = ref(false)

// SQL Editor ref
const sqlEditorRef = ref<InstanceType<typeof SqlEditor> | null>(null)

// Viz options - map to ChartType
const vizOptions: { value: ChartType; label: string }[] = [
  { value: 'table', label: 'Table' },
  { value: 'line', label: 'Line' },
  { value: 'bar', label: 'Bar' },
  { value: 'single_stat', label: 'Stat' },
  { value: 'pie', label: 'Pie' },
]

// Current viz type
const currentViz = computed(() => props.node.meta.viz ?? 'table')
const currentConfig = computed(() => props.node.meta.vizConfig ?? {})

// Column options from result
const columnOptions = computed(() => {
  const result = props.node.meta.result
  if (!result) return []
  return [
    { value: '', label: 'Select column...' },
    ...result.columns.map((col) => ({
      value: col.name,
      label: `${col.name} (${col.data_type})`,
    })),
  ]
})

function updateViz(value: string | number) {
  emit('update:viz', String(value) as ChartType)
}

function updateSql(value: string) {
  emit('update:sql', value)
}

function updateConfig(key: keyof VisualizationConfig, value: unknown) {
  emit('update:vizConfig', {
    ...currentConfig.value,
    [key]: value || undefined,
  })
}

function formatSql() {
  sqlEditorRef.value?.format()
}

function togglePane(which: 'sql' | 'viz') {
  splitCollapsed[which] = !splitCollapsed[which]
}

// Resize handling
function startResize(e: PointerEvent) {
  isResizing.value = true
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onResizeMove(e: PointerEvent) {
  if (!isResizing.value) return
  const splitEl = document.querySelector('.split-pane') as HTMLElement | null
  if (!splitEl) return

  const rect = splitEl.getBoundingClientRect()
  const pct = ((e.clientX - rect.left) / rect.width) * 100
  split.leftPct = Math.min(80, Math.max(20, pct))
}

function endResize(e: PointerEvent) {
  if (isResizing.value) {
    isResizing.value = false
    ;(e.target as HTMLElement).releasePointerCapture?.(e.pointerId)
  }
}
</script>

<template>
  <div
    class="split-pane grid h-full min-h-0"
    :class="{ 'select-none': isResizing }"
    :style="{
      gridTemplateColumns: `${split.leftPct}% 10px calc(100% - ${split.leftPct}% - 10px)`,
    }"
    @pointermove="onResizeMove"
    @pointerup="endResize"
  >
    <!-- SQL Editor pane -->
    <div class="min-h-0 grid grid-rows-[40px_1fr] overflow-hidden">
      <div
        class="flex items-center justify-between px-2.5 py-2 border-b border-border font-semibold text-xs"
      >
        <span>SQL Editor</span>
        <div class="flex gap-2 items-center">
          <LButton variant="ghost" size="sm" @click="formatSql">Format</LButton>
          <LButton variant="ghost" size="sm" @click="togglePane('sql')">
            {{ splitCollapsed.sql ? 'Show' : 'Hide' }}
          </LButton>
        </div>
      </div>

      <div v-show="!splitCollapsed.sql" class="min-h-0 overflow-hidden">
        <SqlEditor
          ref="sqlEditorRef"
          :model-value="props.node.meta.sql ?? ''"
          :minimap="false"
          :line-numbers="true"
          height="100%"
          class="h-full border-0 rounded-none"
          @update:model-value="updateSql"
          @run="$emit('run')"
        />
      </div>
    </div>

    <!-- Resize handle -->
    <div
      class="cursor-col-resize bg-border-muted hover:bg-primary-500 transition-colors"
      :class="{ 'bg-primary-500': isResizing }"
      @pointerdown="startResize"
    />

    <!-- Visualization pane -->
    <div class="min-h-0 grid grid-rows-[auto_1fr] overflow-hidden">
      <div class="border-b border-border">
        <!-- Viz header -->
        <div
          class="flex items-center justify-between px-2.5 py-2 font-semibold text-xs"
        >
          <span>Visualization</span>
          <div class="flex gap-2 items-center">
            <LSelect
              :model-value="currentViz"
              :options="vizOptions"
              class="w-24"
              @update:model-value="updateViz"
            />
            <LButton variant="ghost" size="sm" @click="togglePane('viz')">
              {{ splitCollapsed.viz ? 'Show' : 'Hide' }}
            </LButton>
          </div>
        </div>

        <!-- Chart config (when not table) -->
        <div
          v-if="currentViz !== 'table' && props.node.meta.result"
          class="px-2.5 pb-2.5 space-y-2"
        >
          <!-- Line/Bar config -->
          <template v-if="currentViz === 'line' || currentViz === 'bar'">
            <div class="grid grid-cols-3 gap-2">
              <div>
                <label class="block text-[10px] text-text-muted mb-1">X-Axis</label>
                <LSelect
                  :model-value="currentConfig.x_axis ?? ''"
                  :options="columnOptions"
                  class="text-xs"
                  @update:model-value="updateConfig('x_axis', $event)"
                />
              </div>
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Y-Axis</label>
                <LSelect
                  :model-value="currentConfig.y_axis ?? ''"
                  :options="columnOptions"
                  class="text-xs"
                  @update:model-value="updateConfig('y_axis', $event)"
                />
              </div>
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Series</label>
                <LSelect
                  :model-value="currentConfig.series_column ?? ''"
                  :options="columnOptions"
                  class="text-xs"
                  @update:model-value="updateConfig('series_column', $event)"
                />
              </div>
            </div>
          </template>

          <!-- Single stat config -->
          <template v-if="currentViz === 'single_stat'">
            <div class="grid grid-cols-3 gap-2">
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Value</label>
                <LSelect
                  :model-value="currentConfig.value_column ?? ''"
                  :options="columnOptions"
                  class="text-xs"
                  @update:model-value="updateConfig('value_column', $event)"
                />
              </div>
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Prefix</label>
                <LInput
                  :model-value="currentConfig.prefix ?? ''"
                  class="text-xs h-8"
                  placeholder="$"
                  @update:model-value="updateConfig('prefix', $event)"
                />
              </div>
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Suffix</label>
                <LInput
                  :model-value="currentConfig.suffix ?? ''"
                  class="text-xs h-8"
                  placeholder="%"
                  @update:model-value="updateConfig('suffix', $event)"
                />
              </div>
            </div>
          </template>

          <!-- Pie config -->
          <template v-if="currentViz === 'pie'">
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Label</label>
                <LSelect
                  :model-value="currentConfig.label_column ?? ''"
                  :options="columnOptions"
                  class="text-xs"
                  @update:model-value="updateConfig('label_column', $event)"
                />
              </div>
              <div>
                <label class="block text-[10px] text-text-muted mb-1">Value</label>
                <LSelect
                  :model-value="currentConfig.value_column ?? ''"
                  :options="columnOptions"
                  class="text-xs"
                  @update:model-value="updateConfig('value_column', $event)"
                />
              </div>
            </div>
          </template>
        </div>
      </div>

      <div v-show="!splitCollapsed.viz" class="min-h-0 overflow-auto">
        <!-- Error state -->
        <div v-if="props.node.meta.error" class="p-5 text-center">
          <div class="font-bold text-error mb-2">Error</div>
          <div class="text-sm text-error/70">{{ props.node.meta.error }}</div>
        </div>

        <!-- No results yet -->
        <div
          v-else-if="!props.node.meta.result"
          class="p-5 flex flex-col items-center justify-center gap-2 text-text-muted text-center h-full"
        >
          <div class="text-sm">Run the query to see results</div>
        </div>

        <!-- Results with VisualizationRenderer -->
        <div v-else class="h-full flex flex-col">
          <div class="flex gap-2 flex-wrap px-3 pt-3">
            <LBadge>Rows: {{ props.node.meta.result.row_count }}</LBadge>
            <LBadge>Runtime: {{ props.node.meta.result.execution_time_ms }}ms</LBadge>
            <LBadge v-if="props.node.meta.result.truncated" variant="warning">Truncated</LBadge>
          </div>

          <div class="flex-1 min-h-0 p-3">
            <VisualizationRenderer
              :chart-type="currentViz"
              :data="props.node.meta.result"
              :config="currentConfig"
              height="100%"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.split-pane :deep(.rounded-md) {
  border-radius: 0;
}
.split-pane :deep(.border) {
  border: none;
}
</style>
