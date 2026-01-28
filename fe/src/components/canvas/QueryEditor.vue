<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import * as monaco from 'monaco-editor'
import { LButton, LSelect, LBadge } from '@/components/ui'
import type { CanvasNode, QueryResult } from '@/types'

interface Props {
  node: CanvasNode
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:sql': [sql: string]
  'update:viz': [viz: string]
}>()

// Split pane state
const splitCollapsed = reactive({ sql: false, viz: false })
const split = reactive({ leftPct: 50 })
let resizing = false

// Viz options
const vizOptions = [
  { value: 'table', label: 'Table' },
  { value: 'line', label: 'Line' },
  { value: 'bar', label: 'Bar' },
  { value: 'stat', label: 'Stat' },
]

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

function formatCell(value: unknown): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

function togglePane(which: 'sql' | 'viz') {
  splitCollapsed[which] = !splitCollapsed[which]
}

function updateViz(value: string | number) {
  emit('update:viz', String(value))
}

// Monaco editor
const monacoHost = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

async function initMonaco() {
  await nextTick()
  const host = monacoHost.value
  if (!host || props.node.type !== 'query') return

  if (!editor) {
    editor = monaco.editor.create(host, {
      value: props.node.meta.sql ?? '',
      language: 'sql',
      theme: 'vs-dark',
      minimap: { enabled: false },
      fontSize: 13,
      automaticLayout: true,
      scrollBeyondLastLine: false,
    })

    editor.onDidChangeModelContent(() => {
      emit('update:sql', editor!.getValue())
    })
  }
}

function formatSql() {
  if (!editor) return
  const formatted = editor.getValue().trim() + '\n'
  editor.setValue(formatted)
  emit('update:sql', formatted)
}

// Resize handling
function startResize(e: PointerEvent) {
  resizing = true
  ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onResizeMove(e: PointerEvent) {
  if (!resizing) return
  const splitEl = document.querySelector('.split-pane') as HTMLElement | null
  if (!splitEl) return

  const rect = splitEl.getBoundingClientRect()
  const pct = ((e.clientX - rect.left) / rect.width) * 100
  split.leftPct = Math.min(80, Math.max(20, pct))
}

function endResize() {
  resizing = false
}

// Watch for SQL changes from parent
watch(
  () => props.node.meta.sql,
  (v) => {
    if (!editor) return
    if (typeof v === 'string' && v !== editor.getValue()) {
      editor.setValue(v)
    }
  },
)

onMounted(() => {
  initMonaco()
  window.addEventListener('pointermove', onResizeMove)
  window.addEventListener('pointerup', endResize)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onResizeMove)
  window.removeEventListener('pointerup', endResize)
  if (editor) {
    editor.getModel()?.dispose()
    editor.dispose()
    editor = null
  }
})
</script>

<template>
  <div
    class="split-pane grid h-full min-h-0"
    :style="{
      gridTemplateColumns: `${split.leftPct}% 10px calc(100% - ${split.leftPct}% - 10px)`,
    }"
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
        <div ref="monacoHost" class="h-full w-full"></div>
      </div>
    </div>

    <!-- Resize handle -->
    <div
      class="cursor-col-resize bg-border-muted hover:bg-border"
      @pointerdown="startResize"
    ></div>

    <!-- Visualization pane -->
    <div class="min-h-0 grid grid-rows-[40px_1fr] overflow-hidden">
      <div
        class="flex items-center justify-between px-2.5 py-2 border-b border-border font-semibold text-xs"
      >
        <span>Visualization</span>
        <div class="flex gap-2 items-center">
          <LSelect
            :model-value="props.node.meta.viz ?? 'table'"
            :options="vizOptions"
            class="w-24"
            @update:model-value="updateViz"
          />
          <LButton variant="ghost" size="sm" @click="togglePane('viz')">
            {{ splitCollapsed.viz ? 'Show' : 'Hide' }}
          </LButton>
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
          class="p-5 flex flex-col items-center justify-center gap-2 text-text-muted text-center"
        >
          <div class="font-bold">{{ vizTitle(props.node.meta.viz) }}</div>
          <div class="text-sm">Run the query to see results</div>
        </div>

        <!-- Results -->
        <div v-else class="p-3 flex flex-col gap-3">
          <div class="flex gap-2 flex-wrap">
            <LBadge>Rows: {{ props.node.meta.result.row_count }}</LBadge>
            <LBadge>Runtime: {{ props.node.meta.result.execution_time_ms }}ms</LBadge>
            <LBadge v-if="props.node.meta.result.truncated" variant="warning">Truncated</LBadge>
          </div>

          <!-- Table visualization -->
          <div
            v-if="props.node.meta.viz === 'table'"
            class="border border-border rounded-xl overflow-hidden"
          >
            <div
              class="grid gap-2.5 p-2.5 font-bold bg-surface-sunken border-b border-border text-xs"
              :style="{
                gridTemplateColumns: `repeat(${(props.node.meta.result as QueryResult).columns.length}, minmax(80px, 1fr))`,
              }"
            >
              <span
                v-for="col in (props.node.meta.result as QueryResult).columns"
                :key="col.name"
                class="font-mono"
              >
                {{ col.name }}
              </span>
            </div>
            <div
              v-for="(row, idx) in (props.node.meta.result as QueryResult).rows.slice(0, 50)"
              :key="idx"
              class="grid gap-2.5 p-2.5 border-b border-border-muted last:border-b-0 text-xs"
              :style="{
                gridTemplateColumns: `repeat(${(props.node.meta.result as QueryResult).columns.length}, minmax(80px, 1fr))`,
              }"
            >
              <span v-for="(cell, cidx) in row" :key="cidx" class="font-mono truncate">
                {{ formatCell(cell) }}
              </span>
            </div>
            <div
              v-if="(props.node.meta.result as QueryResult).rows.length > 50"
              class="p-2.5 text-xs text-text-muted text-center"
            >
              ... and {{ (props.node.meta.result as QueryResult).rows.length - 50 }} more rows
            </div>
          </div>

          <!-- Chart placeholder for non-table -->
          <div
            v-else
            class="p-5 flex flex-col items-center justify-center gap-2 text-text-muted text-center"
          >
            <div class="font-bold">{{ vizTitle(props.node.meta.viz) }}</div>
            <div class="text-sm">
              Chart rendering coming soon. Data available:
              {{ props.node.meta.result.row_count }} rows
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
