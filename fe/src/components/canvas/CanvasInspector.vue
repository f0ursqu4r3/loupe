<script setup lang="ts">
import { computed } from 'vue'
import { useCanvasStore } from '@/stores/canvas'
import { LButton, LSelect, LEmptyState } from '@/components/ui'
import { Play, Crosshair } from 'lucide-vue-next'
import QueryEditor from './QueryEditor.vue'
import NoteEditor from './NoteEditor.vue'
import type { CanvasNode } from '@/types/canvas'
import type { Datasource } from '@/types/api'

interface Props {
  node: CanvasNode | null
  datasources: Datasource[]
  isRunning: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:title': [title: string]
  'update:datasource': [id: string]
  'update:sql': [sql: string]
  'update:viz': [viz: string]
  'update:vizConfig': [config: Record<string, unknown>]
  'update:note-text': [text: string]
  run: []
  center: []
}>()

const canvasStore = useCanvasStore()

const timeRangeLabel = computed(() => {
  const preset = canvasStore.activeCanvas?.timeRange.preset ?? '7d'
  const live = canvasStore.activeCanvas?.live ? ' (live)' : ''
  return `Last ${preset}${live}`
})

const datasourceOptions = computed(() => [
  { value: '', label: 'Select datasource...' },
  ...props.datasources.map((ds) => ({ value: ds.id, label: ds.name })),
])

function updateTitle(event: Event) {
  emit('update:title', (event.target as HTMLInputElement).value)
}

function updateDatasource(value: string | number) {
  emit('update:datasource', String(value))
}
</script>

<template>
  <aside
    class="inspector border-l border-border bg-surface-raised grid grid-rows-[auto_auto_1fr] min-h-0 overflow-hidden"
  >
    <LEmptyState
      v-if="!props.node"
      title="No selection"
      description="Select a node to edit SQL, view results, or add notes."
      class="py-8"
    />

    <template v-else>
      <!-- Header -->
      <div class="p-3 flex items-center justify-between gap-3 border-b border-border">
        <div class="flex items-center gap-2.5 min-w-0 flex-1">
          <span
            class="w-2.5 h-2.5 rounded-full shrink-0"
            :class="props.node.type === 'query' ? 'bg-info' : 'bg-warning'"
          ></span>
          <input
            class="w-full min-w-0 font-bold text-sm bg-transparent rounded px-1.5 py-0.5 -ml-1.5 border border-transparent hover:border-border focus:border-primary-500 focus:bg-surface outline-none transition-colors"
            :value="props.node.title"
            @input="updateTitle"
            spellcheck="false"
            placeholder="Untitled Query"
          />
        </div>

        <div class="flex gap-2">
          <LButton
            v-if="props.node.type === 'query'"
            variant="primary"
            :loading="props.isRunning"
            @click="$emit('run')"
          >
            <Play class="h-4 w-4" />
            Run
          </LButton>
          <LButton variant="secondary" @click="$emit('center')">
            <Crosshair class="h-4 w-4" />
            Center
          </LButton>
        </div>
      </div>

      <!-- Meta info -->
      <div class="grid grid-cols-2 gap-2.5 p-3 border-b border-border">
        <div class="grid gap-1">
          <div class="text-[11px] text-text-subtle">Global time</div>
          <div class="text-xs">
            {{ timeRangeLabel }}
            <span v-if="canvasStore.activeCanvas?.live" class="text-text-muted">(live)</span>
          </div>
        </div>
        <div v-if="props.node.type === 'query'" class="grid gap-1">
          <div class="text-[11px] text-text-subtle">Source</div>
          <LSelect
            :model-value="props.node.meta.datasourceId ?? ''"
            :options="datasourceOptions"
            size="xs"
            @update:model-value="updateDatasource"
          />
        </div>
        <div class="grid gap-1">
          <div class="text-[11px] text-text-subtle">Node id</div>
          <div class="text-xs font-mono">{{ props.node.id }}</div>
        </div>
      </div>

      <!-- Query Editor -->
      <QueryEditor
        v-if="props.node.type === 'query'"
        :node="props.node"
        class="min-h-0"
        @update:sql="$emit('update:sql', $event)"
        @update:viz="$emit('update:viz', $event)"
        @update:vizConfig="$emit('update:vizConfig', $event)"
        @run="$emit('run')"
      />

      <!-- Note Editor -->
      <NoteEditor
        v-else
        :text="props.node.meta.text ?? ''"
        @update:text="$emit('update:note-text', $event)"
      />
    </template>
  </aside>
</template>
