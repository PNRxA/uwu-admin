import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/lib/api'
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
  let nextId = 1

  async function execute(command: string) {
    const connection = useConnectionStore()
    const serverId = connection.activeServerId
    if (serverId === null) {
      throw new Error(t('command.noServerSelected'))
    }

    loading.value = true
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

    // Access through the reactive array so mutations trigger reactivity
    const entry = history.value.find(e => e.id === entryId)!

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

  function clear() {
    history.value = []
  }

  const panelOpen = ref(false)

  function togglePanel() {
    panelOpen.value = !panelOpen.value
  }

  return { history, loading, execute, clear, panelOpen, togglePanel }
})
