<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useIsUwu } from '@/composables/useIsUwu'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const { isUwu } = useIsUwu()

async function onSubmit() {
  if (password.value !== confirmPassword.value) {
    auth.error = t('setup.passwordMismatch')
    return
  }
  try {
    await auth.register(username.value, password.value)
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
        <CardTitle class="text-2xl">{{ isUwu ? $t('common.appName') : $t('common.appNameProfessional') }}</CardTitle>
        <CardDescription>{{ $t('setup.description') }}</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="flex flex-col gap-4" @submit.prevent="onSubmit">
          <Alert v-if="auth.error" variant="destructive">
            <AlertDescription>{{ auth.error }}</AlertDescription>
          </Alert>

          <div class="flex flex-col gap-2">
            <Label for="username">{{ $t('setup.username') }}</Label>
            <Input
              id="username"
              v-model="username"
              placeholder="admin"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="password">{{ $t('setup.password') }}</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              required
            />
          </div>

          <div class="flex flex-col gap-2">
            <Label for="confirm-password">{{ $t('setup.confirmPassword') }}</Label>
            <Input
              id="confirm-password"
              v-model="confirmPassword"
              type="password"
              required
            />
          </div>

          <Button type="submit" class="w-full" :disabled="auth.loading">
            {{ auth.loading ? $t('setup.creatingAccount') : $t('setup.createAccount') }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
