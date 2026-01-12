<script setup lang="ts">
import { watch } from 'vue'
import { LInput, LSelect } from '@/components/ui'
import type { ParamDef, ParamType } from '@/types'

const props = defineProps<{
  parameters: ParamDef[]
  modelValue: Record<string, unknown>
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: Record<string, unknown>): void
}>()

// Initialize values with defaults
watch(
  () => props.parameters,
  (params) => {
    const current = { ...props.modelValue }
    let changed = false

    for (const param of params) {
      if (!(param.name in current)) {
        current[param.name] = param.default ?? getDefaultForType(param.param_type)
        changed = true
      }
    }

    if (changed) {
      emit('update:modelValue', current)
    }
  },
  { immediate: true },
)

function formatDateInputValue(value: unknown): string {
  if (value instanceof Date) return toDateInputValue(value)
  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (!trimmed) return ''
    const direct = trimmed.match(/^(\d{4})-(\d{2})-(\d{2})$/)
    if (direct) return `${direct[1]}-${direct[2]}-${direct[3]}`
    const parsed = new Date(trimmed)
    if (Number.isNaN(parsed.getTime())) return ''
    return toDateInputValue(parsed)
  }
  return ''
}

function formatDateTimeInputValue(value: unknown): string {
  if (value instanceof Date) return toDateTimeInputValue(value)
  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (!trimmed) return ''
    const direct = trimmed.match(/^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})/)
    if (direct && !/[zZ]|[+-]\d{2}:?\d{2}$/.test(trimmed)) {
      return `${direct[1]}-${direct[2]}-${direct[3]}T${direct[4]}:${direct[5]}`
    }
    const parsed = new Date(trimmed)
    if (Number.isNaN(parsed.getTime())) return ''
    return toDateTimeInputValue(parsed)
  }
  return ''
}

function toDateInputValue(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function toDateTimeInputValue(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  return `${year}-${month}-${day}T${hours}:${minutes}`
}

function coerceDateTimeValue(value: unknown): string {
  if (value instanceof Date) return value.toISOString()
  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (!trimmed) return ''
    const direct = trimmed.match(
      /^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})(?::(\d{2}))?$/,
    )
    if (direct) {
      const date = new Date(
        Number(direct[1]),
        Number(direct[2]) - 1,
        Number(direct[3]),
        Number(direct[4]),
        Number(direct[5]),
        Number(direct[6] || 0),
      )
      return date.toISOString()
    }
    const parsed = new Date(trimmed)
    if (!Number.isNaN(parsed.getTime())) return parsed.toISOString()
    return trimmed
  }
  return ''
}

function getDefaultForType(type: ParamType): unknown {
  switch (type) {
    case 'number':
      return 0
    case 'boolean':
      return false
    case 'date':
      return toDateInputValue(new Date())
    case 'datetime':
      return new Date().toISOString()
    default:
      return ''
  }
}

function updateValue(name: string, value: unknown, type: ParamType) {
  let coerced = value

  // Coerce to correct type
  if (type === 'number') {
    coerced = Number(value) || 0
  } else if (type === 'boolean') {
    coerced = value === true || value === 'true'
  } else if (type === 'datetime') {
    coerced = coerceDateTimeValue(value)
  }

  emit('update:modelValue', {
    ...props.modelValue,
    [name]: coerced,
  })
}

const boolOptions = [
  { value: 'true', label: 'True' },
  { value: 'false', label: 'False' },
]
</script>

<template>
  <div
    v-if="parameters.length > 0"
    class="flex flex-wrap items-end gap-3 p-3 bg-surface-sunken rounded-lg"
  >
    <div v-for="param in parameters" :key="param.name" class="min-w-30 max-w-50">
      <label class="block text-xs text-text-muted mb-1 truncate" :title="param.name">
        {{ param.name }}
        <span v-if="param.required" class="text-error">*</span>
      </label>

      <!-- Boolean: dropdown -->
      <LSelect
        v-if="param.param_type === 'boolean'"
        :model-value="String(modelValue[param.name] ?? false)"
        @update:model-value="updateValue(param.name, $event === 'true', param.param_type)"
        :options="boolOptions"
      />

      <!-- Date/DateTime: date input -->
      <LInput
        v-else-if="param.param_type === 'date'"
        type="date"
        :model-value="formatDateInputValue(modelValue[param.name])"
        @update:model-value="updateValue(param.name, $event, param.param_type)"
      />

      <LInput
        v-else-if="param.param_type === 'datetime'"
        type="datetime-local"
        :model-value="formatDateTimeInputValue(modelValue[param.name])"
        @update:model-value="updateValue(param.name, $event, param.param_type)"
      />

      <!-- Number: number input -->
      <LInput
        v-else-if="param.param_type === 'number'"
        type="number"
        :model-value="String(modelValue[param.name] ?? 0)"
        @update:model-value="updateValue(param.name, $event, param.param_type)"
      />

      <!-- String: text input -->
      <LInput
        v-else
        :model-value="String(modelValue[param.name] ?? '')"
        @update:model-value="updateValue(param.name, $event, param.param_type)"
        :placeholder="param.default ? String(param.default) : ''"
      />
    </div>
  </div>
</template>
