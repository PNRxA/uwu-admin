<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

const roomsResponse = ref('')
const loading = ref(true)
const roomDetailOpen = ref(false)
const roomDetail = ref('')
const roomDetailLoading = ref(false)
const selectedRoom = ref('')

async function fetchRooms() {
  loading.value = true
  try {
    const res = await api.listRooms()
    roomsResponse.value = res.response
  } catch {
    roomsResponse.value = 'Failed to fetch rooms'
  } finally {
    loading.value = false
  }
}

async function showRoomInfo(roomId: string) {
  selectedRoom.value = roomId
  roomDetailOpen.value = true
  roomDetailLoading.value = true
  try {
    const res = await api.roomInfo(roomId)
    roomDetail.value = res.response
  } catch {
    roomDetail.value = 'Failed to fetch room info'
  } finally {
    roomDetailLoading.value = false
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
        <pre v-else class="whitespace-pre-wrap text-sm max-h-[60vh] overflow-auto">{{ roomsResponse }}</pre>
      </CardContent>
    </Card>

    <Dialog v-model:open="roomDetailOpen">
      <DialogContent class="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Room Info: {{ selectedRoom }}</DialogTitle>
        </DialogHeader>
        <Skeleton v-if="roomDetailLoading" class="h-32 w-full" />
        <pre v-else class="whitespace-pre-wrap text-sm max-h-[60vh] overflow-auto">{{ roomDetail }}</pre>
      </DialogContent>
    </Dialog>
  </div>
</template>
