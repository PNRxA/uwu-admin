<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useConnectionStore } from '@/stores/connection'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'

const router = useRouter()
const connection = useConnectionStore()

const homeserver = ref('')
const username = ref('')
const password = ref('')
const roomId = ref('')

async function onSubmit() {
  try {
    await connection.connect({
      homeserver: homeserver.value,
      username: username.value,
      password: password.value,
      room_id: roomId.value,
    })
    router.push({ name: 'overview' })
  } catch {
    // error is set in store
  }
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-md">
      <CardHeader class="text-center">
        <CardTitle class="text-2xl">uwu-admin</CardTitle>
        <CardDescription>Connect to your Continuwuity homeserver</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="flex flex-col gap-4" @submit.prevent="onSubmit">
          <Alert v-if="connection.error" variant="destructive">
            <AlertDescription>{{ connection.error }}</AlertDescription>
          </Alert>

          <div class="flex flex-col gap-2">
            <Label for="homeserver">Homeserver URL</Label>
            <Input
              id="homeserver"
              v-model="homeserver"
              placeholder="https://matrix.example.com"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="username">Bot Username</Label>
            <Input
              id="username"
              v-model="username"
              placeholder="admin-bot"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="password">Bot Password</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="room-id">Admin Room ID</Label>
            <Input
              id="room-id"
              v-model="roomId"
              placeholder="!roomid:example.com"
              required
            />
          </div>

          <Button type="submit" class="w-full" :disabled="connection.loading">
            {{ connection.loading ? 'Connecting...' : 'Connect' }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
