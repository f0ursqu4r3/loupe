import { inject, type InjectionKey, type Ref } from 'vue'
import type { Toast } from '@/components/ui/LToastContainer.vue'

export interface ToastAPI {
  show: (toast: Omit<Toast, 'id'>) => string
  success: (message: string, title?: string) => string
  error: (message: string, title?: string) => string
  warning: (message: string, title?: string) => string
  info: (message: string, title?: string) => string
  dismiss: (id: string) => void
}

export const ToastSymbol: InjectionKey<Ref<ToastAPI | null>> = Symbol('toast')

export function useToast(): ToastAPI {
  const toastAPI = inject(ToastSymbol, null)

  if (!toastAPI?.value) {
    // Fallback to console if toast system not initialized
    return {
      show: (toast) => {
        console.log('[Toast]', toast)
        return ''
      },
      success: (message, title) => {
        console.log('[Toast Success]', title || 'Success', message)
        return ''
      },
      error: (message, title) => {
        console.error('[Toast Error]', title || 'Error', message)
        return ''
      },
      warning: (message, title) => {
        console.warn('[Toast Warning]', title || 'Warning', message)
        return ''
      },
      info: (message, title) => {
        console.info('[Toast Info]', title || 'Info', message)
        return ''
      },
      dismiss: () => {
        // No-op
      },
    }
  }

  return toastAPI.value
}
