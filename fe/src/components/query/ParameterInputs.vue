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

function getDefaultForType(type: ParamType): unknown {
  switch (type) {
    case 'number':
      return 0
    case 'boolean':
      return false
    case 'date':
    case 'datetime':
      return new Date().toISOString().split('T')[0]
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
        :model-value="String(modelValue[param.name] ?? '')"
        @update:model-value="updateValue(param.name, $event, param.param_type)"
      />

      <LInput
        v-else-if="param.param_type === 'datetime'"
        type="datetime-local"
        :model-value="String(modelValue[param.name] ?? '')"
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
