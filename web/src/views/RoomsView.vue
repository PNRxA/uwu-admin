<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
import { toast } from 'vue-sonner'
import RoomActionsMenu from '@/components/RoomActionsMenu.vue'

interface Room {
  id: string
  members: number
  name: string
}

const roomsResponse = ref('')
const loading = ref(true)

const rooms = computed<Room[]>(() => {
  if (!roomsResponse.value) return []
  return roomsResponse.value
    .split('\n')
    .map((line) => {
      const match = line.match(/^(!\S+)\s+Members:\s*(\d+)\s+Name:\s*(.+)$/)
      if (!match?.[1] || !match[2] || !match[3]) return null
      return { id: match[1], members: parseInt(match[2], 10), name: match[3].trim() }
    })
    .filter((r): r is Room => r !== null)
})

async function fetchRooms() {
  loading.value = true
  try {
    const res = await api.listRooms()
    roomsResponse.value = res.response
  } catch {
    roomsResponse.value = ''
    toast.error('Failed to fetch rooms')
  } finally {
    loading.value = false
  }
}

onMounted(fetchRooms)
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-2xl font-bold">Rooms</h1>

    <Card>
      <CardHeader>
        <CardTitle>Room List</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="loading" class="h-32 w-full" />
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
                <RoomActionsMenu :room-id="room.id" @action-complete="fetchRooms" />
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
