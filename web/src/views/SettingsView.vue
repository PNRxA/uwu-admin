<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { toast } from 'vue-sonner'
import { useSettingsStore } from '@/stores/settings'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Skeleton } from '@/components/ui/skeleton'

const { t } = useI18n()
const settingsStore = useSettingsStore()

onMounted(() => {
  settingsStore.fetchSettings()
})

async function onRedactToggle(checked: boolean) {
  try {
    await settingsStore.updateSetting('redact_messages', String(checked))
    toast.success(t('settings.updateSuccess'))
  } catch {
    toast.error(t('settings.updateFailed'))
  }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div>
      <h1 class="text-2xl font-bold">{{ t('settings.title') }}</h1>
      <p class="text-sm text-muted-foreground">{{ t('settings.subtitle') }}</p>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>{{ t('settings.adminRoom') }}</CardTitle>
        <CardDescription>{{ t('settings.adminRoomDescription') }}</CardDescription>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="settingsStore.loading" class="h-6 w-full" />
        <div v-else class="flex items-center justify-between gap-4">
          <Label for="redact-messages" class="flex flex-col items-start gap-1">
            <span>{{ t('settings.redactMessages') }}</span>
            <span class="text-sm font-normal text-muted-foreground">
              {{ t('settings.redactMessagesDescription') }}
            </span>
          </Label>
          <Switch
            id="redact-messages"
            :model-value="settingsStore.settings.redact_messages !== 'false'"
            @update:model-value="onRedactToggle"
          />
        </div>
      </CardContent>
    </Card>
  </div>
</template>
