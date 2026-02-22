<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/lib/api'
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
import { Plus } from 'lucide-vue-next'
import UserActionsMenu from '@/components/UserActionsMenu.vue'

const usersResponse = ref('')
const loading = ref(true)
const createDialogOpen = ref(false)
const newUsername = ref('')
const newPassword = ref('')
const creating = ref(false)

const users = computed(() => {
  if (!usersResponse.value) return []
  const matches = usersResponse.value.match(/@[a-zA-Z0-9._=\-/]+:[a-zA-Z0-9.\-]+/g)
  return matches ? [...new Set(matches)] : []
})

async function fetchUsers() {
  loading.value = true
  try {
    const res = await api.listUsers()
    usersResponse.value = res.response
  } catch {
    usersResponse.value = ''
    toast.error('Failed to fetch users')
  } finally {
    loading.value = false
  }
}

async function createUser() {
  creating.value = true
  try {
    await api.createUser({
      username: newUsername.value,
      password: newPassword.value || undefined,
    })
    toast.success('User created successfully')
    createDialogOpen.value = false
    newUsername.value = ''
    newPassword.value = ''
    await fetchUsers()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : 'Failed to create user')
  } finally {
    creating.value = false
  }
}

onMounted(fetchUsers)
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold">Users</h1>
      <Dialog v-model:open="createDialogOpen">
        <DialogTrigger as-child>
          <Button>
            <Plus class="size-4" />
            Create User
          </Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create User</DialogTitle>
            <DialogDescription>Create a new user on the homeserver.</DialogDescription>
          </DialogHeader>
          <form class="flex flex-col gap-4" @submit.prevent="createUser">
            <div class="flex flex-col gap-2">
              <Label for="new-username">Username</Label>
              <Input id="new-username" v-model="newUsername" placeholder="username" required />
            </div>
            <div class="flex flex-col gap-2">
              <Label for="new-password">Password (optional)</Label>
              <Input id="new-password" v-model="newPassword" type="password" />
            </div>
            <DialogFooter>
              <Button type="submit" :disabled="creating">
                {{ creating ? 'Creating...' : 'Create' }}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>User List</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="loading" class="h-32 w-full" />
        <Table v-else>
          <TableHeader>
            <TableRow>
              <TableHead>User ID</TableHead>
              <TableHead class="w-12 text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableEmpty v-if="users.length === 0" :colspan="2">
              No users found.
            </TableEmpty>
            <TableRow v-for="user in users" :key="user">
              <TableCell class="font-mono text-sm">{{ user }}</TableCell>
              <TableCell class="text-right">
                <UserActionsMenu :user-id="user" @action-complete="fetchUsers" />
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
