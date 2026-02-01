import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

export function useKeyboardShortcuts() {
  const router = useRouter()

  function isEditableElement(el: EventTarget | null): boolean {
    if (!el || !(el instanceof HTMLElement)) return false
    const tagName = el.tagName.toLowerCase()
    return (
      tagName === 'input' ||
      tagName === 'textarea' ||
      el.isContentEditable ||
      el.closest('.monaco-editor') !== null
    )
  }

  function handleKeyPress(event: KeyboardEvent) {
    // Don't trigger shortcuts when typing in inputs or text areas
    if (isEditableElement(event.target)) return

    // Check for modifier keys
    const isCtrl = event.ctrlKey || event.metaKey
    const isShift = event.shiftKey

    // Navigation shortcuts (G + letter)
    if (event.key === 'g' || event.key === 'G') {
      // Set a flag to capture the next key
      const nextKeyHandler = (e: KeyboardEvent) => {
        if (isEditableElement(e.target)) return

        e.preventDefault()
        switch (e.key.toLowerCase()) {
          case 'd':
            router.push({ name: 'dashboards' })
            break
          case 'q':
            router.push({ name: 'queries' })
            break
          case 'v':
            router.push({ name: 'visualizations' })
            break
          case 's':
            router.push({ name: 'settings' })
            break
          case 'c':
            router.push({ name: 'canvases' })
            break
        }
        window.removeEventListener('keydown', nextKeyHandler)
      }

      window.addEventListener('keydown', nextKeyHandler, { once: true })
      // Remove listener after 2 seconds if no second key is pressed
      setTimeout(() => {
        window.removeEventListener('keydown', nextKeyHandler)
      }, 2000)
    }

    // Help modal shortcut
    if (event.key === '?' && !isShift) {
      event.preventDefault()
      // Emit custom event that the app layout can listen to
      window.dispatchEvent(new CustomEvent('show-keyboard-shortcuts'))
    }

    // Toggle sidebar
    if (isCtrl && event.key === 'b') {
      event.preventDefault()
      window.dispatchEvent(new CustomEvent('toggle-sidebar'))
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyPress)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyPress)
  })

  return {
    // Expose any methods if needed
  }
}
