import { ref, onMounted, onUnmounted } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'loupe-theme'
const theme = ref<Theme>('system')

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(value: Theme) {
  if (typeof window === 'undefined') return

  const effectiveTheme = value === 'system' ? getSystemTheme() : value
  document.documentElement.classList.toggle('dark', effectiveTheme === 'dark')
}

function loadTheme() {
  if (typeof window === 'undefined') return

  const saved = localStorage.getItem(STORAGE_KEY) as Theme | null
  if (saved && ['light', 'dark', 'system'].includes(saved)) {
    theme.value = saved
  }
  applyTheme(theme.value)
}

export function useTheme() {
  let mediaQuery: MediaQueryList | null = null

  function setTheme(value: Theme) {
    theme.value = value
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, value)
    }
    applyTheme(value)
  }

  function handleSystemThemeChange() {
    if (theme.value === 'system') {
      applyTheme('system')
    }
  }

  onMounted(() => {
    // Load saved theme
    loadTheme()

    // Listen for system theme changes
    if (typeof window !== 'undefined') {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', handleSystemThemeChange)
    }
  })

  onUnmounted(() => {
    // Clean up listener
    if (mediaQuery) {
      mediaQuery.removeEventListener('change', handleSystemThemeChange)
    }
  })

  return {
    theme,
    setTheme,
  }
}

// Initialize theme immediately (before any components mount)
export function initTheme() {
  loadTheme()
}
