import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import type { UserRole } from '@/types'

export enum Permission {
  Viewer = 'viewer',
  Editor = 'editor',
  Admin = 'admin',
}

/**
 * Composable for role-based permission checking
 *
 * Permission hierarchy:
 * - Viewer: Can only view resources
 * - Editor: Can view and edit resources (includes Viewer permissions)
 * - Admin: Full access (includes Editor and Viewer permissions)
 *
 * @example
 * ```vue
 * <script setup>
 * import { usePermissions } from '@/composables/usePermissions'
 *
 * const { canEdit, canAdmin, hasPermission } = usePermissions()
 * </script>
 *
 * <template>
 *   <LButton v-if="canEdit" @click="createDashboard">
 *     New Dashboard
 *   </LButton>
 * </template>
 * ```
 */
export function usePermissions() {
  const authStore = useAuthStore()

  const role = computed<UserRole | null>(() => authStore.user?.role ?? null)

  /**
   * All authenticated users can view (Viewer, Editor, Admin)
   */
  const canView = computed(() => !!role.value)

  /**
   * Editors and Admins can edit (create, update, delete)
   */
  const canEdit = computed(() => role.value === 'editor' || role.value === 'admin')

  /**
   * Only Admins have admin privileges (datasource management, user management)
   */
  const canAdmin = computed(() => role.value === 'admin')

  /**
   * Check if the user has a specific permission level
   *
   * Uses permission hierarchy:
   * - admin can do viewer, editor, and admin actions
   * - editor can do viewer and editor actions
   * - viewer can only do viewer actions
   *
   * @param required - The minimum required permission level
   * @returns true if the user has the required permission or higher
   */
  const hasPermission = (required: Permission): boolean => {
    if (!role.value) return false

    const hierarchy: Record<UserRole, Permission[]> = {
      viewer: [Permission.Viewer],
      editor: [Permission.Viewer, Permission.Editor],
      admin: [Permission.Viewer, Permission.Editor, Permission.Admin],
    }

    return hierarchy[role.value]?.includes(required) ?? false
  }

  /**
   * Get a user-friendly message explaining why an action is forbidden
   *
   * @param requiredPermission - The permission level required
   * @returns A message explaining what role is needed
   */
  const getPermissionDeniedMessage = (requiredPermission: Permission): string => {
    const messages: Record<Permission, string> = {
      [Permission.Viewer]: 'You must be logged in to perform this action',
      [Permission.Editor]:
        'This action requires Editor or Admin role. Contact your administrator to upgrade your permissions.',
      [Permission.Admin]:
        'This action requires Admin role. Only administrators can perform this action.',
    }

    return messages[requiredPermission]
  }

  /**
   * Get the variant for role badge display
   *
   * @param userRole - The role to get the variant for (defaults to current user's role)
   * @returns Badge variant for the role
   */
  const getRoleBadgeVariant = (userRole?: UserRole): 'danger' | 'primary' | 'secondary' => {
    const roleToCheck = userRole ?? role.value

    switch (roleToCheck) {
      case 'admin':
        return 'danger' // Red for admin
      case 'editor':
        return 'primary' // Blue for editor
      case 'viewer':
        return 'secondary' // Gray for viewer
      default:
        return 'secondary'
    }
  }

  /**
   * Get a display-friendly role name
   *
   * @param userRole - The role to format (defaults to current user's role)
   * @returns Capitalized role name
   */
  const getRoleDisplayName = (userRole?: UserRole): string => {
    const roleToCheck = userRole ?? role.value
    if (!roleToCheck) return 'Unknown'

    return roleToCheck.charAt(0).toUpperCase() + roleToCheck.slice(1)
  }

  return {
    role,
    canView,
    canEdit,
    canAdmin,
    hasPermission,
    getPermissionDeniedMessage,
    getRoleBadgeVariant,
    getRoleDisplayName,
  }
}
