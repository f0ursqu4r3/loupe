<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Trash2, Layout, Pencil, Check, X } from 'lucide-vue-next'
import { AppLayout } from '@/components/layout'
import { LButton, LCard, LEmptyState, LInput, LModal } from '@/components/ui'
import { useCanvasStore } from '@/stores/canvas'
import { formatDateShort } from '@/utils/dateTime'
import { ref } from 'vue'

const router = useRouter()
const canvasStore = useCanvasStore()

const canvases = computed(() => canvasStore.canvases)

// Editing state
const editingId = ref<string | null>(null)
const editingName = ref('')

// Delete confirmation modal
const showDeleteModal = ref(false)
const canvasToDelete = ref<{ id: string; name: string } | null>(null)

function openCanvas(id: string) {
  canvasStore.setActiveCanvas(id)
  router.push({ name: 'canvas-editor', params: { id } })
}

function createCanvas() {
  const canvas = canvasStore.createCanvas()
  router.push({ name: 'canvas-editor', params: { id: canvas.id } })
}

function deleteCanvas(id: string, event: Event) {
  event.stopPropagation()
  const canvas = canvases.value.find((c) => c.id === id)
  if (canvas) {
    canvasToDelete.value = { id: canvas.id, name: canvas.name }
    showDeleteModal.value = true
  }
}

function confirmDelete() {
  if (!canvasToDelete.value) return
  canvasStore.deleteCanvas(canvasToDelete.value.id)
  showDeleteModal.value = false
  canvasToDelete.value = null
}

function startEditing(id: string, name: string, event: Event) {
  event.stopPropagation()
  editingId.value = id
  editingName.value = name
}

function saveEditing(event: Event) {
  event.stopPropagation()
  if (editingId.value && editingName.value.trim()) {
    canvasStore.renameCanvas(editingId.value, editingName.value.trim())
  }
  cancelEditing()
}

function cancelEditing(event?: Event) {
  event?.stopPropagation()
  editingId.value = null
  editingName.value = ''
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    saveEditing(event)
  } else if (event.key === 'Escape') {
    cancelEditing()
  }
}
</script>

<template>
  <AppLayout title="Canvases">
    <template #header-actions>
      <LButton @click="createCanvas">
        <Plus :size="16" />
        New Canvas
      </LButton>
    </template>

    <!-- Empty state -->
    <LEmptyState
      v-if="canvases.length === 0"
      title="No canvases yet"
      description="Create your first canvas to start exploring your data visually."
    >
      <template #icon>
        <Layout :size="48" class="text-text-subtle" />
      </template>
      <template #action>
        <LButton @click="createCanvas">
          <Plus :size="16" />
          Create Canvas
        </LButton>
      </template>
    </LEmptyState>

    <!-- Canvas grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <LCard
        v-for="canvas in canvases"
        :key="canvas.id"
        class="group hover:border-primary-500/50 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-200 cursor-pointer"
        @click="openCanvas(canvas.id)"
      >
        <div class="flex items-start justify-between mb-3">
          <!-- Editable title -->
          <div
            v-if="editingId === canvas.id"
            class="flex items-center gap-2 flex-1 mr-2"
            @click.stop
          >
            <LInput v-model="editingName" class="flex-1" autofocus @keydown="handleKeydown" />
            <button
              type="button"
              class="p-1.5 rounded text-text-muted hover:text-success hover:bg-success-muted transition-colors"
              @click="saveEditing"
            >
              <Check :size="16" />
            </button>
            <button
              type="button"
              class="p-1.5 rounded text-text-muted hover:text-error hover:bg-error-muted transition-colors"
              @click="cancelEditing"
            >
              <X :size="16" />
            </button>
          </div>
          <h3 v-else class="font-semibold text-text group-hover:text-primary-600 transition-colors">
            {{ canvas.name }}
          </h3>

          <!-- Actions -->
          <div v-if="editingId !== canvas.id" class="flex items-center gap-1">
            <button
              type="button"
              class="p-1.5 rounded text-text-muted hover:text-primary-600 hover:bg-primary-50 transition-colors opacity-0 group-hover:opacity-100"
              @click="startEditing(canvas.id, canvas.name, $event)"
            >
              <Pencil :size="16" />
            </button>
            <button
              type="button"
              class="p-1.5 rounded text-text-muted hover:text-error hover:bg-error-muted transition-colors opacity-0 group-hover:opacity-100"
              @click="deleteCanvas(canvas.id, $event)"
            >
              <Trash2 :size="16" />
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between text-xs text-text-subtle">
          <span>{{ canvas.nodes.length }} nodes</span>
          <span>Updated {{ formatDateShort(canvas.updatedAt) }}</span>
        </div>
      </LCard>
    </div>

    <!-- Delete confirmation modal -->
    <LModal v-model="showDeleteModal" title="Delete Canvas" size="sm">
      <p class="text-text">
        Are you sure you want to delete
        <strong>{{ canvasToDelete?.name }}</strong
        >?
      </p>
      <p class="text-sm text-text-muted mt-2">This action cannot be undone.</p>

      <template #footer>
        <LButton variant="secondary" @click="showDeleteModal = false">Cancel</LButton>
        <LButton variant="danger" @click="confirmDelete">Delete Canvas</LButton>
      </template>
    </LModal>
  </AppLayout>
</template>
