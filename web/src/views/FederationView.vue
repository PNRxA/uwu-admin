<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
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

const federationStatus = ref('')
const loading = ref(true)

interface PduEntry {
  roomId: string
  eventId: string
  elapsed: string
}

const summary = computed(() => {
  const clean = federationStatus.value.replace(/<[^>]+>/g, '')
  const m = clean.match(/Handling\s+(\d+)\s+incoming\s+pdus/)
  return m?.[1] ? `${m[1]} incoming` : null
})

const pdus = computed<PduEntry[]>(() => {
  const clean = federationStatus.value.replace(/<[^>]+>/g, '')
  return clean
    .split(/\r?\n/)
    .map((line) => {
      const m = line.trim().match(/^(!\S+)\s+(\$\S+):\s*(.+)$/)
      if (!m?.[1] || !m[2] || !m[3]) return null
      return { roomId: m[1], eventId: m[2], elapsed: m[3].trim() }
    })
    .filter((e): e is PduEntry => e !== null)
})

onMounted(async () => {
  try {
    const res = await api.command('federation incoming-federation')
    federationStatus.value = res.response
  } catch {
    federationStatus.value = 'Failed to fetch federation status'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-2xl font-bold">Federation</h1>

    <Card v-if="!loading && summary">
      <CardHeader>
        <CardTitle>Incoming PDUs</CardTitle>
        <CardDescription>{{ summary }} PDUs being handled</CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Room ID</TableHead>
              <TableHead>Event ID</TableHead>
              <TableHead class="w-28 text-right">Elapsed</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableEmpty v-if="pdus.length === 0" :colspan="3">
              No incoming PDUs.
            </TableEmpty>
            <TableRow v-for="pdu in pdus" :key="pdu.eventId">
              <TableCell class="font-mono text-sm max-w-64 truncate">{{ pdu.roomId }}</TableCell>
              <TableCell class="font-mono text-sm max-w-64 truncate">{{ pdu.eventId }}</TableCell>
              <TableCell class="text-right tabular-nums">{{ pdu.elapsed }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>

    <Card v-if="loading">
      <CardHeader>
        <CardTitle>Incoming PDUs</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton class="h-32 w-full" />
      </CardContent>
    </Card>

    <Card v-if="!loading && !summary">
      <CardHeader>
        <CardTitle>Incoming PDUs</CardTitle>
        <CardDescription>No incoming federation activity</CardDescription>
      </CardHeader>
    </Card>
  </div>
</template>
