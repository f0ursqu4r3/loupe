import { ref, onMounted } from 'vue'

const isDark = ref(false)

function getSystemPreference(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function getSavedPreference(): boolean | null {
  const saved = localStorage.getItem('theme')
  if (saved === 'dark') return true
  if (saved === 'light') return false
  return null
}

function updateTheme(dark: boolean) {
  isDark.value = dark
  document.documentElement.classList.toggle('dark', dark)
  localStorage.setItem('theme', dark ? 'dark' : 'light')
}

function toggle() {
  updateTheme(!isDark.value)
}

function init() {
  const saved = getSavedPreference()
  const dark = saved !== null ? saved : getSystemPreference()
  updateTheme(dark)

  // Watch for system preference changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (getSavedPreference() === null) {
      updateTheme(e.matches)
    }
  })
}

export function useColorMode() {
  onMounted(() => {
    if (!document.documentElement.classList.contains('dark') && isDark.value) {
      document.documentElement.classList.add('dark')
    }
  })

  return {
    isDark,
    toggle,
    init,
  }
}

// Initialize on module load
if (typeof window !== 'undefined') {
  init()
}
