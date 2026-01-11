<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'

interface Props {
  headers: string[]
  rows: unknown[][]
  striped?: boolean
  hoverable?: boolean
  compact?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  striped: false,
  hoverable: true,
  compact: false,
})

const tableClasses = computed(() => ['w-full text-left', props.class])

const cellPadding = computed(() => (props.compact ? 'px-3 py-2' : 'px-4 py-3'))
</script>

<template>
  <div class="overflow-x-auto rounded-lg border border-border">
    <table :class="tableClasses">
      <thead class="bg-surface-sunken border-b border-border">
        <tr>
          <th
            v-for="(header, index) in headers"
            :key="index"
            :class="[cellPadding, 'text-sm font-medium text-text-muted whitespace-nowrap']"
          >
            {{ header }}
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-border">
        <tr
          v-for="(row, rowIndex) in rows"
          :key="rowIndex"
          :class="[
            striped && rowIndex % 2 === 1 && 'bg-surface-sunken/50',
            hoverable && 'hover:bg-surface-sunken/70 transition-colors',
          ]"
        >
          <td
            v-for="(cell, cellIndex) in row"
            :key="cellIndex"
            :class="[cellPadding, 'text-sm text-text']"
          >
            <slot :name="`cell-${cellIndex}`" :value="cell" :row="row" :rowIndex="rowIndex">
              {{ cell }}
            </slot>
          </td>
        </tr>
        <tr v-if="rows.length === 0">
          <td
            :colspan="headers.length"
            :class="[cellPadding, 'text-sm text-text-muted text-center']"
          >
            <slot name="empty">No data available</slot>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
