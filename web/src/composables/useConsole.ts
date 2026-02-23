import { ref, nextTick, watch } from 'vue'
import { useCommandStore } from '@/stores/command'
import { validateCommand } from '@/composables/useCommandAutocomplete'
import { sanitizeHtml } from '@/lib/sanitize'
import type CommandAutocomplete from '@/components/CommandAutocomplete.vue'

export { sanitizeHtml }

export function useConsole(scrollTargetId: string) {
  const commandStore = useCommandStore()
  const commandInput = ref('')
  const submittedError = ref<string | null>(null)
  const autocompleteRef = ref<InstanceType<typeof CommandAutocomplete> | null>(null)

  watch(commandInput, () => {
    submittedError.value = null
  })

  async function sendCommand() {
    const cmd = commandInput.value.trim()
    if (!cmd) return
    const result = validateCommand(cmd)
    if (!result.valid) {
      submittedError.value = result.error ?? 'Invalid command'
      return
    }
    submittedError.value = null
    commandInput.value = ''
    await commandStore.execute(cmd)
    await nextTick()
    const el = document.getElementById(scrollTargetId)
    el?.scrollIntoView({ behavior: 'smooth' })
    autocompleteRef.value?.focus()
  }

  function formatTime(date: Date) {
    return date.toLocaleTimeString()
  }

  return {
    commandStore,
    commandInput,
    submittedError,
    autocompleteRef,
    sendCommand,
    formatTime,
    sanitizeHtml,
  }
}
