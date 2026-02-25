import { computed, ref } from 'vue'
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
  const historyMap = ref(new Map<number, CommandEntry[]>())
  const unreadCountMap = ref(new Map<number, number>())
  const loadingMap = ref(new Map<number, boolean>())
  let nextId = 1

  const history = computed(() => {
    const connection = useConnectionStore()
    const id = connection.activeServerId
    if (id === null) return []
    return historyMap.value.get(id) ?? []
  })

  const loading = computed(() => {
    const connection = useConnectionStore()
    const id = connection.activeServerId
    if (id === null) return false
    return loadingMap.value.get(id) ?? false
  })

  const unreadCount = computed({
    get() {
      const connection = useConnectionStore()
      const id = connection.activeServerId
      if (id === null) return 0
      return unreadCountMap.value.get(id) ?? 0
    },
    set(val: number) {
      const connection = useConnectionStore()
      const id = connection.activeServerId
      if (id === null) return
      unreadCountMap.value.set(id, val)
    },
  })

  function addEntry(serverId: number, command: string): CommandEntry {
    const entryId = nextId++
    const entry: CommandEntry = {
      id: entryId,
      command,
      response: '',
      timestamp: new Date(),
      success: null,
    }

    if (!historyMap.value.has(serverId)) {
      historyMap.value.set(serverId, [])
    }
    const serverHistory = historyMap.value.get(serverId)!
    serverHistory.push(entry)

    if (serverHistory.length > MAX_HISTORY) {
      historyMap.value.set(serverId, serverHistory.slice(-MAX_HISTORY))
    }

    if (!panelOpen.value) {
      unreadCountMap.value.set(serverId, (unreadCountMap.value.get(serverId) ?? 0) + 1)
    }

    const currentHistory = historyMap.value.get(serverId)!
    return currentHistory[currentHistory.length - 1]!
  }

  async function execute(command: string) {
    const connection = useConnectionStore()
    const serverId = connection.activeServerId
    if (serverId === null) {
      throw new Error(t('command.noServerSelected'))
    }

    loadingMap.value.set(serverId, true)
    const entry = addEntry(serverId, command)

    try {
      const res = await api.command(serverId, command)
      entry.response = res.response
      entry.success = true
    } catch (e) {
      entry.response = e instanceof Error ? e.message : t('command.commandFailed')
      entry.success = false
    } finally {
      loadingMap.value.set(serverId, false)
    }

    return entry
  }

  async function query(command: string): Promise<CommandResponse> {
    const connection = useConnectionStore()
    const serverId = connection.activeServerId
    if (serverId === null) {
      throw new Error(t('command.noServerSelected'))
    }

    const entry = addEntry(serverId, command)

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
    const connection = useConnectionStore()
    const id = connection.activeServerId
    if (id === null) return
    historyMap.value.set(id, [])
    unreadCountMap.value.set(id, 0)
  }

  function clearServer(serverId: number) {
    historyMap.value.delete(serverId)
    unreadCountMap.value.delete(serverId)
    loadingMap.value.delete(serverId)
  }

  function clearAll() {
    historyMap.value.clear()
    unreadCountMap.value.clear()
    loadingMap.value.clear()
  }

  const panelOpen = ref(false)

  function togglePanel() {
    panelOpen.value = !panelOpen.value
  }

  return { history, loading, unreadCount, execute, query, clear, clearServer, clearAll, panelOpen, togglePanel }
})
