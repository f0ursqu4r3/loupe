<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Eye, EyeOff } from 'lucide-vue-next'
import { LButton, LInput, LAlert } from '@/components/ui'
import { ThemeToggle } from '@/components/layout'
import { useAuthStore } from '@/stores/auth'
import { ApiRequestError } from '@/services/api'

const router = useRouter()
const authStore = useAuthStore()

const mode = ref<'login' | 'register'>('login')
const showPassword = ref(false)
const error = ref<string | null>(null)

const form = ref({
  email: '',
  password: '',
  name: '',
})

const isValid = computed(() => {
  if (mode.value === 'login') {
    return form.value.email && form.value.password
  }
  return form.value.email && form.value.password && form.value.name
})

async function submit() {
  error.value = null
  try {
    if (mode.value === 'login') {
      await authStore.login({
        email: form.value.email,
        password: form.value.password,
      })
    } else {
      await authStore.register({
        email: form.value.email,
        password: form.value.password,
        name: form.value.name,
      })
    }
    router.push('/')
  } catch (e) {
    if (e instanceof ApiRequestError) {
      error.value = e.error.message
    } else {
      error.value = 'An unexpected error occurred'
    }
  }
}

function toggleMode() {
  mode.value = mode.value === 'login' ? 'register' : 'login'
  error.value = null
}
</script>

<template>
  <div class="min-h-screen bg-surface flex">
    <!-- Left side - branding -->
    <div class="hidden lg:flex lg:w-1/2 bg-primary-600 p-12 flex-col justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-white/20 flex items-center justify-center">
          <Search :size="20" class="text-white" />
        </div>
        <span class="font-bold text-2xl text-white">Loupe</span>
      </div>

      <div>
        <h1 class="text-4xl font-bold text-white mb-4">Business Intelligence<br />Made Simple</h1>
        <p class="text-primary-100 text-lg">
          Connect your databases, write SQL queries, and create beautiful visualizations and
          dashboards.
        </p>
      </div>

      <p class="text-primary-200 text-sm">© 2026 Loupe. Open source BI platform.</p>
    </div>

    <!-- Right side - auth form -->
    <div class="flex-1 flex flex-col">
      <div class="flex justify-end p-4">
        <ThemeToggle />
      </div>

      <div class="flex-1 flex items-center justify-center p-8">
        <div class="w-full max-w-md">
          <!-- Mobile logo -->
          <div class="lg:hidden flex items-center gap-3 mb-8">
            <div class="w-10 h-10 rounded-xl bg-primary-600 flex items-center justify-center">
              <Search :size="20" class="text-white" />
            </div>
            <span class="font-bold text-2xl text-text">Loupe</span>
          </div>

          <h2 class="text-2xl font-bold text-text mb-2">
            {{ mode === 'login' ? 'Welcome back' : 'Create your account' }}
          </h2>
          <p class="text-text-muted mb-8">
            {{
              mode === 'login'
                ? 'Sign in to your account to continue'
                : 'Get started with Loupe for free'
            }}
          </p>

          <LAlert v-if="error" variant="error" class="mb-6" dismissible @dismiss="error = null">
            {{ error }}
          </LAlert>

          <form class="space-y-4" @submit.prevent="submit">
            <div v-if="mode === 'register'">
              <label class="block text-sm font-medium text-text mb-1.5">Name</label>
              <LInput v-model="form.name" placeholder="Your name" autocomplete="name" />
            </div>

            <div>
              <label class="block text-sm font-medium text-text mb-1.5">Email</label>
              <LInput
                v-model="form.email"
                type="email"
                placeholder="you@example.com"
                autocomplete="email"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-text mb-1.5">Password</label>
              <div class="relative">
                <LInput
                  v-model="form.password"
                  :type="showPassword ? 'text' : 'password'"
                  placeholder="••••••••"
                  :autocomplete="mode === 'login' ? 'current-password' : 'new-password'"
                  class="pr-10"
                />
                <button
                  type="button"
                  class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text"
                  @click="showPassword = !showPassword"
                >
                  <component :is="showPassword ? EyeOff : Eye" :size="16" />
                </button>
              </div>
            </div>

            <LButton type="submit" class="w-full" :loading="authStore.loading" :disabled="!isValid">
              {{ mode === 'login' ? 'Sign in' : 'Create account' }}
            </LButton>
          </form>

          <p class="mt-6 text-center text-sm text-text-muted">
            {{ mode === 'login' ? "Don't have an account?" : 'Already have an account?' }}
            <button
              type="button"
              class="text-primary-600 hover:text-primary-700 font-medium ml-1"
              @click="toggleMode"
            >
              {{ mode === 'login' ? 'Sign up' : 'Sign in' }}
            </button>
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
