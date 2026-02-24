import { ref, nextTick, watch, onMounted } from 'vue'
import { useCommandStore } from '@/stores/command'
import { validateCommand } from '@/composables/useCommandAutocomplete'
import { sanitizeHtml } from '@/lib/sanitize'
import type CommandAutocomplete from '@/components/CommandAutocomplete.vue'

export { sanitizeHtml }

interface ConsoleOptions {
  autoScrollEnabled?: () => boolean
}

export function useConsole(scrollTargetId: string, options?: ConsoleOptions) {
  const commandStore = useCommandStore()
  const commandInput = ref('')
  const submittedError = ref<string | null>(null)
  const autocompleteRef = ref<InstanceType<typeof CommandAutocomplete> | null>(null)
  const shouldAutoScroll = options?.autoScrollEnabled ?? (() => true)

  function scrollToBottom(behavior: ScrollBehavior = 'smooth') {
    nextTick(() => {
      // Double rAF ensures layout has settled after collapsible animations
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          const el = document.getElementById(scrollTargetId)
          el?.scrollIntoView({ behavior })
        })
      })
    })
  }

  onMounted(() => {
    if (shouldAutoScroll()) scrollToBottom('instant')
  })

  watch(() => commandStore.history, () => {
    if (shouldAutoScroll()) scrollToBottom()
  }, { deep: true })

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
    scrollToBottom,
    sendCommand,
    formatTime,
    sanitizeHtml,
  }
}
