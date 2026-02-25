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
  useQueryClient: () => ({ clear: vi.fn(), removeQueries: vi.fn() }),
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

    it('loading is per-server', async () => {
      let resolve: (v: unknown) => void
      vi.mocked(api.command).mockReturnValue(new Promise((r) => { resolve = r }))
      const conn = useConnectionStore()
      conn.servers = [
        { id: 1, homeserver: 'a.example.com', user_id: '@bot:a', connected: true },
        { id: 2, homeserver: 'b.example.com', user_id: '@bot:b', connected: true },
      ]
      conn.activeServerId = 1
      const store = useCommandStore()
      const p = store.execute('cmd')
      expect(store.loading).toBe(true)

      // Switch server — loading should not bleed
      conn.activeServerId = 2
      expect(store.loading).toBe(false)

      // Switch back — still loading
      conn.activeServerId = 1
      expect(store.loading).toBe(true)

      resolve!({ response: 'OK' })
      await p
      expect(store.loading).toBe(false)
    })
  })

  describe('per-server history', () => {
    it('keeps separate history per server', async () => {
      vi.mocked(api.command).mockResolvedValue({ response: 'OK' })
      const conn = useConnectionStore()
      conn.servers = [
        { id: 1, homeserver: 'a.example.com', user_id: '@bot:a', connected: true },
        { id: 2, homeserver: 'b.example.com', user_id: '@bot:b', connected: true },
      ]
      conn.activeServerId = 1
      const store = useCommandStore()

      await store.execute('cmd-a')
      expect(store.history).toHaveLength(1)

      conn.activeServerId = 2
      expect(store.history).toHaveLength(0)
      await store.execute('cmd-b')
      expect(store.history).toHaveLength(1)

      // Switch back — server 1 history is retained
      conn.activeServerId = 1
      expect(store.history).toHaveLength(1)
      expect(store.history[0]!.command).toBe('cmd-a')
    })
  })

  describe('history cap', () => {
    it('caps history at 500 entries', async () => {
      vi.mocked(api.command).mockResolvedValue({
        response: 'OK',
      })
      selectServer(1)
      const store = useCommandStore()

      const promises = []
      for (let i = 0; i < 501; i++) {
        promises.push(store.execute(`cmd-${i}`))
      }
      await Promise.all(promises)
      expect(store.history.length).toBeLessThanOrEqual(500)
    })
  })

  describe('clear', () => {
    it('empties history for the active server only', async () => {
      vi.mocked(api.command).mockResolvedValue({ response: 'OK' })
      const conn = useConnectionStore()
      conn.servers = [
        { id: 1, homeserver: 'a.example.com', user_id: '@bot:a', connected: true },
        { id: 2, homeserver: 'b.example.com', user_id: '@bot:b', connected: true },
      ]
      conn.activeServerId = 1
      const store = useCommandStore()
      await store.execute('cmd-a')

      conn.activeServerId = 2
      await store.execute('cmd-b')

      // Clear server 2
      store.clear()
      expect(store.history).toHaveLength(0)

      // Server 1 still has its history
      conn.activeServerId = 1
      expect(store.history).toHaveLength(1)
    })
  })

  describe('clearServer', () => {
    it('removes history for a specific server', async () => {
      vi.mocked(api.command).mockResolvedValue({ response: 'OK' })
      const conn = useConnectionStore()
      conn.servers = [
        { id: 1, homeserver: 'a.example.com', user_id: '@bot:a', connected: true },
        { id: 2, homeserver: 'b.example.com', user_id: '@bot:b', connected: true },
      ]
      conn.activeServerId = 1
      const store = useCommandStore()
      await store.execute('cmd-a')

      conn.activeServerId = 2
      await store.execute('cmd-b')

      // Clear server 1 by ID (not active)
      store.clearServer(1)
      expect(store.history).toHaveLength(1) // server 2 still has history

      conn.activeServerId = 1
      expect(store.history).toHaveLength(0) // server 1 was cleared
    })
  })

  describe('clearAll', () => {
    it('empties history for all servers', async () => {
      vi.mocked(api.command).mockResolvedValue({ response: 'OK' })
      const conn = useConnectionStore()
      conn.servers = [
        { id: 1, homeserver: 'a.example.com', user_id: '@bot:a', connected: true },
        { id: 2, homeserver: 'b.example.com', user_id: '@bot:b', connected: true },
      ]
      conn.activeServerId = 1
      const store = useCommandStore()
      await store.execute('cmd-a')

      conn.activeServerId = 2
      await store.execute('cmd-b')

      store.clearAll()
      expect(store.history).toHaveLength(0)
      conn.activeServerId = 1
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
