<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import AppLogo from '@/components/AppLogo.vue'

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
        <CardTitle class="flex justify-center">
          <AppLogo size="lg" />
        </CardTitle>
        <CardDescription>{{ $t('login.description') }}</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="flex flex-col gap-4" @submit.prevent="onSubmit">
          <Alert v-if="auth.error" variant="destructive">
            <AlertDescription>{{ auth.error }}</AlertDescription>
          </Alert>

          <div class="flex flex-col gap-2">
            <Label for="username">{{ $t('login.username') }}</Label>
            <Input
              id="username"
              v-model="username"
              placeholder="admin"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="password">{{ $t('login.password') }}</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              required
            />
          </div>

          <Button type="submit" class="w-full" :disabled="auth.loading">
            {{ auth.loading ? $t('login.signingIn') : $t('login.signIn') }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
