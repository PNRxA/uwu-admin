<script setup lang="ts">
import { useI18n } from 'vue-i18n'
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

const { t } = useI18n()

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
    openConfirm(t('users.actions.suspend'), t('users.actions.suspendDescription', { userId: uid() }), `users suspend ${uid()}`),
  unsuspend: () =>
    openConfirm(t('users.actions.unsuspend'), t('users.actions.unsuspendDescription', { userId: uid() }), `users unsuspend ${uid()}`),
  lock: () =>
    openConfirm(t('users.actions.lock'), t('users.actions.lockDescription', { userId: uid() }), `users lock ${uid()}`),
  unlock: () =>
    openConfirm(t('users.actions.unlock'), t('users.actions.unlockDescription', { userId: uid() }), `users unlock ${uid()}`),
  logout: () =>
    openConfirm(t('users.actions.forceLogout'), t('users.actions.forceLogoutDescription', { userId: uid() }), `users logout ${uid()}`),
  enableLogin: () =>
    openConfirm(t('users.actions.enableLogin'), t('users.actions.enableLoginDescription', { userId: uid() }), `users enable-login ${uid()}`),
  disableLogin: () =>
    openConfirm(t('users.actions.disableLogin'), t('users.actions.disableLoginDescription', { userId: uid() }), `users disable-login ${uid()}`),
  makeAdmin: () =>
    openConfirm(t('users.actions.makeAdmin'), t('users.actions.makeAdminDescription', { userId: uid() }), `users make-user-admin ${uid()}`),
  deactivate: () =>
    openConfirm(t('users.actions.deactivateAccount'), t('users.actions.deactivateDescription', { userId: uid() }), `users deactivate ${uid()}`, true),
}

// Actions needing extra input
const inputActions = {
  resetPassword: () =>
    openInputDialog(t('users.actions.resetPassword'), t('users.actions.resetPasswordDescription', { userId: uid() }), `users reset-password ${uid()}`, [
      { name: 'password', label: t('users.actions.newPassword'), placeholder: t('users.actions.newPasswordPlaceholder'), required: false },
    ]),
  forceJoinRoom: () =>
    openInputDialog(t('users.actions.forceJoinRoom'), t('users.actions.forceJoinRoomDescription', { userId: uid() }), `users force-join-room ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
    ]),
  forceLeaveRoom: () =>
    openInputDialog(t('users.actions.forceLeaveRoom'), t('users.actions.forceLeaveRoomDescription', { userId: uid() }), `users force-leave-room ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
    ]),
  forceLeaveRemoteRoom: () =>
    openInputDialog(t('users.actions.forceLeaveRemoteRoom'), t('users.actions.forceLeaveRemoteRoomDescription', { userId: uid() }), `users force-leave-remote-room ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
      { name: 'via', label: t('users.actions.viaServer'), placeholder: t('users.actions.viaServerPlaceholder'), required: false },
    ]),
  forceDemote: () =>
    openInputDialog(t('users.actions.forceDemote'), t('users.actions.forceDemoteDescription', { userId: uid() }), `users force-demote ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
    ]),
  putRoomTag: () =>
    openInputDialog(t('users.actions.setRoomTag'), t('users.actions.setRoomTagDescription', { userId: uid() }), `users put-room-tag ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
      { name: 'tag', label: t('users.actions.tagName'), placeholder: t('users.actions.tagNamePlaceholder'), required: true },
    ]),
  deleteRoomTag: () =>
    openInputDialog(t('users.actions.deleteRoomTag'), t('users.actions.deleteRoomTagDescription', { userId: uid() }), `users delete-room-tag ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
      { name: 'tag', label: t('users.actions.tagName'), placeholder: t('users.actions.tagNamePlaceholder'), required: true },
    ]),
  getRoomTags: () =>
    openInputDialog(t('users.actions.getRoomTags'), t('users.actions.getRoomTagsDescription', { userId: uid() }), `users get-room-tags ${uid()}`, [
      { name: 'room_id', label: t('users.actions.roomId'), placeholder: t('users.actions.roomIdPlaceholder'), required: true },
    ]),
}

// Read-only actions
const readOnlyActions = {
  listJoinedRooms: () =>
    executeReadOnly(t('users.actions.joinedRooms'), `users list-joined-rooms ${uid()}`),
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon-sm">
        <Settings2 class="size-4" />
        <span class="sr-only">{{ $t('common.actions') }}</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="start" class="w-52">
      <DropdownMenuLabel>{{ $t('users.actions.title') }}</DropdownMenuLabel>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('users.actions.account') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="inputActions.resetPassword()">{{ $t('users.actions.resetPassword') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.makeAdmin()">{{ $t('users.actions.makeAdmin') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('users.actions.accountStatus') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.suspend()">{{ $t('users.actions.suspend') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.unsuspend()">{{ $t('users.actions.unsuspend') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.lock()">{{ $t('users.actions.lock') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.unlock()">{{ $t('users.actions.unlock') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('users.actions.session') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.logout()">{{ $t('users.actions.forceLogout') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.enableLogin()">{{ $t('users.actions.enableLogin') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.disableLogin()">{{ $t('users.actions.disableLogin') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('users.actions.rooms') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="readOnlyActions.listJoinedRooms()">{{ $t('users.actions.listJoinedRooms') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceJoinRoom()">{{ $t('users.actions.forceJoinRoom') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceLeaveRoom()">{{ $t('users.actions.forceLeaveRoom') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceLeaveRemoteRoom()">{{ $t('users.actions.forceLeaveRemoteRoom') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.forceDemote()">{{ $t('users.actions.forceDemote') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('users.actions.roomTags') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="inputActions.getRoomTags()">{{ $t('users.actions.getRoomTags') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.putRoomTag()">{{ $t('users.actions.setRoomTag') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.deleteRoomTag()">{{ $t('users.actions.deleteRoomTag') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuItem
        class="text-destructive focus:text-destructive"
        @click="confirmActions.deactivate()"
      >
        {{ $t('users.actions.deactivateAccount') }}
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
        <AlertDialogCancel :disabled="executing">{{ $t('common.cancel') }}</AlertDialogCancel>
        <Button
          :class="alertDestructive ? 'bg-destructive text-white hover:bg-destructive/90' : ''"
          :disabled="executing"
          @click="executeConfirm"
        >
          {{ executing ? $t('common.executing') : $t('common.confirm') }}
        </Button>
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
            {{ executing ? $t('common.executing') : $t('common.execute') }}
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
        <DialogDescription>{{ $t('users.actions.resultsFor', { userId }) }}</DialogDescription>
      </DialogHeader>
      <ResponseDisplay :response="resultResponse" />
    </DialogContent>
  </Dialog>
</template>
