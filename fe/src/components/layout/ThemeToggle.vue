<script setup lang="ts">
import { Sun, Moon, Monitor } from 'lucide-vue-next'
import { LButton } from '@/components/ui'
import { useTheme, type Theme } from '@/composables/useTheme'

const { theme, setTheme } = useTheme()

const themes: { value: Theme; icon: typeof Sun; label: string }[] = [
  { value: 'light', icon: Sun, label: 'Light' },
  { value: 'dark', icon: Moon, label: 'Dark' },
  { value: 'system', icon: Monitor, label: 'System' },
]
</script>

<template>
  <div class="flex items-center gap-1 p-1 rounded-lg bg-surface-sunken">
    <LButton
      v-for="t in themes"
      :key="t.value"
      :title="t.label"
      :aria-label="`Switch to ${t.label.toLowerCase()} theme`"
      :variant="theme === t.value ? 'secondary' : 'ghost'"
      size="xs"
      square
      @click="setTheme(t.value)"
    >
      <component :is="t.icon" :size="14" />
    </LButton>
  </div>
</template>
