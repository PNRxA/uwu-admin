<script setup lang="ts">
import { useConsole, sanitizeHtml } from '@/composables/useConsole'
import CommandAutocomplete from '@/components/CommandAutocomplete.vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Send, Trash2, CircleAlert } from 'lucide-vue-next'

const {
  commandStore,
  commandInput,
  submittedError,
  autocompleteRef,
  sendCommand,
  formatTime,
} = useConsole('console-bottom')
</script>

<template>
  <div class="flex flex-col gap-6 h-[calc(100vh-8rem)]">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold">{{ $t('console.title') }}</h1>
      <Button variant="ghost" size="sm" @click="commandStore.clear">
        <Trash2 class="size-4" />
        {{ $t('console.clear') }}
      </Button>
    </div>

    <Card class="flex-1 flex flex-col min-h-0">
      <CardHeader class="pb-3">
        <CardTitle class="text-sm text-muted-foreground">
          {{ $t('console.commandHint') }} <code class="rounded bg-muted px-1">!admin &lt;command&gt;</code>
        </CardTitle>
      </CardHeader>
      <CardContent class="flex-1 min-h-0 flex flex-col">
        <ScrollArea class="flex-1 min-h-0 pr-4">
          <div class="flex flex-col gap-4">
            <div
              v-for="entry in commandStore.history"
              :key="entry.id"
              class="flex flex-col gap-2"
            >
              <div class="flex items-center gap-2">
                <Badge v-if="entry.success === null" variant="secondary" class="text-xs animate-pulse">
                  ...
                </Badge>
                <Badge v-else :variant="entry.success ? 'default' : 'destructive'" class="text-xs">
                  {{ entry.success ? $t('console.statusOk') : $t('console.statusError') }}
                </Badge>
                <code class="text-sm font-medium">!admin {{ entry.command }}</code>
                <span class="ml-auto text-xs text-muted-foreground">{{ formatTime(entry.timestamp) }}</span>
              </div>
              <div class="console-response text-sm rounded-md bg-muted p-3 max-h-64 overflow-auto" v-html="sanitizeHtml(entry.response || $t('console.waitingForResponse'))" />
              <Separator />
            </div>
            <div v-if="commandStore.history.length === 0" class="text-center text-muted-foreground py-8">
              {{ $t('console.noCommandsYet') }}
            </div>
            <div id="console-bottom" />
          </div>
        </ScrollArea>

        <div class="mt-4">
          <Alert v-if="submittedError" variant="destructive" class="mb-3">
            <CircleAlert class="size-4" />
            <AlertDescription>{{ submittedError }}</AlertDescription>
          </Alert>
          <form
            class="flex gap-2 pt-4 border-t transition-colors rounded-md"
            :class="submittedError ? 'bg-destructive/5' : ''"
            @submit.prevent="sendCommand"
          >
            <div class="flex items-center gap-2 flex-1">
              <span class="text-sm text-muted-foreground whitespace-nowrap">!admin</span>
              <CommandAutocomplete
                ref="autocompleteRef"
                v-model="commandInput"
                placeholder="server uptime"
                :disabled="commandStore.loading"
                @submit="sendCommand"
              />
            </div>
            <Button type="submit" :disabled="commandStore.loading || !commandInput.trim()">
              <Send class="size-4" />
            </Button>
          </form>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
