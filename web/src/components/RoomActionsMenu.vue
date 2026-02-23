<script setup lang="ts">
import { ref } from 'vue'
import { api } from '@/lib/api'
import { useConnectionStore } from '@/stores/connection'
import { toast } from 'vue-sonner'
import { Ellipsis } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import ResponseDisplay from '@/components/ResponseDisplay.vue'

const props = defineProps<{ roomId: string }>()
const emit = defineEmits<{ 'action-complete': [] }>()

const connection = useConnectionStore()

// Alert dialog state
const alertOpen = ref(false)
const alertTitle = ref('')
const alertDescription = ref('')
const alertCommand = ref('')
const alertDestructive = ref(false)
const executing = ref(false)

// Input dialog state
const inputDialogOpen = ref(false)
const inputDialogTitle = ref('')
const inputDialogDescription = ref('')
const inputDialogCommand = ref('')
const inputFields = ref<{ name: string; label: string; placeholder: string; required: boolean; value: string }[]>([])

// Result dialog state
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
    emit('action-complete')
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
  fields: { name: string; label: string; placeholder: string; required: boolean }[],
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
  const args = inputFields.value.map((f) => f.value.trim()).filter(Boolean)
  const command = `${inputDialogCommand.value} ${args.join(' ')}`.trim()

  executing.value = true
  try {
    await api.command(connection.activeServerId, command)
    toast.success(`${inputDialogTitle.value} completed`)
    inputDialogOpen.value = false
    emit('action-complete')
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

const rid = () => props.roomId

// Read-only actions
const readOnlyActions = {
  listMembers: () =>
    executeReadOnly('Joined Members', `rooms info list-joined-members ${rid()}`),
  viewTopic: () =>
    executeReadOnly('Room Topic', `rooms info view-room-topic ${rid()}`),
  checkExists: () =>
    executeReadOnly('Room Exists', `rooms exists ${rid()}`),
  listAliases: () =>
    executeReadOnly('Room Aliases', `rooms alias list ${rid()}`),
  roomInfo: async () => {
    if (connection.activeServerId === null) return
    resultDialogTitle.value = 'Room Info'
    resultResponse.value = 'Loading...'
    resultDialogOpen.value = true
    try {
      const res = await api.roomInfo(connection.activeServerId, rid())
      resultResponse.value = res.response
    } catch (e) {
      resultResponse.value = e instanceof Error ? e.message : 'Failed to fetch room info'
    }
  },
}

// Confirm actions (only room_id arg)
const confirmActions = {
  publish: () =>
    openConfirm('Publish Room', `Publish ${rid()} to the room directory?`, `rooms directory publish ${rid()}`),
  unpublish: () =>
    openConfirm('Unpublish Room', `Remove ${rid()} from the room directory?`, `rooms directory unpublish ${rid()}`),
  ban: () =>
    openConfirm('Ban Room', `Ban room ${rid()}? This will prevent users from joining.`, `rooms moderation ban-room ${rid()}`, true),
  unban: () =>
    openConfirm('Unban Room', `Unban room ${rid()}?`, `rooms moderation unban-room ${rid()}`),
}

// Actions needing extra input
const inputActions = {
  setAlias: () =>
    openInputDialog('Set Room Alias', `Set an alias for ${rid()}.`, `rooms alias set ${rid()}`, [
      { name: 'alias', label: 'Alias Localpart', placeholder: 'e.g. my-room', required: true },
    ]),
  removeAlias: () =>
    openInputDialog('Remove Room Alias', `Remove an alias from ${rid()}.`, 'rooms alias remove', [
      { name: 'alias', label: 'Alias Localpart', placeholder: 'e.g. my-room', required: true },
    ]),
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon-sm">
        <Ellipsis class="size-4" />
        <span class="sr-only">Actions</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="w-52">
      <DropdownMenuLabel>Room Actions</DropdownMenuLabel>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Info</DropdownMenuLabel>
        <DropdownMenuItem @click="readOnlyActions.roomInfo()">Room Info</DropdownMenuItem>
        <DropdownMenuItem @click="readOnlyActions.listMembers()">List Members</DropdownMenuItem>
        <DropdownMenuItem @click="readOnlyActions.viewTopic()">View Topic</DropdownMenuItem>
        <DropdownMenuItem @click="readOnlyActions.checkExists()">Check Exists</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Aliases</DropdownMenuLabel>
        <DropdownMenuItem @click="readOnlyActions.listAliases()">List Aliases</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.setAlias()">Set Alias</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.removeAlias()">Remove Alias</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Directory</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.publish()">Publish</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.unpublish()">Unpublish</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Moderation</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.unban()">Unban Room</DropdownMenuItem>
        <DropdownMenuItem
          class="text-destructive focus:text-destructive"
          @click="confirmActions.ban()"
        >
          Ban Room
        </DropdownMenuItem>
      </DropdownMenuGroup>
    </DropdownMenuContent>
  </DropdownMenu>

  <!-- Alert Dialog for confirm actions -->
  <AlertDialog v-model:open="alertOpen">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>{{ alertTitle }}</AlertDialogTitle>
        <AlertDialogDescription>{{ alertDescription }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel :disabled="executing">Cancel</AlertDialogCancel>
        <AlertDialogAction
          :class="alertDestructive ? 'bg-destructive text-white hover:bg-destructive/90' : ''"
          :disabled="executing"
          @click.prevent="executeConfirm"
        >
          {{ executing ? 'Executing...' : 'Confirm' }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>

  <!-- Input Dialog for actions with extra args -->
  <Dialog v-model:open="inputDialogOpen">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{ inputDialogTitle }}</DialogTitle>
        <DialogDescription>{{ inputDialogDescription }}</DialogDescription>
      </DialogHeader>
      <form class="flex flex-col gap-4" @submit.prevent="executeInputDialog">
        <div v-for="field in inputFields" :key="field.name" class="flex flex-col gap-2">
          <Label :for="`input-${field.name}`">{{ field.label }}<span v-if="field.required" class="text-destructive"> *</span></Label>
          <Input
            :id="`input-${field.name}`"
            v-model="field.value"
            :placeholder="field.placeholder"
            :required="field.required"
          />
        </div>
        <DialogFooter>
          <Button type="submit" :disabled="executing">
            {{ executing ? 'Executing...' : 'Execute' }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>

  <!-- Result Dialog for read-only actions -->
  <Dialog v-model:open="resultDialogOpen">
    <DialogContent class="max-w-2xl">
      <DialogHeader>
        <DialogTitle>{{ resultDialogTitle }}</DialogTitle>
        <DialogDescription>Results for {{ roomId }}</DialogDescription>
      </DialogHeader>
      <ResponseDisplay :response="resultResponse" />
    </DialogContent>
  </Dialog>
</template>
