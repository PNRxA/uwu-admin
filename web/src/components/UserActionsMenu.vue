<script setup lang="ts">
import { useActionDialogs } from '@/composables/useActionDialogs'
import { Settings2 } from 'lucide-vue-next'
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

const props = defineProps<{ userId: string }>()
const emit = defineEmits<{ 'action-complete': [] }>()

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

// Action helpers
const uid = () => props.userId

// Simple confirm actions (only user_id arg)
const confirmActions = {
  suspend: () =>
    openConfirm('Suspend User', `Suspend ${uid()}? They will have read-only access.`, `users suspend ${uid()}`),
  unsuspend: () =>
    openConfirm('Unsuspend User', `Unsuspend ${uid()}?`, `users unsuspend ${uid()}`),
  lock: () =>
    openConfirm('Lock User', `Lock ${uid()}? They will lose all access.`, `users lock ${uid()}`),
  unlock: () =>
    openConfirm('Unlock User', `Unlock ${uid()}?`, `users unlock ${uid()}`),
  logout: () =>
    openConfirm('Force Logout', `Force logout all sessions for ${uid()}?`, `users logout ${uid()}`),
  enableLogin: () =>
    openConfirm('Enable Login', `Enable login for ${uid()}?`, `users enable-login ${uid()}`),
  disableLogin: () =>
    openConfirm('Disable Login', `Disable login for ${uid()}? Existing sessions will be kept.`, `users disable-login ${uid()}`),
  makeAdmin: () =>
    openConfirm('Make Admin', `Grant admin privileges to ${uid()}?`, `users make-user-admin ${uid()}`),
  deactivate: () =>
    openConfirm('Deactivate Account', `Permanently deactivate ${uid()}? This cannot be undone.`, `users deactivate ${uid()}`, true),
}

// Actions needing extra input
const inputActions = {
  resetPassword: () =>
    openInputDialog('Reset Password', `Reset password for ${uid()}.`, `users reset-password ${uid()}`, [
      { name: 'password', label: 'New Password', placeholder: 'Leave blank for auto-generated', required: false },
    ]),
  forceJoinRoom: () =>
    openInputDialog('Force Join Room', `Force ${uid()} to join a room.`, `users force-join-room ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
    ]),
  forceLeaveRoom: () =>
    openInputDialog('Force Leave Room', `Force ${uid()} to leave a room.`, `users force-leave-room ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
    ]),
  forceLeaveRemoteRoom: () =>
    openInputDialog('Force Leave Remote Room', `Force ${uid()} to leave a remote room.`, `users force-leave-remote-room ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
      { name: 'via', label: 'Via Server', placeholder: 'server.com (optional)', required: false },
    ]),
  forceDemote: () =>
    openInputDialog('Force Demote', `Drop power levels for ${uid()} in a room.`, `users force-demote ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
    ]),
  putRoomTag: () =>
    openInputDialog('Set Room Tag', `Set a room tag for ${uid()}.`, `users put-room-tag ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
      { name: 'tag', label: 'Tag Name', placeholder: 'e.g. m.favourite', required: true },
    ]),
  deleteRoomTag: () =>
    openInputDialog('Delete Room Tag', `Delete a room tag for ${uid()}.`, `users delete-room-tag ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
      { name: 'tag', label: 'Tag Name', placeholder: 'e.g. m.favourite', required: true },
    ]),
  getRoomTags: () =>
    openInputDialog('Get Room Tags', `Get room tags for ${uid()}.`, `users get-room-tags ${uid()}`, [
      { name: 'room_id', label: 'Room ID', placeholder: '!room:server.com', required: true },
    ]),
}

// Read-only actions
const readOnlyActions = {
  listJoinedRooms: () =>
    executeReadOnly('Joined Rooms', `users list-joined-rooms ${uid()}`),
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon-sm">
        <Settings2 class="size-4" />
        <span class="sr-only">Actions</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="start" class="w-52">
      <DropdownMenuLabel>User Actions</DropdownMenuLabel>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Account</DropdownMenuLabel>
        <DropdownMenuItem @click="inputActions.resetPassword()">Reset Password</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.makeAdmin()">Make Admin</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Account Status</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.suspend()">Suspend</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.unsuspend()">Unsuspend</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.lock()">Lock</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.unlock()">Unlock</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Session</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.logout()">Force Logout</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.enableLogin()">Enable Login</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.disableLogin()">Disable Login</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Rooms</DropdownMenuLabel>
        <DropdownMenuItem @click="readOnlyActions.listJoinedRooms()">List Joined Rooms</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceJoinRoom()">Force Join Room</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceLeaveRoom()">Force Leave Room</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceLeaveRemoteRoom()">Force Leave Remote Room</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceDemote()">Force Demote</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">Room Tags</DropdownMenuLabel>
        <DropdownMenuItem @click="inputActions.getRoomTags()">Get Room Tags</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.putRoomTag()">Set Room Tag</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.deleteRoomTag()">Delete Room Tag</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuItem
        class="text-destructive focus:text-destructive"
        @click="confirmActions.deactivate()"
      >
        Deactivate Account
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>

  <!-- Alert Dialog for simple confirm actions -->
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
    <DialogContent class="sm:max-w-4xl">
      <DialogHeader>
        <DialogTitle>{{ resultDialogTitle }}</DialogTitle>
        <DialogDescription>Results for {{ userId }}</DialogDescription>
      </DialogHeader>
      <ResponseDisplay :response="resultResponse" />
    </DialogContent>
  </Dialog>
</template>
