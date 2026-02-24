import { setActivePinia } from 'pinia'
import { createTestingPinia } from '@pinia/testing'
import { useCommandStore } from '../command'
import { useConnectionStore } from '../connection'
import { api } from '@/lib/api'

vi.mock('@/lib/api', () => ({
  api: {
    command: vi.fn(),
    listServers: vi.fn(),
    addServer: vi.fn(),
    removeServer: vi.fn(),
  },
  setAuthToken: vi.fn(),
  setRefreshToken: vi.fn(),
  loadAuthToken: vi.fn().mockReturnValue(null),
  clearAllTokens: vi.fn(),
}))

vi.mock('@tanstack/vue-query', () => ({
  useQueryClient: () => ({ clear: vi.fn() }),
}))

beforeEach(() => {
  setActivePinia(createTestingPinia({ stubActions: false }))
  vi.clearAllMocks()
})

describe('useCommandStore', () => {
  function selectServer(id: number) {
    const conn = useConnectionStore()
    conn.servers = [{ id, homeserver: 'example.com', user_id: '@bot:example.com', connected: true }]
    conn.activeServerId = id
  }

  describe('execute', () => {
    it('calls api.command and records success in history', async () => {
      vi.mocked(api.command).mockResolvedValue({
        response: 'OK',
      })
      selectServer(1)
      const store = useCommandStore()
      const entry = await store.execute('admin users list')

      expect(api.command).toHaveBeenCalledWith(1, 'admin users list')
      expect(entry!.success).toBe(true)
      expect(entry!.response).toBe('OK')
      expect(store.history).toHaveLength(1)
    })

    it('records error in history on API failure', async () => {
      vi.mocked(api.command).mockRejectedValue(new Error('Server error'))
      selectServer(1)
      const store = useCommandStore()
      const entry = await store.execute('bad command')

      expect(entry!.success).toBe(false)
      expect(entry!.response).toBe('Server error')
    })

    it('throws when no server is selected', async () => {
      const store = useCommandStore()
      await expect(store.execute('anything')).rejects.toThrow('No server selected')
    })

    it('sets loading during execution', async () => {
      let resolve: (v: unknown) => void
      vi.mocked(api.command).mockReturnValue(new Promise((r) => { resolve = r }))
      selectServer(1)
      const store = useCommandStore()
      const p = store.execute('cmd')
      expect(store.loading).toBe(true)
      resolve!({ response: 'OK' })
      await p
      expect(store.loading).toBe(false)
    })
  })

  describe('history cap', () => {
    it('caps history at 500 entries', async () => {
      vi.mocked(api.command).mockResolvedValue({
        response: 'OK',
      })
      selectServer(1)
      const store = useCommandStore()

      for (let i = 0; i < 502; i++) {
        await store.execute(`cmd-${i}`)
      }
      expect(store.history.length).toBeLessThanOrEqual(500)
    })
  })

  describe('clear', () => {
    it('empties history', async () => {
      vi.mocked(api.command).mockResolvedValue({
        response: 'OK',
      })
      selectServer(1)
      const store = useCommandStore()
      await store.execute('cmd')
      expect(store.history).toHaveLength(1)
      store.clear()
      expect(store.history).toHaveLength(0)
    })
  })

  describe('togglePanel', () => {
    it('toggles panelOpen', () => {
      const store = useCommandStore()
      expect(store.panelOpen).toBe(false)
      store.togglePanel()
      expect(store.panelOpen).toBe(true)
      store.togglePanel()
      expect(store.panelOpen).toBe(false)
    })
  })
})
