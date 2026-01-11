import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi, api } from '@/services/api'
import type { User, LoginRequest, RegisterRequest } from '@/types'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const token = ref<string | null>(null)
  const loading = ref(false)
  const initialized = ref(false)

  const isAuthenticated = computed(() => !!token.value && !!user.value)

  async function init() {
    const savedToken = localStorage.getItem('auth_token')
    if (savedToken) {
      token.value = savedToken
      api.setToken(savedToken)
      try {
        user.value = await authApi.me()
      } catch {
        // Token invalid, clear it
        logout()
      }
    }
    initialized.value = true
  }

  async function login(data: LoginRequest) {
    loading.value = true
    try {
      const response = await authApi.login(data)
      user.value = response.user
      token.value = response.token
      api.setToken(response.token)
      return response
    } finally {
      loading.value = false
    }
  }

  async function register(data: RegisterRequest) {
    loading.value = true
    try {
      const response = await authApi.register(data)
      user.value = response.user
      token.value = response.token
      api.setToken(response.token)
      return response
    } finally {
      loading.value = false
    }
  }

  function logout() {
    user.value = null
    token.value = null
    api.setToken(null)
  }

  return {
    user,
    token,
    loading,
    initialized,
    isAuthenticated,
    init,
    login,
    register,
    logout,
  }
})
