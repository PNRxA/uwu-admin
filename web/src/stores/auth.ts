import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api, setAuthToken, loadAuthToken } from '@/lib/api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(loadAuthToken())
  const authenticated = ref(!!token.value)
  const setupRequired = ref(false)
  const initialized = ref(false)
  const loading = ref(false)
  const error = ref('')

  async function checkAuthStatus() {
    try {
      const res = await api.authStatus()
      setupRequired.value = res.setup_required
      // If we have a token, we're authenticated
      authenticated.value = !!token.value
    } catch {
      authenticated.value = false
    } finally {
      initialized.value = true
    }
  }

  async function login(username: string, password: string) {
    loading.value = true
    error.value = ''
    try {
      const res = await api.login(username, password)
      token.value = res.token
      setAuthToken(res.token)
      authenticated.value = true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Login failed'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function register(username: string, password: string) {
    loading.value = true
    error.value = ''
    try {
      const res = await api.register(username, password)
      token.value = res.token
      setAuthToken(res.token)
      authenticated.value = true
      setupRequired.value = false
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Registration failed'
      throw e
    } finally {
      loading.value = false
    }
  }

  function logout() {
    token.value = null
    setAuthToken(null)
    authenticated.value = false
  }

  return {
    token,
    authenticated,
    setupRequired,
    initialized,
    loading,
    error,
    checkAuthStatus,
    login,
    register,
    logout,
  }
})
