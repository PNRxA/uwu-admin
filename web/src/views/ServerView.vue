<script setup lang="ts">
import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useConnectionStore } from '@/stores/connection'
import { useCommandStore } from '@/stores/command'
import { stripHtml } from '@/lib/response-parser'
import { queryKeys } from '@/lib/query-keys'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { RefreshCw } from 'lucide-vue-next'

const connection = useConnectionStore()
const commandStore = useCommandStore()
const serverId = computed(() => connection.activeServerId)

const {
  data: uptime,
  isPending: uptimePending,
  isFetching: uptimeFetching,
  refetch: refetchUptime,
} = useQuery({
  queryKey: computed(() => queryKeys.serverUptime(serverId.value!)),
  queryFn: async () => (await commandStore.query('server uptime')).response,
  staleTime: 15_000,
  enabled: computed(() => serverId.value !== null),
})

const {
  data: stats,
  isPending: statsPending,
  isFetching: statsFetching,
  refetch: refetchStats,
} = useQuery({
  queryKey: computed(() => queryKeys.serverStatus(serverId.value!)),
  queryFn: async () => (await commandStore.query('server memory-usage')).response,
  staleTime: 15_000,
  enabled: computed(() => serverId.value !== null),
})

const loading = computed(() => uptimePending.value || statsPending.value)
const isFetching = computed(() => uptimeFetching.value || statsFetching.value)

function refetch() {
  refetchUptime()
  refetchStats()
}

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

const uptimeFormatted = computed(() => formatUptime(uptime.value ?? ''))

const services = computed<ServiceEntry[]>(() => {
  if (!stats.value) return []
  const servicesMatch = stats.value.match(/Services:\s*\n?([\s\S]*?)(?:\n\s*\n|Database:)/)
  if (!servicesMatch?.[1]) return []
  return servicesMatch[1]
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((line) => {
      const clean = stripHtml(line).trim()
      const m = clean.match(/^(.+?):\s*(.+?)(?:\s+\((.+?)\))?$/)
      if (!m?.[1] || !m[2]) return null
      return { name: m[1].trim(), value: m[2].trim(), size: m[3]?.trim() ?? '' }
    })
    .filter((e): e is ServiceEntry => e !== null)
})

const database = computed<DatabaseEntry[]>(() => {
  if (!stats.value) return []
  const dbMatch = stats.value.match(/Database:\s*\n?([\s\S]*)$/)
  if (!dbMatch?.[1]) return []
  return dbMatch[1]
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((line) => {
      const clean = stripHtml(line).trim()
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
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-2">
      <h1 class="text-2xl font-bold">{{ $t('server.title') }}</h1>
      <Button variant="ghost" size="icon-sm" :disabled="isFetching || !serverId" @click="refetch()">
        <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
        <span class="sr-only">{{ $t('common.refresh') }}</span>
      </Button>
    </div>

    <div v-if="!serverId" class="text-muted-foreground text-sm">
      {{ $t('common.noServerSelected') }}
    </div>

    <template v-else>
      <div class="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{{ $t('server.uptime') }}</CardTitle>
            <CardDescription>{{ $t('server.uptimeDescription') }}</CardDescription>
          </CardHeader>
          <CardContent>
            <Skeleton v-if="loading" class="h-12 w-full" />
            <div v-else class="flex flex-wrap items-baseline gap-3">
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
            <CardTitle>{{ $t('server.databaseMemory') }}</CardTitle>
            <CardDescription>{{ $t('server.databaseMemoryDescription') }}</CardDescription>
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
          <CardTitle>{{ $t('server.services') }}</CardTitle>
          <CardDescription>{{ $t('server.servicesDescription') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{{ $t('server.serviceName') }}</TableHead>
                <TableHead class="w-32 text-right">{{ $t('server.count') }}</TableHead>
                <TableHead class="w-32 text-right">{{ $t('server.size') }}</TableHead>
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
          <CardTitle>{{ $t('server.database') }}</CardTitle>
          <CardDescription>{{ $t('server.databaseDescription') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{{ $t('server.component') }}</TableHead>
                <TableHead class="w-40 text-right">{{ $t('server.memory') }}</TableHead>
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
    </template>
  </div>
</template>
