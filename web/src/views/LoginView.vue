<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'

const router = useRouter()
const auth = useAuthStore()

const username = ref('')
const password = ref('')

async function onSubmit() {
  try {
    await auth.login(username.value, password.value)
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
        <CardDescription>Sign in to your admin panel</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="flex flex-col gap-4" @submit.prevent="onSubmit">
          <Alert v-if="auth.error" variant="destructive">
            <AlertDescription>{{ auth.error }}</AlertDescription>
          </Alert>

          <div class="flex flex-col gap-2">
            <Label for="username">Username</Label>
            <Input
              id="username"
              v-model="username"
              placeholder="admin"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="password">Password</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              required
            />
          </div>

          <Button type="submit" class="w-full" :disabled="auth.loading">
            {{ auth.loading ? 'Signing in...' : 'Sign In' }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
