<script setup lang="ts">
import { watch } from 'vue'
import { useIsUwu } from '@/composables/useIsUwu'
import { useConsole, sanitizeHtml } from '@/composables/useConsole'
import CommandAutocomplete from '@/components/CommandAutocomplete.vue'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Send, Trash2, Terminal, ChevronUp, CircleAlert, Maximize2 } from 'lucide-vue-next'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()
const { isUwu } = useIsUwu()

const {
  commandStore,
  commandInput,
  submittedError,
  autocompleteRef,
  scrollToBottom,
  sendCommand,
  formatTime,
} = useConsole('console-panel-bottom', {
  autoScrollEnabled: () => commandStore.panelOpen,
})

watch(() => commandStore.panelOpen, (open) => {
  if (open) {
    commandStore.unreadCount = 0
    scrollToBottom('instant')
  }
})
</script>

<template>
  <Collapsible v-model:open="commandStore.panelOpen" class="shrink-0 border-t bg-card">
    <CollapsibleTrigger as-child>
      <button
        class="flex w-full items-center gap-2 px-4 py-2 text-sm bg-sidebar hover:bg-sidebar-accent transition-colors cursor-pointer"
      >
        <Terminal class="size-4 text-muted-foreground" />
        <span class="font-medium">Console</span>
        <Badge v-if="commandStore.unreadCount > 0" variant="secondary" class="text-xs">
          {{ commandStore.unreadCount }}
        </Badge>
        <Button variant="ghost" size="sm" class="ml-auto h-7 px-2" @click.stop="router.push({ name: 'console', params: { serverId: route.params.serverId } })">
          <Maximize2 class="size-3.5" />
        </Button>
        <Button variant="ghost" size="sm" class="h-7 px-2" @click.stop="commandStore.clear">
          <Trash2 class="size-3.5" />
        </Button>
        <ChevronUp
          class="size-4 text-muted-foreground transition-transform duration-200"
          :class="commandStore.panelOpen ? 'rotate-180' : ''"
        />
      </button>
    </CollapsibleTrigger>

    <CollapsibleContent>
      <div class="flex flex-col border-t" style="height: 384px">
        <ScrollArea class="flex-1 min-h-0 px-4">
          <div class="flex flex-col gap-3 pb-3">
            <div
              v-for="entry in commandStore.history"
              :key="entry.id"
              class="flex flex-col gap-1.5"
            >
              <div class="flex items-center gap-2">
                <Badge v-if="entry.success === null" variant="secondary" class="text-xs animate-pulse">
                  ...
                </Badge>
                <Badge v-else :variant="entry.success ? 'default' : 'destructive'" class="text-xs">
                  {{ entry.success ? 'OK' : 'ERR' }}
                </Badge>
                <code class="text-xs font-medium">!admin {{ entry.command }}</code>
                <span class="ml-auto text-xs text-muted-foreground">{{ formatTime(entry.timestamp) }}</span>
              </div>
              <div data-testid="console-response" class="console-response text-xs rounded-md bg-muted p-2 max-h-40 overflow-auto" v-html="sanitizeHtml(entry.response || 'Waiting for response...')" />
              <Separator />
            </div>
            <div v-if="commandStore.history.length === 0" class="text-center text-muted-foreground py-4 text-sm">
              <div v-if="isUwu" class="text-lg mb-1">(=^-ω-^=)</div>
              {{ isUwu ? $t('console.noCommandsYetUwu') : $t('console.noCommandsYet') }}
            </div>
            <div id="console-panel-bottom" />
          </div>
        </ScrollArea>

        <div class="shrink-0 relative">
          <Alert v-if="submittedError" variant="destructive" class="rounded-none border-x-0 border-b-0">
            <CircleAlert class="size-4" />
            <AlertDescription>{{ submittedError }}</AlertDescription>
          </Alert>
          <form
            class="flex gap-2 px-4 py-2 border-t transition-colors"
            :class="submittedError ? 'bg-destructive/5' : ''"
            @submit.prevent="sendCommand"
          >
            <div class="flex items-center gap-2 flex-1 min-w-0">
              <span class="text-xs text-muted-foreground whitespace-nowrap">!admin</span>
              <div class="flex-1 min-w-0">
                <CommandAutocomplete
                  ref="autocompleteRef"
                  v-model="commandInput"
                  placeholder="server uptime"
                  :disabled="commandStore.loading"
                  @submit="sendCommand"
                />
              </div>
            </div>
            <Button type="submit" size="default" :disabled="commandStore.loading || !commandInput.trim()">
              <Send class="size-4" />
            </Button>
          </form>
        </div>
      </div>
    </CollapsibleContent>
  </Collapsible>
</template>