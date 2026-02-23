<script setup lang="ts">
import { useConnectionStore } from '@/stores/connection'
import { useActionDialogs } from '@/composables/useActionDialogs'
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

const {
  alertOpen,
  alertTitle,
  alertDescription,
  alertDestructive,
  executing,
  openConfirm,
  executeConfirm,
  inputDialogOpen,
  inputDialogTitle,
  inputDialogDescription,
  inputFields,
  openInputDialog,
  executeInputDialog,
  resultDialogOpen,
  resultDialogTitle,
  resultResponse,
  executeReadOnly,
} = useActionDialogs(() => emit('action-complete'))

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
