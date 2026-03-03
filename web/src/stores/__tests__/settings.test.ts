import { setActivePinia } from 'pinia'
import { createTestingPinia } from '@pinia/testing'
import { useSettingsStore } from '../settings'
import { api } from '@/lib/api'

vi.mock('@/lib/api', () => ({
  api: {
    getSettings: vi.fn(),
    updateSettings: vi.fn(),
  },
  setAuthToken: vi.fn(),
  loadAuthToken: vi.fn().mockReturnValue(null),
  clearAllTokens: vi.fn(),
}))

beforeEach(() => {
  setActivePinia(createTestingPinia({ stubActions: false }))
  vi.clearAllMocks()
})

describe('useSettingsStore', () => {
  describe('fetchSettings', () => {
    it('populates settings from API', async () => {
      vi.mocked(api.getSettings).mockResolvedValue({ redact_messages: 'true' })
      const store = useSettingsStore()
      await store.fetchSettings()
      expect(store.settings).toEqual({ redact_messages: 'true' })
      expect(store.loading).toBe(false)
    })

    it('sets error on failure', async () => {
      vi.mocked(api.getSettings).mockRejectedValue(new Error('Network error'))
      const store = useSettingsStore()
      await store.fetchSettings()
      expect(store.error).toBe('Network error')
      expect(store.loading).toBe(false)
    })

    it('sets loading flag during request', async () => {
      let resolve: (v: Record<string, string>) => void
      vi.mocked(api.getSettings).mockReturnValue(new Promise((r) => { resolve = r }))
      const store = useSettingsStore()
      const p = store.fetchSettings()
      expect(store.loading).toBe(true)
      resolve!({ redact_messages: 'true' })
      await p
      expect(store.loading).toBe(false)
    })
  })

  describe('updateSetting', () => {
    it('updates settings from API response', async () => {
      vi.mocked(api.updateSettings).mockResolvedValue({ redact_messages: 'false' })
      const store = useSettingsStore()
      await store.updateSetting('redact_messages', 'false')
      expect(store.settings).toEqual({ redact_messages: 'false' })
      expect(api.updateSettings).toHaveBeenCalledWith({ redact_messages: 'false' })
    })

    it('sets error and rethrows on failure', async () => {
      vi.mocked(api.updateSettings).mockRejectedValue(new Error('Server error'))
      const store = useSettingsStore()
      await expect(store.updateSetting('redact_messages', 'false')).rejects.toThrow('Server error')
      expect(store.error).toBe('Server error')
    })
  })
})
