import { api } from './client'
import type { User, UserRole } from '@/types'

export interface UpdateUserRoleRequest {
  role: UserRole
}

export const organizationsApi = {
  /**
   * List all users in the current organization
   */
  async listUsers(): Promise<User[]> {
    return api.get<User[]>('/organizations/users')
  },

  /**
   * Update a user's role within the organization (Admin only)
   */
  async updateUserRole(userId: string, role: UserRole): Promise<User> {
    return api.put<User>(`/organizations/users/${userId}/role`, { role })
  },

  /**
   * Remove a user from the organization (Admin only)
   */
  async removeUser(userId: string): Promise<void> {
    return api.delete(`/organizations/users/${userId}`)
  },
}
