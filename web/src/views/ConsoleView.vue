<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useCommandStore } from '@/stores/command'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Send, Trash2 } from 'lucide-vue-next'

const commandStore = useCommandStore()
const commandInput = ref('')
const scrollRef = ref<InstanceType<typeof ScrollArea> | null>(null)

async function sendCommand() {
  const cmd = commandInput.value.trim()
  if (!cmd) return
  commandInput.value = ''
  await commandStore.execute(cmd)
  await nextTick()
  // scroll to bottom
  const el = document.getElementById('console-bottom')
  el?.scrollIntoView({ behavior: 'smooth' })
}

function formatTime(date: Date) {
  return date.toLocaleTimeString()
}
</script>

<template>
  <div class="flex flex-col gap-6 h-[calc(100vh-8rem)]">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold">Console</h1>
      <Button variant="ghost" size="sm" @click="commandStore.clear">
        <Trash2 class="size-4" />
        Clear
      </Button>
    </div>

    <Card class="flex-1 flex flex-col min-h-0">
      <CardHeader class="pb-3">
        <CardTitle class="text-sm text-muted-foreground">
          Commands are sent as <code class="rounded bg-muted px-1">!admin &lt;command&gt;</code>
        </CardTitle>
      </CardHeader>
      <CardContent class="flex-1 min-h-0 flex flex-col">
        <ScrollArea class="flex-1 pr-4">
          <div class="flex flex-col gap-4">
            <div
              v-for="entry in commandStore.history"
              :key="entry.id"
              class="flex flex-col gap-2"
            >
              <div class="flex items-center gap-2">
                <Badge :variant="entry.success ? 'default' : 'destructive'" class="text-xs">
                  {{ entry.success ? 'OK' : 'ERR' }}
                </Badge>
                <code class="text-sm font-medium">!admin {{ entry.command }}</code>
                <span class="ml-auto text-xs text-muted-foreground">{{ formatTime(entry.timestamp) }}</span>
              </div>
              <pre class="whitespace-pre-wrap text-sm rounded-md bg-muted p-3 max-h-64 overflow-auto">{{ entry.response || 'Waiting for response...' }}</pre>
              <Separator />
            </div>
            <div v-if="commandStore.history.length === 0" class="text-center text-muted-foreground py-8">
              No commands sent yet. Type a command below to get started.
            </div>
            <div id="console-bottom" />
          </div>
        </ScrollArea>

        <form class="flex gap-2 pt-4 border-t mt-4" @submit.prevent="sendCommand">
          <div class="flex items-center gap-2 flex-1">
            <span class="text-sm text-muted-foreground whitespace-nowrap">!admin</span>
            <Input
              v-model="commandInput"
              placeholder="server uptime"
              :disabled="commandStore.loading"
              class="flex-1"
            />
          </div>
          <Button type="submit" :disabled="commandStore.loading || !commandInput.trim()">
            <Send class="size-4" />
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
