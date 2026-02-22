<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

const uptime = ref('')
const stats = ref('')
const loading = ref(true)

function formatUptime(raw: string) {
  const match = raw.match(/([\d.]+)\s*(seconds?|minutes?|hours?|days?)/)
  if (!match?.[1] || !match[2]) return { parts: [{ value: raw.replace('.', ''), unit: '' }] }

  const value = parseFloat(match[1])
  const unit = match[2].toLowerCase()

  let totalSeconds = value
  if (unit.startsWith('minute')) totalSeconds = value * 60
  else if (unit.startsWith('hour')) totalSeconds = value * 3600
  else if (unit.startsWith('day')) totalSeconds = value * 86400

  const days = Math.floor(totalSeconds / 86400)
  const hours = Math.floor((totalSeconds % 86400) / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = Math.floor(totalSeconds % 60)

  const parts: { value: number; unit: string }[] = []
  if (days > 0) parts.push({ value: days, unit: days === 1 ? 'day' : 'days' })
  if (hours > 0) parts.push({ value: hours, unit: hours === 1 ? 'hr' : 'hrs' })
  if (minutes > 0) parts.push({ value: minutes, unit: 'min' })
  if (seconds > 0 && days === 0) parts.push({ value: seconds, unit: 'sec' })
  if (parts.length === 0) parts.push({ value: 0, unit: 'sec' })

  return { parts }
}

interface ServiceEntry {
  name: string
  value: string
  size: string
}

interface DatabaseEntry {
  name: string
  size: string
}

const uptimeFormatted = computed(() => formatUptime(uptime.value))

const services = computed<ServiceEntry[]>(() => {
  const servicesMatch = stats.value.match(/Services:\s*\n?([\s\S]*?)(?:\n\s*\n|Database:)/)
  if (!servicesMatch?.[1]) return []
  return servicesMatch[1]
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((line) => {
      const clean = line.replace(/<[^>]+>/g, '').trim()
      const m = clean.match(/^(.+?):\s*(.+?)(?:\s+\((.+?)\))?$/)
      if (!m?.[1] || !m[2]) return null
      return { name: m[1].trim(), value: m[2].trim(), size: m[3]?.trim() ?? '' }
    })
    .filter((e): e is ServiceEntry => e !== null)
})

const database = computed<DatabaseEntry[]>(() => {
  const dbMatch = stats.value.match(/Database:\s*\n?([\s\S]*)$/)
  if (!dbMatch?.[1]) return []
  return dbMatch[1]
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((line) => {
      const clean = line.replace(/<[^>]+>/g, '').trim()
      const m = clean.match(/^(.+?):\s*(.+)$/)
      if (!m?.[1] || !m[2]) return null
      return { name: m[1].trim(), size: m[2].trim() }
    })
    .filter((e): e is DatabaseEntry => e !== null)
})

const totalDbMemory = computed(() => {
  let total = 0
  for (const entry of database.value) {
    const m = entry.size.match(/([\d.]+)\s*MiB/)
    if (m?.[1]) total += parseFloat(m[1])
  }
  return total > 1024 ? `${(total / 1024).toFixed(2)} GiB` : `${total.toFixed(2)} MiB`
})

onMounted(async () => {
  try {
    const [uptimeRes, statsRes] = await Promise.all([
      api.serverUptime(),
      api.serverStatus(),
    ])
    uptime.value = uptimeRes.response
    stats.value = statsRes.response
  } catch {
    // silently fail
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-2xl font-bold">Server</h1>

    <div class="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Uptime</CardTitle>
          <CardDescription>How long the server has been running</CardDescription>
        </CardHeader>
        <CardContent>
          <Skeleton v-if="loading" class="h-12 w-full" />
          <div v-else class="flex items-baseline gap-3">
            <template v-for="(part, i) in uptimeFormatted.parts" :key="i">
              <div class="flex items-baseline gap-1">
                <span class="text-3xl font-bold tabular-nums">{{ part.value }}</span>
                <span class="text-sm text-muted-foreground">{{ part.unit }}</span>
              </div>
            </template>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Database Memory</CardTitle>
          <CardDescription>Total memory used by database</CardDescription>
        </CardHeader>
        <CardContent>
          <Skeleton v-if="loading" class="h-12 w-full" />
          <div v-else class="flex items-baseline gap-1">
            <span class="text-3xl font-bold tabular-nums">{{ totalDbMemory.split(' ')[0] }}</span>
            <span class="text-sm text-muted-foreground">{{ totalDbMemory.split(' ')[1] }}</span>
          </div>
        </CardContent>
      </Card>
    </div>

    <Card v-if="!loading && services.length > 0">
      <CardHeader>
        <CardTitle>Services</CardTitle>
        <CardDescription>Active services and caches</CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead class="w-32 text-right">Count</TableHead>
              <TableHead class="w-32 text-right">Size</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="service in services" :key="service.name">
              <TableCell class="font-mono text-sm">{{ service.name }}</TableCell>
              <TableCell class="text-right tabular-nums">{{ service.value }}</TableCell>
              <TableCell class="text-right tabular-nums text-muted-foreground">{{ service.size || '\u2014' }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>

    <Card v-if="!loading && database.length > 0">
      <CardHeader>
        <CardTitle>Database</CardTitle>
        <CardDescription>Memory allocation by component</CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Component</TableHead>
              <TableHead class="w-40 text-right">Memory</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="entry in database" :key="entry.name">
              <TableCell class="font-mono text-sm">{{ entry.name }}</TableCell>
              <TableCell class="text-right tabular-nums">{{ entry.size }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
