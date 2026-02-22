import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/lib/api'

export interface CommandEntry {
  id: number
  command: string
  response: string
  timestamp: Date
  success: boolean
}

export const useCommandStore = defineStore('command', () => {
  const history = ref<CommandEntry[]>([])
  const loading = ref(false)
  let nextId = 1

  async function execute(command: string) {
    loading.value = true
    const entry: CommandEntry = {
      id: nextId++,
      command,
      response: '',
      timestamp: new Date(),
      success: false,
    }
    history.value.push(entry)
    const idx = history.value.length - 1

    try {
      const res = await api.command(command)
      history.value[idx].response = res.response
      history.value[idx].success = true
    } catch (e) {
      history.value[idx].response = e instanceof Error ? e.message : 'Command failed'
      history.value[idx].success = false
    } finally {
      loading.value = false
    }

    return history.value[idx]
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
