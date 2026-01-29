<script setup lang="ts">
import { computed } from 'vue'
import { LBadge } from '@/components/ui'
import { Link, Copy, X } from 'lucide-vue-next'
import type { CanvasNode } from '@/types/canvas'

interface Props {
  node: CanvasNode
  selected: boolean
  timeRangeLabel: string
  datasourceName: string
}

const props = defineProps<Props>()

defineEmits<{
  connect: [id: string]
  duplicate: [id: string]
  delete: [id: string]
}>()

const nodeStyle = computed(() => ({
  left: `${props.node.x}px`,
  top: `${props.node.y}px`,
  width: `${props.node.w}px`,
  height: `${props.node.h}px`,
}))

const statusClass = computed(() => {
  switch (props.node.meta.status) {
    case 'ok':
      return 'text-success'
    case 'warn':
      return 'text-warning'
    case 'error':
      return 'text-error'
    case 'running':
      return 'text-info'
    default:
      return 'text-text-subtle'
  }
})
</script>

<template>
  <div
    data-canvas-node
    class="absolute rounded-xl border bg-surface-raised shadow-lg grid grid-rows-[36px_1fr_32px] select-none cursor-grab"
    :class="{
      'border-border-strong ring-2 ring-border': props.selected,
      'border-border': !props.selected,
    }"
    :style="nodeStyle"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-2.5 py-2 border-b border-border">
      <div
        class="flex items-center gap-2 font-semibold text-sm overflow-hidden text-ellipsis whitespace-nowrap"
      >
        <span
          class="w-2.5 h-2.5 rounded-full shrink-0"
          :class="props.node.type === 'query' ? 'bg-info' : 'bg-warning'"
        ></span>
        <span>{{ props.node.title }}</span>
      </div>
      <div class="flex gap-1.5">
        <button
          class="w-6 h-6 rounded-lg border border-border bg-surface hover:bg-surface-sunken text-xs flex items-center justify-center"
          title="Connect"
          @click.stop="$emit('connect', props.node.id)"
        >
          <Link class="h-3.5 w-3.5" />
        </button>
        <button
          class="w-6 h-6 rounded-lg border border-border bg-surface hover:bg-surface-sunken text-xs flex items-center justify-center"
          title="Duplicate"
          @click.stop="$emit('duplicate', props.node.id)"
        >
          <Copy class="h-3.5 w-3.5" />
        </button>
        <button
          class="w-6 h-6 rounded-lg border border-border bg-surface hover:bg-surface-sunken text-xs flex items-center justify-center"
          title="Delete"
          @click.stop="$emit('delete', props.node.id)"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>

    <!-- Body -->
    <div class="p-2.5 min-h-0 flex flex-col gap-2.5 overflow-hidden">
      <template v-if="props.node.type === 'query'">
        <div class="grid gap-1.5 text-xs">
          <div class="flex justify-between gap-2.5">
            <span class="text-text-muted">Data source</span>
            <span>{{ props.datasourceName }}</span>
          </div>
          <div class="flex justify-between gap-2.5">
            <span class="text-text-muted">Last run</span>
            <span>{{ props.node.meta.lastRun || 'never' }}</span>
          </div>
          <div class="flex justify-between gap-2.5">
            <span class="text-text-muted">Status</span>
            <span :class="statusClass">
              {{ props.node.meta.status || 'idle' }}
            </span>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="text-xs leading-snug whitespace-pre-wrap overflow-hidden">
          {{ props.node.meta.text || '(empty note)' }}
        </div>
      </template>
    </div>

    <!-- Footer -->
    <div class="flex gap-2 items-center px-2.5 py-2 border-t border-border overflow-hidden">
      <LBadge size="sm">{{ props.timeRangeLabel }}</LBadge>
      <LBadge v-if="props.node.type === 'query'" variant="info" size="sm">SQL</LBadge>
      <LBadge v-if="props.node.type === 'note'" variant="warning" size="sm">NOTE</LBadge>
    </div>
  </div>
</template>
