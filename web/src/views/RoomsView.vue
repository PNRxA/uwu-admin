<script setup lang="ts">
import { computed } from 'vue'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useConnectionStore } from '@/stores/connection'
import { useCommandStore } from '@/stores/command'
import { stripHtml } from '@/lib/response-parser'
import { queryKeys } from '@/lib/query-keys'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
import RoomActionsMenu from '@/components/RoomActionsMenu.vue'

const queryClient = useQueryClient()
const connection = useConnectionStore()
const commandStore = useCommandStore()
const serverId = computed(() => connection.activeServerId)

interface Room {
  id: string
  members: number
  name: string
}

const { data: roomsResponse, isPending, isFetching, refetch } = useQuery({
  queryKey: computed(() => queryKeys.rooms(serverId.value!)),
  queryFn: () => commandStore.query('rooms list-rooms'),
  staleTime: 30_000,
  enabled: computed(() => serverId.value !== null),
})

const rooms = computed<Room[]>(() => {
  if (!roomsResponse.value) return []
  const { parsed, response } = roomsResponse.value
  if (parsed.type === 'table') {
    const membersIdx = parsed.columns.indexOf('Members')
    const nameIdx = parsed.columns.indexOf('Name')
    return parsed.rows
      .map((row) => {
        const id = row[0]
        const members = membersIdx >= 0 ? parseInt(row[membersIdx] ?? '0', 10) : 0
        const name = nameIdx >= 0 ? (row[nameIdx] ?? '') : ''
        return id ? { id, members, name } : null
      })
      .filter((r): r is Room => r !== null)
  }
  // Fallback to regex extraction
  return stripHtml(response)
    .split('\n')
    .map((line) => {
      const match = line.match(/^(!\S+)\s+Members:\s*(\d+)\s+Name:\s*(.+)$/)
      if (!match?.[1] || !match[2] || !match[3]) return null
      return { id: match[1], members: parseInt(match[2], 10), name: match[3].trim() }
    })
    .filter((r): r is Room => r !== null)
})

function onActionComplete() {
  if (serverId.value !== null) {
    queryClient.invalidateQueries({ queryKey: queryKeys.rooms(serverId.value) })
  }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-2">
      <h1 class="text-2xl font-bold">{{ $t('rooms.title') }}</h1>
      <Button variant="ghost" size="icon-sm" :disabled="isFetching || !serverId" @click="refetch()">
        <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
        <span class="sr-only">{{ $t('common.refresh') }}</span>
      </Button>
    </div>

    <div v-if="!serverId" class="text-muted-foreground text-sm">
      {{ $t('common.noServerSelected') }}
    </div>

    <Card v-else>
      <CardHeader>
        <CardTitle>{{ $t('rooms.roomList') }}</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="isPending" class="h-32 w-full" />
        <Table v-else>
          <TableHeader>
            <TableRow>
              <TableHead class="w-12">{{ $t('common.actions') }}</TableHead>
              <TableHead>{{ $t('rooms.roomId') }}</TableHead>
              <TableHead class="w-24 text-right">{{ $t('rooms.members') }}</TableHead>
              <TableHead>{{ $t('rooms.name') }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableEmpty v-if="rooms.length === 0" :colspan="4">
              {{ $t('rooms.noRoomsFound') }}
            </TableEmpty>
            <TableRow v-for="room in rooms" :key="room.id">
              <TableCell>
                <RoomActionsMenu :room-id="room.id" @action-complete="onActionComplete" />
              </TableCell>
              <TableCell class="font-mono text-sm max-w-64 truncate">{{ room.id }}</TableCell>
              <TableCell class="text-right tabular-nums">{{ room.members.toLocaleString() }}</TableCell>
              <TableCell>{{ room.name }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
