import { api } from './client'
import type {
  User,
  LoginRequest,
  RegisterRequest,
  AuthResponse,
} from '@/types'

export const authApi = {
  login(data: LoginRequest): Promise<AuthResponse> {
    return api.post<AuthResponse>('/auth/login', data)
  },

  async register(data: RegisterRequest): Promise<AuthResponse> {
    // Backend returns user directly on register, then we auto-login
    const user = await api.post<User>('/auth/register', data)
    // Auto-login after registration
    return api.post<AuthResponse>('/auth/login', {
      email: data.email,
      password: data.password,
    })
  },

  logout(): Promise<void> {
    return api.post('/auth/logout')
  },

  me(): Promise<User> {
    return api.get<User>('/auth/me')
  },
}
