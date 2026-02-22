<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
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
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { toast } from 'vue-sonner'
import { Plus } from 'lucide-vue-next'

const usersResponse = ref('')
const loading = ref(true)
const createDialogOpen = ref(false)
const newUsername = ref('')
const newPassword = ref('')
const creating = ref(false)

async function fetchUsers() {
  loading.value = true
  try {
    const res = await api.listUsers()
    usersResponse.value = res.response
  } catch {
    usersResponse.value = 'Failed to fetch users'
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

async function deactivateUser(userId: string) {
  try {
    await api.command(`users deactivate ${userId}`)
    toast.success('User deactivated')
    await fetchUsers()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : 'Failed to deactivate user')
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
        <pre v-else class="whitespace-pre-wrap text-sm max-h-[60vh] overflow-auto">{{ usersResponse }}</pre>
      </CardContent>
    </Card>
  </div>
</template>
