import { ref } from 'vue'
import { defineStore } from 'pinia'
import { useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export const useConnectionStore = defineStore('connection', () => {
  const queryClient = useQueryClient()
  const connected = ref(false)
  const homeserver = ref('')
  const userId = ref('')
  const loading = ref(false)
  const error = ref('')

  async function connect(params: {
    homeserver: string
    username: string
    password: string
    room_id: string
  }) {
    loading.value = true
    error.value = ''
    try {
      const res = await api.connect(params)
      connected.value = true
      homeserver.value = res.homeserver
      userId.value = res.user_id
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Connection failed'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function disconnect() {
    await api.disconnect()
    connected.value = false
    homeserver.value = ''
    userId.value = ''
    queryClient.clear()
  }

  async function checkStatus() {
    try {
      const res = await api.status()
      connected.value = res.connected
      homeserver.value = res.homeserver ?? ''
      userId.value = res.user_id ?? ''
    } catch {
      connected.value = false
    }
  }

  return { connected, homeserver, userId, loading, error, connect, disconnect, checkStatus }
})
