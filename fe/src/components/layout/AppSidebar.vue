<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import {
  LayoutDashboard,
  Database,
  FileCode,
  BarChart3,
  Calendar,
  Settings,
  Search,
  ChevronLeft,
  ChevronRight,
} from 'lucide-vue-next'

interface Props {
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  collapsed: false,
})

const emit = defineEmits<{
  'update:collapsed': [value: boolean]
}>()

const route = useRoute()

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboards' },
  { to: '/datasources', icon: Database, label: 'Datasources' },
  { to: '/queries', icon: FileCode, label: 'Queries' },
  { to: '/visualizations', icon: BarChart3, label: 'Visualizations' },
  { to: '/schedules', icon: Calendar, label: 'Schedules' },
]

const bottomNavItems = [{ to: '/settings', icon: Settings, label: 'Settings' }]

function isActive(to: string): boolean {
  if (to === '/') return route.path === '/' || route.path.startsWith('/dashboards')
  return route.path.startsWith(to)
}

function toggleCollapse() {
  emit('update:collapsed', !props.collapsed)
}
</script>

<template>
  <aside
    :class="[
      'fixed left-0 top-0 bottom-0 z-40 flex flex-col bg-surface-raised border-r border-border transition-all duration-300',
      collapsed ? 'w-16' : 'w-64',
    ]"
  >
    <!-- Logo -->
    <div class="h-16 flex items-center px-4 border-b border-border">
      <RouterLink to="/" class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-primary-600 flex items-center justify-center">
          <Search class="h-4 w-4 text-white" />
        </div>
        <span v-if="!collapsed" class="font-semibold text-lg text-text transition-opacity">
          Loupe
        </span>
      </RouterLink>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 py-4 overflow-y-auto scrollbar-thin">
      <ul class="space-y-1 px-2">
        <li v-for="item in navItems" :key="item.to">
          <RouterLink
            :to="item.to"
            :class="[
              'flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors',
              isActive(item.to)
                ? 'bg-primary-600 text-white'
                : 'text-text-muted hover:text-text hover:bg-surface-sunken',
            ]"
          >
            <component :is="item.icon" class="h-5 w-5 shrink-0" />
            <span v-if="!collapsed" class="text-sm font-medium">
              {{ item.label }}
            </span>
          </RouterLink>
        </li>
      </ul>
    </nav>

    <!-- Bottom navigation -->
    <div class="py-4 border-t border-border">
      <ul class="space-y-1 px-2">
        <li v-for="item in bottomNavItems" :key="item.to">
          <RouterLink
            :to="item.to"
            :class="[
              'flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors',
              isActive(item.to)
                ? 'bg-primary-600 text-white'
                : 'text-text-muted hover:text-text hover:bg-surface-sunken',
            ]"
          >
            <component :is="item.icon" class="h-5 w-5 shrink-0" />
            <span v-if="!collapsed" class="text-sm font-medium">
              {{ item.label }}
            </span>
          </RouterLink>
        </li>
      </ul>

      <!-- Collapse toggle -->
      <div class="px-2 mt-4">
        <button
          type="button"
          :class="[
            'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-text-muted hover:text-text hover:bg-surface-sunken transition-colors',
            collapsed && 'justify-center',
          ]"
          @click="toggleCollapse"
        >
          <component :is="collapsed ? ChevronRight : ChevronLeft" class="h-5 w-5 shrink-0" />
          <span v-if="!collapsed" class="text-sm font-medium">Collapse</span>
        </button>
      </div>
    </div>
  </aside>
</template>
