import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api, type CommandResponse } from '@/lib/api'
import { useConnectionStore } from '@/stores/connection'
import i18n from '@/i18n'

const t = i18n.global.t

export interface CommandEntry {
  id: number
  command: string
  response: string
  timestamp: Date
  success: boolean | null
}

const MAX_HISTORY = 500

export const useCommandStore = defineStore('command', () => {
  const history = ref<CommandEntry[]>([])
  const loading = ref(false)
  const unreadCount = ref(0)
  let nextId = 1

  function addEntry(command: string): CommandEntry {
    const entryId = nextId++
    history.value.push({
      id: entryId,
      command,
      response: '',
      timestamp: new Date(),
      success: null,
    })
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(-MAX_HISTORY)
    }
    if (!panelOpen.value) unreadCount.value++
    return history.value[history.value.length - 1]
  }

  async function execute(command: string) {
    const connection = useConnectionStore()
    const serverId = connection.activeServerId
    if (serverId === null) {
      throw new Error(t('command.noServerSelected'))
    }

    loading.value = true
    const entry = addEntry(command)

    try {
      const res = await api.command(serverId, command)
      entry.response = res.response
      entry.success = true
    } catch (e) {
      entry.response = e instanceof Error ? e.message : t('command.commandFailed')
      entry.success = false
    } finally {
      loading.value = false
    }

    return entry
  }

  async function query(command: string): Promise<CommandResponse> {
    const connection = useConnectionStore()
    const serverId = connection.activeServerId
    if (serverId === null) {
      throw new Error(t('command.noServerSelected'))
    }

    const entry = addEntry(command)

    try {
      const res = await api.command(serverId, command)
      entry.response = res.response
      entry.success = true
      return res
    } catch (e) {
      entry.response = e instanceof Error ? e.message : t('command.commandFailed')
      entry.success = false
      throw e
    }
  }

  function clear() {
    history.value = []
    unreadCount.value = 0
  }

  const panelOpen = ref(false)

  function togglePanel() {
    panelOpen.value = !panelOpen.value
  }

  return { history, loading, unreadCount, execute, query, clear, panelOpen, togglePanel }
})
