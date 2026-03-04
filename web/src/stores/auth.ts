import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api, setAuthToken, loadAuthToken, clearAllTokens } from '@/lib/api'
import { useConnectionStore } from '@/stores/connection'
import { useCommandStore } from '@/stores/command'
import i18n from '@/i18n'

const t = i18n.global.t

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(loadAuthToken())
  const authenticated = ref(!!token.value)
  const setupRequired = ref(false)
  const initialized = ref(false)
  const loading = ref(false)
  const error = ref('')
  const apiUnavailable = ref(false)

  async function checkAuthStatus() {
    try {
      const res = await api.authStatus()
      setupRequired.value = res.setup_required
      apiUnavailable.value = false
      // If we have a token, we're authenticated
      authenticated.value = !!token.value
    } catch {
      authenticated.value = false
      apiUnavailable.value = true
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
      initialized.value = true
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('auth.loginFailed')
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
      initialized.value = true
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('auth.registrationFailed')
      throw e
    } finally {
      loading.value = false
    }
  }

  async function logout() {
    try {
      await api.logout()
    } catch {
      // Best-effort server-side revocation
    }
    token.value = null
    clearAllTokens()
    authenticated.value = false
    initialized.value = false
    useConnectionStore().reset()
    useCommandStore().clearAll()
  }

  return {
    token,
    authenticated,
    setupRequired,
    initialized,
    loading,
    error,
    apiUnavailable,
    checkAuthStatus,
    login,
    register,
    logout,
  }
})
