import { setActivePinia } from 'pinia'
import { createTestingPinia } from '@pinia/testing'
import { useConnectionStore } from '../connection'
import { useCommandStore } from '../command'
import { api } from '@/lib/api'

vi.mock('@/lib/api', () => ({
  api: {
    listServers: vi.fn(),
    addServer: vi.fn(),
    removeServer: vi.fn(),
    command: vi.fn(),
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

const SERVER_A = { id: 1, homeserver: 'a.example.com', user_id: '@bot:a.example.com', connected: true }
const SERVER_B = { id: 2, homeserver: 'b.example.com', user_id: '@bot:b.example.com', connected: true }

describe('useConnectionStore', () => {
  describe('fetchServers', () => {
    it('populates servers from API', async () => {
      vi.mocked(api.listServers).mockResolvedValue({ servers: [SERVER_A, SERVER_B] })
      const store = useConnectionStore()
      await store.fetchServers()
      expect(store.servers).toHaveLength(2)
      expect(store.loaded).toBe(true)
    })

    it('auto-selects first server when none selected', async () => {
      vi.mocked(api.listServers).mockResolvedValue({ servers: [SERVER_A, SERVER_B] })
      const store = useConnectionStore()
      await store.fetchServers()
      expect(store.activeServerId).toBe(1)
    })

    it('resets activeServerId when active server is removed', async () => {
      vi.mocked(api.listServers).mockResolvedValue({ servers: [SERVER_B] })
      const store = useConnectionStore()
      store.activeServerId = 99 // server that no longer exists
      await store.fetchServers()
      // Should auto-select first available
      expect(store.activeServerId).toBe(2)
    })

    it('sets error on failure', async () => {
      vi.mocked(api.listServers).mockRejectedValue(new Error('Network error'))
      const store = useConnectionStore()
      await store.fetchServers()
      expect(store.error).toBe('Network error')
    })
  })

  describe('setActiveServer', () => {
    it('updates activeServerId', () => {
      const store = useConnectionStore()
      store.servers = [SERVER_A, SERVER_B]
      store.setActiveServer(2)
      expect(store.activeServerId).toBe(2)
    })

    it('clears command history when switching', () => {
      const store = useConnectionStore()
      const cmdStore = useCommandStore()
      store.servers = [SERVER_A, SERVER_B]
      cmdStore.history = [{ id: 1, command: 'test', response: 'ok', timestamp: new Date(), success: true }]
      store.setActiveServer(2)
      expect(cmdStore.history).toHaveLength(0)
    })
  })

  describe('addServer', () => {
    it('adds server and selects it', async () => {
      vi.mocked(api.addServer).mockResolvedValue({ id: 3, homeserver: 'c.example.com', user_id: '@bot:c.example.com' })
      const store = useConnectionStore()
      await store.addServer({ homeserver: 'c.example.com', username: 'bot', password: 'pass', room_id: '!room:c.example.com' })
      expect(store.servers).toHaveLength(1)
      expect(store.activeServerId).toBe(3)
    })

    it('sets error and rethrows on failure', async () => {
      vi.mocked(api.addServer).mockRejectedValue(new Error('Connection failed'))
      const store = useConnectionStore()
      await expect(store.addServer({ homeserver: 'bad', username: 'x', password: 'x', room_id: 'x' })).rejects.toThrow('Connection failed')
      expect(store.error).toBe('Connection failed')
      expect(store.loading).toBe(false)
    })
  })

  describe('removeServer', () => {
    it('removes server from list', async () => {
      vi.mocked(api.removeServer).mockResolvedValue({ removed: true })
      const store = useConnectionStore()
      store.servers = [SERVER_A, SERVER_B]
      store.activeServerId = 1
      await store.removeServer(2)
      expect(store.servers).toHaveLength(1)
      expect(store.activeServerId).toBe(1) // unchanged
    })

    it('reselects first server when active one is removed', async () => {
      vi.mocked(api.removeServer).mockResolvedValue({ removed: true })
      const store = useConnectionStore()
      store.servers = [SERVER_A, SERVER_B]
      store.activeServerId = 1
      await store.removeServer(1)
      expect(store.activeServerId).toBe(2)
    })

    it('sets activeServerId to null when last server is removed', async () => {
      vi.mocked(api.removeServer).mockResolvedValue({ removed: true })
      const store = useConnectionStore()
      store.servers = [SERVER_A]
      store.activeServerId = 1
      await store.removeServer(1)
      expect(store.activeServerId).toBeNull()
    })
  })

  describe('computed properties', () => {
    it('activeServer returns the active server object', () => {
      const store = useConnectionStore()
      store.servers = [SERVER_A, SERVER_B]
      store.activeServerId = 2
      expect(store.activeServer).toEqual(SERVER_B)
    })

    it('connected is true when server is selected', () => {
      const store = useConnectionStore()
      store.servers = [SERVER_A]
      store.activeServerId = 1
      expect(store.connected).toBe(true)
    })

    it('connected is false when no server selected', () => {
      const store = useConnectionStore()
      expect(store.connected).toBe(false)
    })

    it('homeserver returns active server homeserver', () => {
      const store = useConnectionStore()
      store.servers = [SERVER_A]
      store.activeServerId = 1
      expect(store.homeserver).toBe('a.example.com')
    })

    it('userId returns active server user_id', () => {
      const store = useConnectionStore()
      store.servers = [SERVER_A]
      store.activeServerId = 1
      expect(store.userId).toBe('@bot:a.example.com')
    })
  })
})
