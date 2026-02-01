import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLastDashboardId } from '@/utils/dashboardHistory'

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
      name: 'dashboard-entry',
      redirect: () => {
        const lastViewed = getLastDashboardId()
        if (lastViewed) {
          return { name: 'dashboard-editor', params: { id: lastViewed } }
        }
        return { name: 'dashboards' }
      },
      meta: { requiresAuth: true },
    },
    {
      path: '/dashboards',
      name: 'dashboards',
      component: () => import('@/views/DashboardsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/dashboards/new',
      name: 'dashboard-new',
      component: () => import('@/views/DashboardEditorView.vue'),
      meta: { requiresAuth: true, requiresEdit: true },
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
      meta: { requiresAuth: true, requiresAdmin: true },
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
      meta: { requiresAuth: true, requiresEdit: true },
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
      meta: { requiresAuth: true, requiresEdit: true },
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
      meta: { requiresAuth: true, requiresEdit: true },
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
    {
      path: '/style-guide',
      name: 'style-guide',
      component: () => import('@/views/StyleGuide.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/canvases',
      name: 'canvases',
      component: () => import('@/views/CanvasView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/canvases/new',
      name: 'canvas-new',
      component: () => import('@/views/CanvasEditorView.vue'),
      meta: { requiresAuth: true, requiresEdit: true },
    },
    {
      path: '/canvases/:id',
      name: 'canvas-editor',
      component: () => import('@/views/CanvasEditorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/access-denied',
      name: 'access-denied',
      component: () => import('@/views/AccessDenied.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      redirect: { name: 'dashboard-entry' },
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
    next({ name: 'dashboard-entry' })
  } else if (to.meta.requiresAdmin && authStore.user?.role !== 'admin') {
    // Redirect non-admins to access denied page
    console.warn('Access denied: Admin role required for', to.path)
    next({ name: 'access-denied', query: { required: 'admin' } })
  } else if (to.meta.requiresEdit && authStore.user?.role === 'viewer') {
    // Redirect viewers to access denied page for editor-only routes
    console.warn('Access denied: Editor or Admin role required for', to.path)
    next({ name: 'access-denied', query: { required: 'editor' } })
  } else {
    next()
  }
})

export default router
