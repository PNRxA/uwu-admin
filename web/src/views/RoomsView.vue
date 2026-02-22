<script setup lang="ts">
import { computed } from 'vue'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'
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

interface Room {
  id: string
  members: number
  name: string
}

const { data: roomsResponse, isPending, isFetching, refetch } = useQuery({
  queryKey: queryKeys.rooms,
  queryFn: async () => (await api.listRooms()).response,
  staleTime: 30_000,
})

const rooms = computed<Room[]>(() => {
  if (!roomsResponse.value) return []
  return roomsResponse.value
    .replace(/<[^>]+>/g, '')
    .split('\n')
    .map((line) => {
      const match = line.match(/^(!\S+)\s+Members:\s*(\d+)\s+Name:\s*(.+)$/)
      if (!match?.[1] || !match[2] || !match[3]) return null
      return { id: match[1], members: parseInt(match[2], 10), name: match[3].trim() }
    })
    .filter((r): r is Room => r !== null)
})

function onActionComplete() {
  queryClient.invalidateQueries({ queryKey: queryKeys.rooms })
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-2">
      <h1 class="text-2xl font-bold">Rooms</h1>
      <Button variant="ghost" size="icon-sm" :disabled="isFetching" @click="refetch()">
        <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
        <span class="sr-only">Refresh</span>
      </Button>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>Room List</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="isPending" class="h-32 w-full" />
        <Table v-else>
          <TableHeader>
            <TableRow>
              <TableHead>Room ID</TableHead>
              <TableHead class="w-24 text-right">Members</TableHead>
              <TableHead>Name</TableHead>
              <TableHead class="w-12 text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableEmpty v-if="rooms.length === 0" :colspan="4">
              No rooms found.
            </TableEmpty>
            <TableRow v-for="room in rooms" :key="room.id">
              <TableCell class="font-mono text-sm max-w-64 truncate">{{ room.id }}</TableCell>
              <TableCell class="text-right tabular-nums">{{ room.members.toLocaleString() }}</TableCell>
              <TableCell>{{ room.name }}</TableCell>
              <TableCell class="text-right">
                <RoomActionsMenu :room-id="room.id" @action-complete="onActionComplete" />
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
