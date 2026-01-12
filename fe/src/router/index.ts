import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/auth',
      name: 'auth',
      component: () => import('@/views/AuthView.vue'),
      meta: { guest: true },
    },
    {
      path: '/',
      name: 'dashboards',
      component: () => import('@/views/DashboardsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/dashboards/new',
      name: 'dashboard-new',
      component: () => import('@/views/DashboardEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/dashboards/:id',
      name: 'dashboard-editor',
      component: () => import('@/views/DashboardEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/datasources',
      name: 'datasources',
      component: () => import('@/views/DatasourcesView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/queries',
      name: 'queries',
      component: () => import('@/views/QueriesView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/queries/new',
      name: 'query-new',
      component: () => import('@/views/QueryEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/queries/:id',
      name: 'query-editor',
      component: () => import('@/views/QueryEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/visualizations',
      name: 'visualizations',
      component: () => import('@/views/VisualizationsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/visualizations/new',
      name: 'visualization-new',
      component: () => import('@/views/VisualizationEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/visualizations/:id',
      name: 'visualization-editor',
      component: () => import('@/views/VisualizationEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/schedules',
      name: 'schedules',
      component: () => import('@/views/SchedulesView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/schedules/new',
      name: 'schedule-new',
      component: () => import('@/views/ScheduleEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/schedules/:id',
      name: 'schedule-editor',
      component: () => import('@/views/ScheduleEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

// Navigation guards
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()

  // Wait for auth to initialize on first load
  if (!authStore.initialized) {
    await authStore.init()
  }

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    // Redirect to login if not authenticated
    next({ name: 'auth', query: { redirect: to.fullPath } })
  } else if (to.meta.guest && authStore.isAuthenticated) {
    // Redirect to home if already authenticated
    next({ name: 'dashboards' })
  } else {
    next()
  }
})

export default router
