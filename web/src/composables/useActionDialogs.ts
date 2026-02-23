import { ref } from 'vue'
import { api } from '@/lib/api'
import { useConnectionStore } from '@/stores/connection'
import { toast } from 'vue-sonner'

export interface InputField {
  name: string
  label: string
  placeholder: string
  required: boolean
  value: string
}

export function useActionDialogs(onComplete?: () => void) {
  const connection = useConnectionStore()

  // Alert dialog state (simple confirm actions)
  const alertOpen = ref(false)
  const alertTitle = ref('')
  const alertDescription = ref('')
  const alertCommand = ref('')
  const alertDestructive = ref(false)
  const executing = ref(false)

  // Input dialog state (actions needing extra args)
  const inputDialogOpen = ref(false)
  const inputDialogTitle = ref('')
  const inputDialogDescription = ref('')
  const inputDialogCommand = ref('')
  const inputFields = ref<InputField[]>([])

  // Result dialog state (read-only actions)
  const resultDialogOpen = ref(false)
  const resultDialogTitle = ref('')
  const resultResponse = ref('')

  function openConfirm(title: string, description: string, command: string, destructive = false) {
    alertTitle.value = title
    alertDescription.value = description
    alertCommand.value = command
    alertDestructive.value = destructive
    alertOpen.value = true
  }

  async function executeConfirm() {
    if (connection.activeServerId === null) return
    executing.value = true
    try {
      await api.command(connection.activeServerId, alertCommand.value)
      toast.success(`${alertTitle.value} completed`)
      alertOpen.value = false
      onComplete?.()
    } catch (e) {
      toast.error(e instanceof Error ? e.message : `Failed: ${alertTitle.value}`)
    } finally {
      executing.value = false
    }
  }

  function openInputDialog(
    title: string,
    description: string,
    commandPrefix: string,
    fields: Omit<InputField, 'value'>[],
  ) {
    inputDialogTitle.value = title
    inputDialogDescription.value = description
    inputDialogCommand.value = commandPrefix
    inputFields.value = fields.map((f) => ({ ...f, value: '' }))
    inputDialogOpen.value = true
  }

  async function executeInputDialog() {
    if (connection.activeServerId === null) return
    const missing = inputFields.value.find((f) => f.required && !f.value.trim())
    if (missing) {
      toast.error(`${missing.label} is required`)
      return
    }
    const badField = inputFields.value.find((f) => f.value.trim() && /\s/.test(f.value.trim()))
    if (badField) {
      toast.error(`${badField.label} must not contain whitespace`)
      return
    }
    const args = inputFields.value.map((f) => f.value.trim()).filter(Boolean)
    const command = `${inputDialogCommand.value} ${args.join(' ')}`.trim()

    executing.value = true
    try {
      await api.command(connection.activeServerId, command)
      toast.success(`${inputDialogTitle.value} completed`)
      inputDialogOpen.value = false
      onComplete?.()
    } catch (e) {
      toast.error(e instanceof Error ? e.message : `Failed: ${inputDialogTitle.value}`)
    } finally {
      executing.value = false
    }
  }

  async function executeReadOnly(title: string, command: string) {
    if (connection.activeServerId === null) return
    resultDialogTitle.value = title
    resultResponse.value = 'Loading...'
    resultDialogOpen.value = true
    try {
      const res = await api.command(connection.activeServerId, command)
      resultResponse.value = res.response
    } catch (e) {
      resultResponse.value = e instanceof Error ? e.message : 'Failed to execute command'
    }
  }

  return {
    // Alert dialog
    alertOpen,
    alertTitle,
    alertDescription,
    alertDestructive,
    executing,
    openConfirm,
    executeConfirm,
    // Input dialog
    inputDialogOpen,
    inputDialogTitle,
    inputDialogDescription,
    inputFields,
    openInputDialog,
    executeInputDialog,
    // Result dialog
    resultDialogOpen,
    resultDialogTitle,
    resultResponse,
    executeReadOnly,
  }
}
