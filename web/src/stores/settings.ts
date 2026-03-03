import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/lib/api'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Record<string, string>>({})
  const loading = ref(false)
  const error = ref('')

  async function fetchSettings() {
    loading.value = true
    error.value = ''
    try {
      settings.value = await api.getSettings()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load settings'
    } finally {
      loading.value = false
    }
  }

  async function updateSetting(key: string, value: string) {
    error.value = ''
    try {
      settings.value = await api.updateSettings({ [key]: value })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update setting'
      throw e
    }
  }

  return { settings, loading, error, fetchSettings, updateSetting }
})
