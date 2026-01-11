<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Sun, Moon, Monitor } from 'lucide-vue-next'

type Theme = 'light' | 'dark' | 'system'

const theme = ref<Theme>('system')

const themes: { value: Theme; icon: typeof Sun; label: string }[] = [
  { value: 'light', icon: Sun, label: 'Light' },
  { value: 'dark', icon: Moon, label: 'Dark' },
  { value: 'system', icon: Monitor, label: 'System' },
]

function getSystemTheme(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(value: Theme) {
  const effectiveTheme = value === 'system' ? getSystemTheme() : value
  document.documentElement.classList.toggle('dark', effectiveTheme === 'dark')
}

function setTheme(value: Theme) {
  theme.value = value
  localStorage.setItem('loupe-theme', value)
  applyTheme(value)
}

onMounted(() => {
  const saved = localStorage.getItem('loupe-theme') as Theme | null
  if (saved && ['light', 'dark', 'system'].includes(saved)) {
    theme.value = saved
  }
  applyTheme(theme.value)

  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (theme.value === 'system') {
      applyTheme('system')
    }
  })
})
</script>

<template>
  <div class="flex items-center gap-1 p-1 rounded-lg bg-surface-sunken">
    <button
      v-for="t in themes"
      :key="t.value"
      type="button"
      :title="t.label"
      :class="[
        'p-2 rounded-md transition-colors focus-ring',
        theme === t.value ? 'bg-surface text-text shadow-sm' : 'text-text-muted hover:text-text',
      ]"
      @click="setTheme(t.value)"
    >
      <component :is="t.icon" class="h-4 w-4" />
    </button>
  </div>
</template>
