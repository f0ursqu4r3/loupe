<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { AppLayout } from '@/components/layout'
import { LCard, LInput, LButton, LCheckbox, LBadge, LSpinner, LModal, LSelect } from '@/components/ui'
import { ThemeToggle } from '@/components/layout'
import { useAuthStore } from '@/stores/auth'
import { organizationsApi } from '@/services/api'
import { usePermissions } from '@/composables/usePermissions'
import { useApiError } from '@/composables/useApiError'
import { useToast } from '@/composables/useToast'
import { User, Mail, Building2, Calendar, Shield, Users, Trash2, UserCog, LogOut } from 'lucide-vue-next'
import { formatDateLong } from '@/utils/dateTime'
import { useRouter } from 'vue-router'
import type { User as UserType, UserRole } from '@/types'

const router = useRouter()
const authStore = useAuthStore()
const { canAdmin, getRoleBadgeVariant, getRoleDisplayName } = usePermissions()
const { handleError } = useApiError()
const toast = useToast()

const displayName = ref(authStore.user?.name ?? '')

// Organization management
const users = ref<UserType[]>([])
const loadingUsers = ref(false)
const updating = ref<string | null>(null)

// Role change state
const showRoleModal = ref(false)
const userToUpdate = ref<UserType | null>(null)
const newRole = ref<UserRole>('viewer')

// Remove user state
const showRemoveModal = ref(false)
const userToRemove = ref<UserType | null>(null)
const removing = ref(false)

const currentUser = computed(() => authStore.user)

const roleOptions = [
  { value: 'admin', label: 'Admin - Full access to all features' },
  { value: 'editor', label: 'Editor - Can create and edit resources' },
  { value: 'viewer', label: 'Viewer - Read-only access' },
]

watch(
  () => authStore.user?.name,
  (name) => {
    displayName.value = name ?? ''
  },
)

function formatDate(dateStr: string | undefined) {
  if (!dateStr) return '—'
  return formatDateLong(dateStr)
}

async function loadUsers() {
  try {
    loadingUsers.value = true
    const response = await organizationsApi.listUsers()
    users.value = response.items
  } catch (e) {
    handleError(e, 'Failed to load organization users')
  } finally {
    loadingUsers.value = false
  }
}

function openRoleModal(user: UserType) {
  userToUpdate.value = user
  newRole.value = user.role
  showRoleModal.value = true
}

async function updateUserRole() {
  if (!userToUpdate.value) return

  try {
    updating.value = userToUpdate.value.id
    const updated = await organizationsApi.updateUserRole(userToUpdate.value.id, newRole.value)

    // Update the user in the list
    const index = users.value.findIndex((u) => u.id === updated.id)
    if (index !== -1) {
      users.value[index] = updated
    }

    toast.success(`Updated ${updated.name}'s role to ${getRoleDisplayName(updated.role)}`)
    showRoleModal.value = false
    userToUpdate.value = null
  } catch (e) {
    handleError(e, 'Failed to update user role')
  } finally {
    updating.value = null
  }
}

function openRemoveModal(user: UserType) {
  userToRemove.value = user
  showRemoveModal.value = true
}

async function removeUser() {
  if (!userToRemove.value) return

  try {
    removing.value = true
    await organizationsApi.removeUser(userToRemove.value.id)

    // Remove from list
    users.value = users.value.filter((u) => u.id !== userToRemove.value!.id)

    toast.success(`Removed ${userToRemove.value.name} from the organization`)
    showRemoveModal.value = false
    userToRemove.value = null
  } catch (e) {
    handleError(e, 'Failed to remove user')
  } finally {
    removing.value = false
  }
}

function isCurrentUser(user: UserType): boolean {
  return user.id === currentUser.value?.id
}

function logout() {
  authStore.logout()
  router.push({ name: 'auth' })
}

onMounted(loadUsers)
</script>

