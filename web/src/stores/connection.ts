import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import { useCommandStore } from '@/stores/command'
import i18n from '@/i18n'

const t = i18n.global.t

export interface ServerInfo {
  id: number
  homeserver: string
  user_id: string
  connected: boolean
}

export const useConnectionStore = defineStore('connection', () => {
  const queryClient = useQueryClient()
  const servers = ref<ServerInfo[]>([])
  const activeServerId = ref<number | null>(null)
  const loading = ref(false)
  const error = ref('')
  const loaded = ref(false)

  const activeServer = computed(() =>
    servers.value.find((s) => s.id === activeServerId.value) ?? null,
  )

  const connected = computed(() => activeServer.value !== null)

  const homeserver = computed(() => activeServer.value?.homeserver ?? '')

  const userId = computed(() => activeServer.value?.user_id ?? '')

  async function fetchServers() {
    try {
      const res = await api.listServers()
      servers.value = res.servers
      // If active server was removed, reset
      if (activeServerId.value !== null && !servers.value.find((s) => s.id === activeServerId.value)) {
        activeServerId.value = null
      }
      // Auto-select first server if none selected
      if (activeServerId.value === null && servers.value.length > 0) {
        activeServerId.value = servers.value[0]?.id ?? null
      }
      loaded.value = true
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('connection.loadFailed')
    }
  }

  function setActiveServer(id: number) {
    activeServerId.value = id
  }

  async function addServer(params: {
    homeserver: string
    username: string
    password: string
    room_id: string
  }) {
    loading.value = true
    error.value = ''
    try {
      const res = await api.addServer(params)
      servers.value.push({
        id: res.id,
        homeserver: res.homeserver,
        user_id: res.user_id,
        connected: true,
      })
      setActiveServer(res.id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('connection.addFailed')
      throw e
    } finally {
      loading.value = false
    }
  }

  async function removeServer(id: number) {
    await api.removeServer(id)
    servers.value = servers.value.filter((s) => s.id !== id)
    if (activeServerId.value === id) {
      activeServerId.value = servers.value[0]?.id ?? null
    }
    queryClient.removeQueries({
      predicate: (query) => query.queryKey[query.queryKey.length - 1] === id,
    })
    useCommandStore().clearServer(id)
  }

  function reset() {
    servers.value = []
    activeServerId.value = null
    loading.value = false
    error.value = ''
    loaded.value = false
    queryClient.clear()
  }

  return {
    servers,
    activeServerId,
    activeServer,
    connected,
    homeserver,
    userId,
    loading,
    error,
    loaded,
    fetchServers,
    setActiveServer,
    addServer,
    removeServer,
    reset,
  }
})
