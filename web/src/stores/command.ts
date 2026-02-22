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

    try {
      const res = await api.command(command)
      entry.response = res.response
      entry.success = true
    } catch (e) {
      entry.response = e instanceof Error ? e.message : 'Command failed'
      entry.success = false
    } finally {
      loading.value = false
    }

    return entry
  }

  function clear() {
    history.value = []
  }

  return { history, loading, execute, clear }
})