<template>
  <AppLayout title="Settings">
    <div class="max-w-2xl space-y-6">
      <!-- Profile Overview -->
      <LCard>
        <div class="flex items-start justify-between gap-4">
          <div class="flex items-start gap-4 flex-1 min-w-0">
            <div
              class="w-16 h-16 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center shrink-0"
            >
              <span class="text-2xl font-semibold text-primary-600 dark:text-primary-400">
                {{ authStore.user?.name.charAt(0).toUpperCase() || 'U' }}
              </span>
            </div>
            <div class="flex-1 min-w-0">
              <h2 class="text-xl font-semibold text-text truncate">
                {{ authStore.user?.name || 'Anonymous' }}
              </h2>
              <p class="text-sm text-text-muted truncate">{{ authStore.user?.email }}</p>
              <div class="flex items-center gap-2 mt-2">
                <LBadge :variant="getRoleBadgeVariant()">
                  <Shield :size="12" class="mr-1" />
                  {{ getRoleDisplayName() }}
                </LBadge>
              </div>
            </div>
          </div>
          <LButton variant="secondary" @click="logout">
            <LogOut :size="16" />
            Log out
          </LButton>
        </div>
      </LCard>

      <!-- Account Details -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Account Details</h2>

        <div class="space-y-4">
          <div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
            <Mail :size="20" class="text-text-muted" />
            <div class="flex-1 min-w-0">
              <p class="text-xs text-text-muted">Email</p>
              <p class="text-sm text-text truncate">{{ authStore.user?.email }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
            <Building2 :size="20" class="text-text-muted" />
            <div class="flex-1 min-w-0">
              <p class="text-xs text-text-muted">Organization ID</p>
              <p class="text-sm text-text font-mono truncate">{{ authStore.user?.org_id }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
            <Calendar :size="20" class="text-text-muted" />
            <div class="flex-1 min-w-0">
              <p class="text-xs text-text-muted">Member Since</p>
              <p class="text-sm text-text">{{ formatDate(authStore.user?.created_at) }}</p>
            </div>
          </div>
        </div>
      </LCard>

      <!-- Edit Profile -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Edit Profile</h2>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text mb-1.5">Display Name</label>
            <LInput v-model="displayName" placeholder="Your name" />
          </div>

          <div class="pt-2">
            <LButton>Save Changes</LButton>
          </div>
        </div>
      </LCard>

      <!-- Appearance -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Appearance</h2>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text mb-2">Theme</label>
            <p class="text-sm text-text-muted mb-3">
              Choose how Loupe looks to you. Select a single theme, or sync with your system.
            </p>
            <ThemeToggle />
          </div>
        </div>
      </LCard>

      <!-- Notifications -->
      <LCard>
        <h2 class="text-lg font-semibold text-text mb-4">Notifications</h2>

        <div class="space-y-4">
          <label class="flex items-center gap-3 cursor-pointer">
            <LCheckbox :model-value="true" />
            <div>
              <p class="text-sm font-medium text-text">Query failures</p>
              <p class="text-xs text-text-muted">Get notified when a scheduled query fails</p>
            </div>
          </label>

          <label class="flex items-center gap-3 cursor-pointer">
            <LCheckbox :model-value="false" />
            <div>
              <p class="text-sm font-medium text-text">Schedule reminders</p>
              <p class="text-xs text-text-muted">Remind me before scheduled queries run</p>
            </div>
          </label>
        </div>
      </LCard>

      <!-- Organization Members -->
      <LCard>
        <div class="flex items-center justify-between mb-4">
          <div>
            <h2 class="text-lg font-semibold text-text flex items-center gap-2">
              <Users :size="20" />
              Organization Members
            </h2>
            <p class="text-sm text-text-muted mt-1">
              {{
                canAdmin
                  ? 'Manage user roles and permissions for your organization'
                  : 'View members of your organization'
              }}
            </p>
          </div>
          <LBadge variant="secondary">{{ users.length }} {{ users.length === 1 ? 'member' : 'members' }}</LBadge>
        </div>

        <!-- Loading state -->
        <div v-if="loadingUsers" class="flex items-center justify-center py-8">
          <LSpinner />
        </div>

        <!-- Users List -->
        <div v-else class="space-y-3">
          <div
            v-for="user in users"
            :key="user.id"
            class="flex items-center justify-between p-3 bg-surface-alt rounded-lg"
          >
            <div class="flex items-center gap-3 flex-1 min-w-0">
              <!-- Avatar -->
              <div class="w-10 h-10 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center shrink-0">
                <span class="text-primary-600 dark:text-primary-400 font-semibold text-sm">
                  {{ user.name.charAt(0).toUpperCase() }}
                </span>
              </div>

              <!-- User Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <h3 class="font-medium text-text text-sm truncate">{{ user.name }}</h3>
                  <span v-if="isCurrentUser(user)" class="text-xs text-text-muted">(You)</span>
                </div>
                <p class="text-xs text-text-muted truncate">{{ user.email }}</p>
              </div>

              <!-- Role Badge -->
              <LBadge :variant="getRoleBadgeVariant(user.role)" size="sm">
                {{ getRoleDisplayName(user.role) }}
              </LBadge>
            </div>

            <!-- Actions (Admin only, not for self) -->
            <div v-if="canAdmin && !isCurrentUser(user)" class="flex items-center gap-2 ml-3">
              <LButton
                variant="secondary"
                size="sm"
                @click="openRoleModal(user)"
                :loading="updating === user.id"
              >
                <UserCog :size="14" />
              </LButton>

              <LButton
                variant="secondary"
                size="sm"
                @click="openRemoveModal(user)"
              >
                <Trash2 :size="14" />
              </LButton>
            </div>
          </div>
        </div>

        <!-- Info message for non-admins -->
        <div v-if="!canAdmin" class="mt-4 p-3 bg-surface-sunken rounded-lg border border-border">
          <p class="text-xs text-text-muted">
            <Shield :size="14" class="inline mr-1" />
            Only administrators can manage user roles and remove users.
          </p>
        </div>
      </LCard>

      <!-- Danger Zone -->
      <LCard class="border-error/50">
        <h2 class="text-lg font-semibold text-error mb-4">Danger Zone</h2>

        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text">Delete Account</p>
            <p class="text-xs text-text-muted">
              Permanently delete your account and all associated data
            </p>
          </div>
          <LButton variant="danger">Delete Account</LButton>
        </div>
      </LCard>
    </div>

    <!-- Change Role Modal -->
    <LModal v-model="showRoleModal" title="Change User Role">
      <div v-if="userToUpdate" class="space-y-4">
        <div>
          <p class="text-sm text-text-muted mb-4">
            Change role for <strong class="text-text">{{ userToUpdate.name }}</strong>
          </p>

          <label class="block text-sm font-medium text-text mb-2">New Role</label>
          <LSelect v-model="newRole" :options="roleOptions" />
        </div>

        <div class="p-3 bg-surface-sunken rounded-lg border border-border">
          <p class="text-xs text-text-muted">
            <strong>Admin:</strong> Full access including user management and datasources<br />
            <strong>Editor:</strong> Can create and edit dashboards, queries, and visualizations<br />
            <strong>Viewer:</strong> Read-only access to all resources
          </p>
        </div>
      </div>

      <template #footer>
        <LButton variant="secondary" @click="showRoleModal = false">Cancel</LButton>
        <LButton
          variant="primary"
          :loading="updating !== null"
          @click="updateUserRole"
          :disabled="!userToUpdate || newRole === userToUpdate.role"
        >
          Update Role
        </LButton>
      </template>
    </LModal>

    <!-- Remove User Modal -->
    <LModal v-model="showRemoveModal" title="Remove User">
      <div v-if="userToRemove" class="space-y-4">
        <p class="text-text">
          Are you sure you want to remove <strong>{{ userToRemove.name }}</strong> from the organization?
        </p>

        <div class="p-3 bg-error-muted rounded-lg border border-error">
          <p class="text-sm text-error">
            <strong>Warning:</strong> This user will immediately lose access to all resources and data in this organization. This action cannot be undone.
          </p>
        </div>
      </div>

      <template #footer>
        <LButton variant="secondary" @click="showRemoveModal = false">Cancel</LButton>
        <LButton variant="danger" :loading="removing" @click="removeUser">
          Remove User
        </LButton>
      </template>
    </LModal>
  </AppLayout>
</template>
