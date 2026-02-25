<script setup lang="ts">
import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useConnectionStore } from '@/stores/connection'
import { useCommandStore } from '@/stores/command'
import { stripHtml } from '@/lib/response-parser'
import { queryKeys } from '@/lib/query-keys'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
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
import { RefreshCw } from 'lucide-vue-next'
import CopyableCell from '@/components/CopyableCell.vue'

const connection = useConnectionStore()
const commandStore = useCommandStore()
const serverId = computed(() => connection.activeServerId)

interface PduEntry {
  roomId: string
  eventId: string
  elapsed: string
}

const { data: federationStatus, isPending, isFetching, refetch } = useQuery({
  queryKey: computed(() => queryKeys.federation(serverId.value!)),
  queryFn: async () => (await commandStore.query('federation incoming-federation')).response,
  staleTime: 10_000,
  enabled: computed(() => serverId.value !== null),
})

const summary = computed(() => {
  if (!federationStatus.value) return null
  const clean = stripHtml(federationStatus.value)
  const m = clean.match(/Handling\s+(\d+)\s+incoming\s+pdus/)
  return m?.[1] ?? null
})

const pdus = computed<PduEntry[]>(() => {
  if (!federationStatus.value) return []
  const clean = stripHtml(federationStatus.value)
  return clean
    .split(/\r?\n/)
    .map((line) => {
      const m = line.trim().match(/^(!\S+)\s+(\$\S+):\s*(.+)$/)
      if (!m?.[1] || !m[2] || !m[3]) return null
      return { roomId: m[1], eventId: m[2], elapsed: m[3].trim() }
    })
    .filter((e): e is PduEntry => e !== null)
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-2">
      <h1 class="text-2xl font-bold">{{ $t('federation.title') }}</h1>
      <Button variant="ghost" size="icon-sm" :disabled="isFetching || !serverId" @click="refetch()">
        <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
        <span class="sr-only">{{ $t('common.refresh') }}</span>
      </Button>
    </div>

    <div v-if="!serverId" class="text-muted-foreground text-sm">
      {{ $t('common.noServerSelected') }}
    </div>

    <template v-else>
      <Card v-if="!isPending && summary">
        <CardHeader>
          <CardTitle>{{ $t('federation.incomingPDUs') }}</CardTitle>
          <CardDescription>{{ $t('federation.pdusBeingHandled', { count: summary }) }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{{ $t('federation.roomId') }}</TableHead>
                <TableHead>{{ $t('federation.eventId') }}</TableHead>
                <TableHead class="w-28 text-right">{{ $t('federation.elapsed') }}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableEmpty v-if="pdus.length === 0" :colspan="3">
                {{ $t('federation.noIncomingPDUs') }}
              </TableEmpty>
              <TableRow v-for="pdu in pdus" :key="pdu.eventId">
                <TableCell class="font-mono text-sm max-w-64"><CopyableCell :value="pdu.roomId" /></TableCell>
                <TableCell class="font-mono text-sm max-w-64"><CopyableCell :value="pdu.eventId" /></TableCell>
                <TableCell class="tabular-nums"><CopyableCell :value="pdu.elapsed" align="right" /></TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Card v-if="isPending">
        <CardHeader>
          <CardTitle>{{ $t('federation.incomingPDUs') }}</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton class="h-32 w-full" />
        </CardContent>
      </Card>

      <Card v-if="!isPending && !summary">
        <CardHeader>
          <CardTitle>{{ $t('federation.incomingPDUs') }}</CardTitle>
          <CardDescription>{{ $t('federation.noFederationActivity') }}</CardDescription>
        </CardHeader>
      </Card>
    </template>
  </div>
</template>
