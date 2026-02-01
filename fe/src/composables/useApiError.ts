import { useToast } from './useToast'
import { ApiRequestError } from '@/services/api/client'

/**
 * Composable for centralized API error handling
 *
 * Provides user-friendly error messages for common API errors,
 * with special handling for permission-related errors (401, 403)
 *
 * @example
 * ```vue
 * <script setup>
 * import { useApiError } from '@/composables/useApiError'
 *
 * const { handleError } = useApiError()
 *
 * async function createDashboard() {
 *   try {
 *     await dashboardsApi.create(data)
 *   } catch (error) {
 *     handleError(error, 'Failed to create dashboard')
 *   }
 * }
 * </script>
 * ```
 */
export function useApiError() {
  const toast = useToast()

  /**
   * Handle API errors with user-friendly toast notifications
   *
   * Special handling for:
   * - 401 Unauthorized: Suggests re-login
   * - 403 Forbidden: Shows permission-related message
   * - 404 Not Found: Resource not found
   * - 422 Validation Error: Shows validation details
   * - 5xx: Generic server error message
   *
   * @param error - The error object (typically ApiRequestError)
   * @param fallbackMessage - Optional fallback message if error details aren't available
   * @returns void
   */
  function handleError(error: unknown, fallbackMessage?: string): void {
    console.error('API Error:', error)

    if (error instanceof ApiRequestError) {
      // 401 Unauthorized - user needs to log in
      if (error.isUnauthorized) {
        toast.error(
          error.error.message || 'Your session has expired. Please log in again.',
          'Authentication Required',
        )
        return
      }

      // 403 Forbidden - user doesn't have permission
      if (error.isForbidden) {
        const message =
          error.error.message ||
          'You do not have permission to perform this action. Contact your administrator if you need access.'

        toast.error(message, 'Permission Denied')

        // Log error ID if available for debugging
        if (error.error.details && typeof error.error.details === 'object') {
          const details = error.error.details as Record<string, unknown>
          if (details.error_id) {
            console.error('Error ID:', details.error_id)
          }
        }

        return
      }

      // 404 Not Found
      if (error.isNotFound) {
        toast.error(
          error.error.message || 'The requested resource was not found.',
          'Not Found',
        )
        return
      }

      // 422 Validation Error
      if (error.isValidationError) {
        const message = error.error.message || 'Please check your input and try again.'
        toast.error(message, 'Validation Error')
        return
      }

      // 5xx Server Errors
      if (error.status >= 500) {
        toast.error(
          'An unexpected server error occurred. Please try again later.',
          'Server Error',
        )
        return
      }

      // Other API errors
      const message = error.error.message || fallbackMessage || 'An error occurred'
      toast.error(message)
      return
    }

    // Non-API errors (network errors, etc.)
    const message = error instanceof Error ? error.message : String(error)
    toast.error(
      fallbackMessage || message || 'An unexpected error occurred',
      'Error',
    )
  }

  /**
   * Check if an error is a permission error (403 Forbidden)
   *
   * @param error - The error to check
   * @returns true if the error is a 403 Forbidden error
   */
  function isPermissionError(error: unknown): boolean {
    return error instanceof ApiRequestError && error.isForbidden
  }

  /**
   * Check if an error is an authentication error (401 Unauthorized)
   *
   * @param error - The error to check
   * @returns true if the error is a 401 Unauthorized error
   */
  function isAuthError(error: unknown): boolean {
    return error instanceof ApiRequestError && error.isUnauthorized
  }

  /**
   * Get a user-friendly error message from an error object
   *
   * @param error - The error object
   * @param fallback - Fallback message if none can be extracted
   * @returns User-friendly error message
   */
  function getErrorMessage(error: unknown, fallback = 'An error occurred'): string {
    if (error instanceof ApiRequestError) {
      return error.error.message || fallback
    }

    if (error instanceof Error) {
      return error.message || fallback
    }

    return fallback
  }

  return {
    handleError,
    isPermissionError,
    isAuthError,
    getErrorMessage,
  }
}
