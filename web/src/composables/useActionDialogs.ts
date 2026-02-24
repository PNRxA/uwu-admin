import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConnectionStore } from '@/stores/connection'
import { useCommandStore } from '@/stores/command'
import { toast } from 'vue-sonner'

export interface InputField {
  name: string
  label: string
  placeholder: string
  required: boolean
  value: string
}

export function useActionDialogs(onComplete?: () => void) {
  const { t } = useI18n()
  const connection = useConnectionStore()
  const commandStore = useCommandStore()

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
      const entry = await commandStore.execute(alertCommand.value)
      if (entry.success) {
        toast.success(t('actionDialog.completed', { action: alertTitle.value }))
        alertOpen.value = false
        onComplete?.()
      } else {
        toast.error(entry.response || t('actionDialog.failed', { action: alertTitle.value }))
      }
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
      toast.error(t('actionDialog.fieldRequired', { field: missing.label }))
      return
    }
    const badField = inputFields.value.find((f) => f.value.trim() && /\s/.test(f.value.trim()))
    if (badField) {
      toast.error(t('actionDialog.noWhitespace', { field: badField.label }))
      return
    }
    const args = inputFields.value.map((f) => f.value.trim()).filter(Boolean)
    const command = `${inputDialogCommand.value} ${args.join(' ')}`.trim()

    executing.value = true
    try {
      const entry = await commandStore.execute(command)
      if (entry.success) {
        toast.success(t('actionDialog.completed', { action: inputDialogTitle.value }))
        inputDialogOpen.value = false
        onComplete?.()
      } else {
        toast.error(entry.response || t('actionDialog.failed', { action: inputDialogTitle.value }))
      }
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
      const res = await commandStore.query(command)
      resultResponse.value = res.response
    } catch (e) {
      resultResponse.value = e instanceof Error ? e.message : t('actionDialog.executeFailed')
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
