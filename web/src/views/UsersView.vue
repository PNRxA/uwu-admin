<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useConnectionStore } from '@/stores/connection'
import { api } from '@/lib/api'
import { queryKeys } from '@/lib/query-keys'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableEmpty,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { toast } from 'vue-sonner'
import { Plus, RefreshCw } from 'lucide-vue-next'
import UserActionsMenu from '@/components/UserActionsMenu.vue'

const { t } = useI18n()
const queryClient = useQueryClient()
const connection = useConnectionStore()
const serverId = computed(() => connection.activeServerId)

const { data: usersResponse, isPending, isFetching, refetch } = useQuery({
  queryKey: computed(() => queryKeys.users(serverId.value!)),
  queryFn: () => api.command(serverId.value!, 'users list-users'),
  staleTime: 30_000,
  enabled: computed(() => serverId.value !== null),
})

const createDialogOpen = ref(false)
const newUsername = ref('')
const newPassword = ref('')
const creating = ref(false)

const users = computed(() => {
  if (!usersResponse.value) return []
  const { parsed, response } = usersResponse.value
  if (parsed.type === 'list') {
    return parsed.items
  }
  // Fallback to regex extraction
  const matches = response.match(/@[a-zA-Z0-9._=\-/]+:[a-zA-Z0-9.\-]+/g)
  return matches ? [...new Set(matches)] : []
})

async function createUser() {
  if (serverId.value === null) return
  creating.value = true
  try {
    const cmd = newPassword.value
      ? `users create-user ${newUsername.value} ${newPassword.value}`
      : `users create-user ${newUsername.value}`
    await api.command(serverId.value, cmd)
    toast.success(t('users.createdSuccess'))
    createDialogOpen.value = false
    newUsername.value = ''
    newPassword.value = ''
    await queryClient.invalidateQueries({ queryKey: queryKeys.users(serverId.value) })
  } catch (e) {
    toast.error(e instanceof Error ? e.message : t('users.createFailed'))
  } finally {
    creating.value = false
  }
}

function onActionComplete() {
  if (serverId.value !== null) {
    queryClient.invalidateQueries({ queryKey: queryKeys.users(serverId.value) })
  }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <h1 class="text-2xl font-bold">{{ $t('users.title') }}</h1>
        <Button variant="ghost" size="icon-sm" :disabled="isFetching || !serverId" @click="refetch()">
          <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
          <span class="sr-only">{{ $t('common.refresh') }}</span>
        </Button>
      </div>
      <Dialog v-model:open="createDialogOpen">
        <DialogTrigger as-child>
          <Button :disabled="!serverId">
            <Plus class="size-4" />
            {{ $t('users.createUser') }}
          </Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{{ $t('users.createUser') }}</DialogTitle>
            <DialogDescription>{{ $t('users.createUserDescription') }}</DialogDescription>
          </DialogHeader>
          <form class="flex flex-col gap-4" @submit.prevent="createUser">
            <div class="flex flex-col gap-2">
              <Label for="new-username">{{ $t('users.username') }}</Label>
              <Input id="new-username" v-model="newUsername" placeholder="username" required />
            </div>
            <div class="flex flex-col gap-2">
              <Label for="new-password">{{ $t('users.passwordOptional') }}</Label>
              <Input id="new-password" v-model="newPassword" type="password" />
            </div>
            <DialogFooter>
              <Button type="submit" :disabled="creating">
                {{ creating ? $t('users.creating') : $t('users.create') }}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>

    <div v-if="!serverId" class="text-muted-foreground text-sm">
      {{ $t('common.noServerSelected') }}
    </div>

    <Card v-else>
      <CardHeader>
        <CardTitle>{{ $t('users.userList') }}</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="isPending" class="h-32 w-full" />
        <Table v-else>
          <TableHeader>
            <TableRow>
              <TableHead class="w-12">{{ $t('common.actions') }}</TableHead>
              <TableHead>{{ $t('users.userId') }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableEmpty v-if="users.length === 0" :colspan="2">
              {{ $t('users.noUsersFound') }}
            </TableEmpty>
            <TableRow v-for="user in users" :key="user">
              <TableCell>
                <UserActionsMenu :user-id="user" @action-complete="onActionComplete" />
              </TableCell>
              <TableCell class="font-mono text-sm">{{ user }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
