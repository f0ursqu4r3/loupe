<script setup lang="ts">
import { computed } from 'vue'
import { Plus, Trash2, AlertTriangle } from 'lucide-vue-next'
import { LButton, LInput, LSelect } from '@/components/ui'
import type { ParamDef, ParamType } from '@/types'

const props = defineProps<{
  modelValue: ParamDef[]
  sql: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: ParamDef[]): void
}>()

// Param type options
const paramTypeOptions = [
  { value: 'string', label: 'String' },
  { value: 'number', label: 'Number' },
  { value: 'boolean', label: 'Boolean' },
  { value: 'date', label: 'Date' },
  { value: 'datetime', label: 'DateTime' },
]

// Extract parameters from SQL using {{param}} syntax
const detectedParams = computed(() => {
  const regex = /\{\{\s*(\w+)\s*\}\}/g
  const params = new Set<string>()
  let match
  while ((match = regex.exec(props.sql)) !== null) {
    if (match[1]) {
      params.add(match[1])
    }
  }
  return Array.from(params)
})

// Find missing parameters (in SQL but not defined)
const missingParams = computed(() => {
  const definedNames = new Set(props.modelValue.map((p) => p.name))
  return detectedParams.value.filter((name) => !definedNames.has(name))
})

// Find unused parameters (defined but not in SQL)
const unusedParams = computed(() => {
  const detected = new Set(detectedParams.value)
  return props.modelValue.filter((p) => !detected.has(p.name))
})

// Add a new parameter
function addParameter(name?: string) {
  const newParam: ParamDef = {
    name: name || `param${props.modelValue.length + 1}`,
    param_type: 'string',
    required: true,
    default: undefined,
  }
  emit('update:modelValue', [...props.modelValue, newParam])
}

// Remove a parameter
function removeParameter(index: number) {
  const updated = [...props.modelValue]
  updated.splice(index, 1)
  emit('update:modelValue', updated)
}

// Update a parameter field
function updateParameter(index: number, field: keyof ParamDef, value: unknown) {
  const updated = [...props.modelValue]
  const current = updated[index]
  if (!current) return
  updated[index] = {
    name: field === 'name' ? (value as string) : current.name,
    param_type: field === 'param_type' ? (value as ParamType) : current.param_type,
    default: field === 'default' ? value : current.default,
    required: field === 'required' ? (value as boolean) : current.required,
  }
  emit('update:modelValue', updated)
}

// Auto-add missing parameters
function addMissingParams() {
  const newParams = missingParams.value.map(
    (name): ParamDef => ({
      name,
      param_type: 'string',
      required: true,
      default: undefined,
    }),
  )
  emit('update:modelValue', [...props.modelValue, ...newParams])
}
</script>

<template>
  <div class="space-y-4">
    <!-- Warnings -->
    <div
      v-if="missingParams.length > 0"
      class="flex items-start gap-2 p-3 bg-warning-muted text-warning rounded-lg text-sm"
    >
      <AlertTriangle class="h-4 w-4 shrink-0 mt-0.5" />
      <div class="flex-1">
        <p class="font-medium">Missing parameter definitions</p>
        <p class="text-xs mt-1">
          Found in SQL but not defined:
          <code v-for="(p, i) in missingParams" :key="p" class="bg-warning/20 px-1 rounded">
            {{ p }}{{ i < missingParams.length - 1 ? ', ' : '' }}
          </code>
        </p>
        <LButton variant="ghost" size="sm" class="mt-2 -ml-2" @click="addMissingParams">
          <Plus class="h-3.5 w-3.5" />
          Add missing parameters
        </LButton>
      </div>
    </div>

    <!-- Parameter list -->
    <div v-if="modelValue.length > 0" class="space-y-3">
      <div
        v-for="(param, index) in modelValue"
        :key="index"
        class="grid grid-cols-[1fr_120px_100px_auto] gap-2 items-start p-3 bg-surface-sunken rounded-lg"
        :class="{ 'opacity-60': unusedParams.includes(param) }"
      >
        <div>
          <label class="block text-xs text-text-muted mb-1">Name</label>
          <LInput
            :model-value="param.name"
            @update:model-value="updateParameter(index, 'name', $event)"
            placeholder="param_name"
            class="font-mono text-sm"
          />
          <p v-if="unusedParams.includes(param)" class="text-xs text-warning mt-1">
            Not used in SQL
          </p>
        </div>

        <div>
          <label class="block text-xs text-text-muted mb-1">Type</label>
          <LSelect
            :model-value="param.param_type"
            @update:model-value="updateParameter(index, 'param_type', $event as ParamType)"
            :options="paramTypeOptions"
          />
        </div>

        <div>
          <label class="block text-xs text-text-muted mb-1">Default</label>
          <LInput
            :model-value="String(param.default ?? '')"
            @update:model-value="updateParameter(index, 'default', $event || undefined)"
            placeholder="—"
          />
        </div>

        <div class="pt-5">
          <LButton variant="ghost" size="sm" @click="removeParameter(index)">
            <Trash2 class="h-4 w-4 text-error" />
          </LButton>
        </div>
      </div>
    </div>

    <!-- Empty state / Add button -->
    <div class="flex items-center gap-3">
      <LButton variant="secondary" size="sm" @click="addParameter()">
        <Plus class="h-4 w-4" />
        Add Parameter
      </LButton>
      <p v-if="modelValue.length === 0" class="text-xs text-text-muted">
        Use <code class="bg-surface-sunken px-1 rounded">{<!-- -->{param_name}}</code> syntax in SQL
        to reference parameters
      </p>
    </div>
  </div>
</template>
