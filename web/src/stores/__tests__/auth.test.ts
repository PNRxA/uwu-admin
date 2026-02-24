import { setActivePinia } from 'pinia'
import { createTestingPinia } from '@pinia/testing'
import { useAuthStore } from '../auth'
import { api } from '@/lib/api'

vi.mock('@tanstack/vue-query', () => ({
  useQueryClient: () => ({ clear: vi.fn() }),
}))

vi.mock('@/lib/api', () => ({
  api: {
    authStatus: vi.fn(),
    login: vi.fn(),
    register: vi.fn(),
    logout: vi.fn(),
  },
  setAuthToken: vi.fn(),
  setRefreshToken: vi.fn(),
  loadAuthToken: vi.fn().mockReturnValue(null),
  clearAllTokens: vi.fn(),
}))

beforeEach(() => {
  setActivePinia(createTestingPinia({ stubActions: false }))
  vi.clearAllMocks()
})

describe('useAuthStore', () => {
  describe('checkAuthStatus', () => {
    it('sets setupRequired from API response', async () => {
      vi.mocked(api.authStatus).mockResolvedValue({ setup_required: true })
      const store = useAuthStore()
      await store.checkAuthStatus()
      expect(store.setupRequired).toBe(true)
      expect(store.initialized).toBe(true)
    })

    it('sets authenticated false on error', async () => {
      vi.mocked(api.authStatus).mockRejectedValue(new Error('network'))
      const store = useAuthStore()
      await store.checkAuthStatus()
      expect(store.authenticated).toBe(false)
      expect(store.initialized).toBe(true)
    })
  })

  describe('login', () => {
    it('sets token and authenticated on success', async () => {
      vi.mocked(api.login).mockResolvedValue({ token: 'tk', refresh_token: 'rt' })
      const store = useAuthStore()
      await store.login('user', 'pass')
      expect(store.token).toBe('tk')
      expect(store.authenticated).toBe(true)
      expect(store.loading).toBe(false)
    })

    it('sets error message on failure', async () => {
      vi.mocked(api.login).mockRejectedValue(new Error('Invalid credentials'))
      const store = useAuthStore()
      await expect(store.login('user', 'bad')).rejects.toThrow('Invalid credentials')
      expect(store.error).toBe('Invalid credentials')
      expect(store.authenticated).toBe(false)
      expect(store.loading).toBe(false)
    })

    it('sets loading flag during request', async () => {
      let resolve: (v: { token: string; refresh_token: string }) => void
      vi.mocked(api.login).mockReturnValue(new Promise((r) => { resolve = r }))
      const store = useAuthStore()
      const p = store.login('user', 'pass')
      expect(store.loading).toBe(true)
      resolve!({ token: 'tk', refresh_token: 'rt' })
      await p
      expect(store.loading).toBe(false)
    })
  })

  describe('register', () => {
    it('sets token, authenticated, and clears setupRequired', async () => {
      vi.mocked(api.register).mockResolvedValue({ token: 'tk', refresh_token: 'rt' })
      const store = useAuthStore()
      store.setupRequired = true
      await store.register('admin', 'pass')
      expect(store.token).toBe('tk')
      expect(store.authenticated).toBe(true)
      expect(store.setupRequired).toBe(false)
    })

    it('sets error on failure', async () => {
      vi.mocked(api.register).mockRejectedValue(new Error('Username taken'))
      const store = useAuthStore()
      await expect(store.register('admin', 'pass')).rejects.toThrow()
      expect(store.error).toBe('Username taken')
    })
  })

  describe('logout', () => {
    it('clears auth state', async () => {
      vi.mocked(api.logout).mockResolvedValue({ ok: true })
      const store = useAuthStore()
      store.token = 'tk'
      store.authenticated = true
      store.initialized = true
      await store.logout()
      expect(store.token).toBeNull()
      expect(store.authenticated).toBe(false)
      expect(store.initialized).toBe(false)
    })

    it('clears state even when API call fails', async () => {
      vi.mocked(api.logout).mockRejectedValue(new Error('network'))
      const store = useAuthStore()
      store.token = 'tk'
      store.authenticated = true
      store.initialized = true
      await store.logout()
      expect(store.token).toBeNull()
      expect(store.authenticated).toBe(false)
      expect(store.initialized).toBe(false)
    })
  })
})
