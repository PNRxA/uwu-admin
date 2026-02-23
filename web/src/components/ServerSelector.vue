<script setup lang="ts">
import { ref } from 'vue'
import { useConnectionStore } from '@/stores/connection'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
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
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ChevronDown, Plus, Trash2, Server } from 'lucide-vue-next'
import { toast } from 'vue-sonner'

const connection = useConnectionStore()

// Add server dialog
const addDialogOpen = ref(false)
const homeserver = ref('')
const username = ref('')
const password = ref('')
const roomId = ref('')

async function onAddServer() {
  try {
    await connection.addServer({
      homeserver: homeserver.value,
      username: username.value,
      password: password.value,
      room_id: roomId.value,
    })
    toast.success('Server added successfully')
    addDialogOpen.value = false
    homeserver.value = ''
    username.value = ''
    password.value = ''
    roomId.value = ''
  } catch {
    // error is set in store
  }
}

// Remove server dialog
const removeDialogOpen = ref(false)
const serverToRemove = ref<{ id: number; homeserver: string } | null>(null)

function confirmRemove(id: number, hs: string) {
  serverToRemove.value = { id, homeserver: hs }
  removeDialogOpen.value = true
}

async function onRemoveServer() {
  if (!serverToRemove.value) return
  try {
    await connection.removeServer(serverToRemove.value.id)
    toast.success('Server removed')
    removeDialogOpen.value = false
  } catch (e) {
    toast.error(e instanceof Error ? e.message : 'Failed to remove server')
  }
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" class="h-auto gap-2 py-1.5 font-normal">
        <Server class="size-4 shrink-0 text-muted-foreground" />
        <template v-if="connection.activeServer">
          <div class="flex flex-col items-start min-w-0">
            <span class="text-xs truncate">{{ connection.homeserver }}</span>
            <span class="text-[0.65rem] leading-tight text-muted-foreground truncate">{{ connection.userId }}</span>
          </div>
        </template>
        <span v-else class="text-sm text-muted-foreground">No server selected</span>
        <ChevronDown class="size-3.5 shrink-0 text-muted-foreground" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="start" class="w-72">
      <DropdownMenuLabel>Servers</DropdownMenuLabel>
      <DropdownMenuSeparator />

      <DropdownMenuItem
        v-for="server in connection.servers"
        :key="server.id"
        class="flex items-center justify-between gap-2"
        @click="connection.setActiveServer(server.id)"
      >
        <div class="flex flex-col min-w-0">
          <span class="text-sm truncate">{{ server.homeserver }}</span>
          <span class="text-xs text-muted-foreground truncate">{{ server.user_id }}</span>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          class="shrink-0"
          @click.stop="confirmRemove(server.id, server.homeserver)"
        >
          <Trash2 class="size-3.5 text-muted-foreground" />
        </Button>
      </DropdownMenuItem>

      <DropdownMenuSeparator v-if="connection.servers.length > 0" />

      <DropdownMenuItem @click="addDialogOpen = true">
        <Plus class="size-4" />
        Add Server
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>

  <!-- Add Server Dialog -->
  <Dialog v-model:open="addDialogOpen">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Add Server</DialogTitle>
        <DialogDescription>Connect to a Continuwuity homeserver admin room.</DialogDescription>
      </DialogHeader>
      <form class="flex flex-col gap-4" @submit.prevent="onAddServer">
        <Alert v-if="connection.error" variant="destructive">
          <AlertDescription>{{ connection.error }}</AlertDescription>
        </Alert>

        <div class="flex flex-col gap-2">
          <Label for="add-homeserver">Homeserver URL</Label>
          <Input
            id="add-homeserver"
            v-model="homeserver"
            placeholder="https://matrix.example.com"
            required
          />
        </div>

        <div class="flex flex-col gap-2">
          <Label for="add-username">Bot Username</Label>
          <Input
            id="add-username"
            v-model="username"
            placeholder="admin-bot"
            required
          />
        </div>

        <div class="flex flex-col gap-2">
          <Label for="add-password">Bot Password</Label>
          <Input
            id="add-password"
            v-model="password"
            type="password"
            required
          />
        </div>

        <div class="flex flex-col gap-2">
          <Label for="add-room-id">Admin Room ID</Label>
          <Input
            id="add-room-id"
            v-model="roomId"
            placeholder="!roomid:example.com"
            required
          />
        </div>

        <DialogFooter>
          <Button type="submit" :disabled="connection.loading">
            {{ connection.loading ? 'Connecting...' : 'Connect' }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>

  <!-- Remove Server Confirmation -->
  <AlertDialog v-model:open="removeDialogOpen">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Remove Server</AlertDialogTitle>
        <AlertDialogDescription>
          Remove {{ serverToRemove?.homeserver }}? This will disconnect the server.
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction
          class="bg-destructive text-white hover:bg-destructive/90"
          @click="onRemoveServer"
        >
          Remove
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
