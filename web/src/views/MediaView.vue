<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import { queryKeys } from '@/lib/query-keys'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { RefreshCw } from 'lucide-vue-next'

const { data: mediaStatus, isPending, isFetching, refetch } = useQuery({
  queryKey: queryKeys.mediaHelp,
  queryFn: async () => (await api.command('media --help')).response,
  staleTime: 300_000,
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-2">
      <h1 class="text-2xl font-bold">Media</h1>
      <Button variant="ghost" size="icon-sm" :disabled="isFetching" @click="refetch()">
        <RefreshCw class="size-4" :class="{ 'animate-spin': isFetching }" />
        <span class="sr-only">Refresh</span>
      </Button>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>Media Management</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="isPending" class="h-32 w-full" />
        <pre v-else class="whitespace-pre-wrap text-sm max-h-[60vh] overflow-auto">{{ mediaStatus }}</pre>
      </CardContent>
    </Card>
  </div>
</template>
