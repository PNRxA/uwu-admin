<script setup lang="ts">
import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useConnectionStore } from '@/stores/connection'
import { api } from '@/lib/api'
import { stripHtml } from '@/lib/response-parser'
import { queryKeys } from '@/lib/query-keys'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { RefreshCw } from 'lucide-vue-next'

const connection = useConnectionStore()

const serverId = computed(() => connection.activeServerId)

const {
  data: uptime,
  isPending: uptimePending,
  isFetching: uptimeFetching,
  refetch: refetchUptime,
} = useQuery({
  queryKey: computed(() => queryKeys.serverUptime(serverId.value!)),
  queryFn: async () => (await api.command(serverId.value!, 'server uptime')).response,
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
  queryFn: async () => (await api.command(serverId.value!, 'server memory-usage')).response,
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

const uptimeFormatted = computed(() => formatUptime(uptime.value ?? ''))

const totalDbMemory = computed(() => {
  if (!stats.value) return ''
  const clean = stripHtml(stats.value)
  const dbMatch = clean.match(/Database:\s*\n?([\s\S]*)$/)
  if (!dbMatch?.[1]) return ''
  let total = 0
  for (const line of dbMatch[1].split(/\r?\n/)) {
    const m = line.match(/([\d.]+)\s*MiB/)
    if (m?.[1]) total += parseFloat(m[1])
  }
  if (total === 0) return ''
  return total > 1024 ? `${(total / 1024).toFixed(2)} GiB` : `${total.toFixed(2)} MiB`
})

const serviceCount = computed(() => {
  if (!stats.value) return 0
  const clean = stripHtml(stats.value)
  const servicesMatch = clean.match(/Services:\s*\n?([\s\S]*?)(?:\n\s*\n|Database:)/)
  if (!servicesMatch?.[1]) return 0
  return servicesMatch[1].split(/\r?\n/).filter((l) => l.trim()).length
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-2">
      <h1 class="text-2xl font-bold">{{ $t('overview.title') }}</h1>
      <Button variant="ghost" size="icon-sm" :disabled="isFetching || !serverId" @click="refetch()">
        <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
        <span class="sr-only">{{ $t('common.refresh') }}</span>
      </Button>
    </div>

    <div v-if="!serverId" class="text-muted-foreground text-sm">
      {{ $t('common.noServerSelected') }}
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card class="md:col-span-2">
        <CardHeader>
          <CardTitle>{{ $t('overview.connection') }}</CardTitle>
          <CardDescription>{{ $t('overview.connectionDescription') }}</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-1 text-sm">
          <div><span class="text-muted-foreground">{{ $t('overview.homeserver') }}</span> {{ connection.homeserver }}</div>
          <div><span class="text-muted-foreground">{{ $t('overview.userId') }}</span> {{ connection.userId }}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{{ $t('overview.uptime') }}</CardTitle>
          <CardDescription>{{ $t('overview.serverUptime') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Skeleton v-if="loading" class="h-10 w-full" />
          <div v-else class="flex flex-wrap items-baseline gap-3">
            <template v-for="(part, i) in uptimeFormatted.parts" :key="i">
              <div class="flex items-baseline gap-1">
                <span class="text-2xl font-bold tabular-nums">{{ part.value }}</span>
                <span class="text-xs text-muted-foreground">{{ part.unit }}</span>
              </div>
            </template>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{{ $t('overview.memory') }}</CardTitle>
          <CardDescription>{{ $t('overview.databaseUsage') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Skeleton v-if="loading" class="h-10 w-full" />
          <template v-else>
            <div v-if="totalDbMemory" class="flex items-baseline gap-1">
              <span class="text-2xl font-bold tabular-nums">{{ totalDbMemory.split(' ')[0] }}</span>
              <span class="text-xs text-muted-foreground">{{ totalDbMemory.split(' ')[1] }}</span>
            </div>
            <span v-else class="text-sm text-muted-foreground">{{ $t('common.noData') }}</span>
            <div v-if="serviceCount" class="mt-1 text-xs text-muted-foreground">
              {{ $t('overview.activeServices', { count: serviceCount }) }}
            </div>
          </template>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
