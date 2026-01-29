<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { Splitpanes, Pane } from 'splitpanes'
import 'splitpanes/dist/splitpanes.css'
import { LButton, LSelect, LBadge, LInput } from '@/components/ui'
import {
  Columns2,
  Rows2,
  WandSparkles,
  Code2,
  BarChart3,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  ChevronDown,
} from 'lucide-vue-next'
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

// Split pane state - load from localStorage
const STORAGE_KEY = 'loupe:canvas:queryEditor:layout'

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
const collapsed = reactive({ sql: false, viz: false })
const splitDirection = ref<'horizontal' | 'vertical'>(storedLayout?.splitDirection ?? 'horizontal')
const splitPct = ref(storedLayout?.splitPct ?? 50)

// Persist layout changes
watch([splitDirection, splitPct], () => {
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({
      splitDirection: splitDirection.value,
      splitPct: splitPct.value,
    }),
  )
})

function toggleSplitDirection() {
  splitDirection.value = splitDirection.value === 'horizontal' ? 'vertical' : 'horizontal'
}

function onPaneResized(panes: { size: number }[]) {
  if (panes[0] && !collapsed.sql && !collapsed.viz) {
    splitPct.value = panes[0].size
  }
}

// Computed pane sizes based on collapsed state
const sqlPaneSize = computed(() => {
  if (collapsed.sql) return 0
  if (collapsed.viz) return 100
  return splitPct.value
})

const vizPaneSize = computed(() => {
  if (collapsed.viz) return 0
  if (collapsed.sql) return 100
  return 100 - splitPct.value
})

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
  // Don't allow both to be collapsed
  if (!collapsed[which] && collapsed[which === 'sql' ? 'viz' : 'sql']) {
    return
  }
  collapsed[which] = !collapsed[which]
}

// Collapse direction icons based on split direction
const sqlCollapseIcon = computed(() => {
  if (collapsed.sql) {
    return splitDirection.value === 'horizontal' ? ChevronRight : ChevronDown
  }
  return splitDirection.value === 'horizontal' ? ChevronLeft : ChevronUp
})

const vizCollapseIcon = computed(() => {
  if (collapsed.viz) {
    return splitDirection.value === 'horizontal' ? ChevronLeft : ChevronUp
  }
  return splitDirection.value === 'horizontal' ? ChevronRight : ChevronDown
})
</script>

<template>
  <div class="h-full w-full grid grid-rows-[36px_1fr] min-h-0 overflow-hidden">
    <!-- Shared toolbar -->
    <div
      class="w-full flex items-center justify-between px-2.5 border-b border-border bg-surface-raised text-xs overflow-visible"
    >
      <!-- SQL controls -->
      <div class="flex items-center gap-1.5 shrink-0">
        <LButton variant="ghost" size="sm" title="Toggle SQL Editor" @click="togglePane('sql')">
          <component :is="sqlCollapseIcon" :size="14" class="text-text-subtle" />
          <Code2 :size="14" />
          <span class="font-medium">SQL</span>
        </LButton>
        <Transition name="fade" appear>
          <LButton
            v-if="!collapsed.sql"
            variant="ghost"
            size="sm"
            title="Format SQL"
            @click="formatSql"
          >
            <WandSparkles :size="14" />
          </LButton>
        </Transition>
      </div>

      <!-- Center controls -->
      <LButton
        variant="text"
        class="shrink-0"
        :title="
          splitDirection === 'horizontal'
            ? 'Switch to Vertical Split'
            : 'Switch to Horizontal Split'
        "
        @click="toggleSplitDirection"
      >
        <Columns2 v-if="splitDirection === 'horizontal'" :size="14" />
        <Rows2 v-else :size="14" />
      </LButton>

      <!-- Viz controls -->
      <div class="flex items-center gap-1.5 shrink-0">
        <Transition name="fade" appear>
          <LSelect
            v-if="!collapsed.viz"
            :model-value="currentViz"
            :options="vizOptions"
            size="xs"
            class="w-24 text-xs"
            @update:model-value="updateViz"
          />
        </Transition>
        <LButton variant="ghost" size="sm" title="Toggle Visualization" @click="togglePane('viz')">
          <BarChart3 :size="14" />
          <span class="font-medium">Viz</span>
          <component :is="vizCollapseIcon" :size="14" class="text-text-subtle" />
        </LButton>
      </div>
    </div>

    <!-- Split panes -->
    <Splitpanes
      class="default-theme query-editor-split min-h-0 w-full overflow-hidden"
      :horizontal="splitDirection === 'vertical'"
      @resized="onPaneResized"
    >
      <!-- SQL Editor pane -->
      <Pane v-if="!collapsed.sql" :size="sqlPaneSize" :min-size="20">
        <div class="h-full overflow-hidden bg-surface-raised">
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
      </Pane>

      <!-- Visualization pane -->
      <Pane v-if="!collapsed.viz" :size="vizPaneSize" :min-size="20">
        <div class="h-full flex flex-col overflow-hidden bg-surface-raised">
          <!-- Chart config (when not table) -->
          <div
            v-if="currentViz !== 'table' && props.node.meta.result"
            class="border-b border-border px-2.5 py-2 space-y-2 max-h-32 overflow-y-auto shrink-0"
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

          <div class="flex-1 min-h-0 overflow-auto">
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
            <div v-else class="h-full flex flex-col min-h-0">
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
      </Pane>
    </Splitpanes>
  </div>
</template>

<style scoped>
.query-editor-split :deep(.rounded-md) {
  border-radius: 0;
}
.query-editor-split :deep(.border) {
  border: none;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
