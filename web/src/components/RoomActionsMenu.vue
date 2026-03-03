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

const props = defineProps<{ roomId: string }>()
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

const rid = () => props.roomId

// Read-only actions
const readOnlyActions = {
  listMembers: () =>
    executeReadOnly(t('rooms.actions.joinedMembers'), `rooms info list-joined-members ${rid()}`),
  viewTopic: () =>
    executeReadOnly(t('rooms.actions.roomTopic'), `rooms info view-room-topic ${rid()}`),
  checkExists: () =>
    executeReadOnly(t('rooms.actions.roomExists'), `rooms exists ${rid()}`),
  listAliases: () =>
    executeReadOnly(t('rooms.actions.roomAliases'), `rooms alias list ${rid()}`),
}

// Confirm actions (only room_id arg)
const confirmActions = {
  publish: () =>
    openConfirm(t('rooms.actions.publish'), t('rooms.actions.publishDescription', { roomId: rid() }), `rooms directory publish ${rid()}`),
  unpublish: () =>
    openConfirm(t('rooms.actions.unpublish'), t('rooms.actions.unpublishDescription', { roomId: rid() }), `rooms directory unpublish ${rid()}`),
  ban: () =>
    openConfirm(t('rooms.actions.banRoom'), t('rooms.actions.banDescription', { roomId: rid() }), `rooms moderation ban-room ${rid()}`, true),
  unban: () =>
    openConfirm(t('rooms.actions.unbanRoom'), t('rooms.actions.unbanDescription', { roomId: rid() }), `rooms moderation unban-room ${rid()}`),
}

// Actions needing extra input
const inputActions = {
  setAlias: () =>
    openInputDialog(t('rooms.actions.setAlias'), t('rooms.actions.setAliasDescription', { roomId: rid() }), `rooms alias set ${rid()}`, [
      { name: 'alias', label: t('rooms.actions.aliasLocalpart'), placeholder: t('rooms.actions.aliasPlaceholder'), required: true },
    ]),
  removeAlias: () =>
    openInputDialog(t('rooms.actions.removeAlias'), t('rooms.actions.removeAliasDescription', { roomId: rid() }), 'rooms alias remove', [
      { name: 'alias', label: t('rooms.actions.aliasLocalpart'), placeholder: t('rooms.actions.aliasPlaceholder'), required: true },
    ]),
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
      <DropdownMenuLabel>{{ $t('rooms.actions.title') }}</DropdownMenuLabel>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('rooms.actions.info') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="readOnlyActions.listMembers()">{{ $t('rooms.actions.listMembers') }}</DropdownMenuItem>
        <DropdownMenuItem @click="readOnlyActions.viewTopic()">{{ $t('rooms.actions.viewTopic') }}</DropdownMenuItem>
        <DropdownMenuItem @click="readOnlyActions.checkExists()">{{ $t('rooms.actions.checkExists') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('rooms.actions.aliases') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="readOnlyActions.listAliases()">{{ $t('rooms.actions.listAliases') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.setAlias()">{{ $t('rooms.actions.setAlias') }}</DropdownMenuItem>
        <DropdownMenuItem @click="inputActions.removeAlias()">{{ $t('rooms.actions.removeAlias') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('rooms.actions.directory') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.publish()">{{ $t('rooms.actions.publish') }}</DropdownMenuItem>
        <DropdownMenuItem @click="confirmActions.unpublish()">{{ $t('rooms.actions.unpublish') }}</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />

      <DropdownMenuGroup>
        <DropdownMenuLabel class="text-xs text-muted-foreground font-normal">{{ $t('rooms.actions.moderation') }}</DropdownMenuLabel>
        <DropdownMenuItem @click="confirmActions.unban()">{{ $t('rooms.actions.unbanRoom') }}</DropdownMenuItem>
        <DropdownMenuItem
          class="text-destructive focus:text-destructive"
          @click="confirmActions.ban()"
        >
          {{ $t('rooms.actions.banRoom') }}
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
        <DialogDescription>{{ $t('rooms.actions.resultsFor', { roomId }) }}</DialogDescription>
      </DialogHeader>
      <ResponseDisplay :response="resultResponse" />
    </DialogContent>
  </Dialog>
</template>
