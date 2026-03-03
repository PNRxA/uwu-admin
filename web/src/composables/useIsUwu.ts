import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'

export function useIsUwu() {
  const settingsStore = useSettingsStore()

  // Fall back to localStorage when the settings store hasn't been populated
  // (e.g. on the login page before authentication)
  const isUwu = computed(() => {
    const storeVal = settingsStore.settings.flavour_text
    if (storeVal !== undefined) return storeVal !== 'false'
    return localStorage.getItem('uwu-flavour-text') !== 'false'
  })

  return { isUwu }
}
