<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConnectionStore } from '@/stores/connection'
import { api } from '@/lib/api'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

const connection = useConnectionStore()

const uptime = ref('')
const stats = ref('')
const loading = ref(true)

onMounted(async () => {
  try {
    const [uptimeRes, statsRes] = await Promise.all([
      api.serverUptime(),
      api.serverStatus(),
    ])
    uptime.value = uptimeRes.response
    stats.value = statsRes.response
  } catch {
    // silently fail, show empty state
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-2xl font-bold">Overview</h1>

    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card>
        <CardHeader>
          <CardTitle>Connection</CardTitle>
          <CardDescription>Current homeserver connection</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-1 text-sm">
          <div><span class="text-muted-foreground">Homeserver:</span> {{ connection.homeserver }}</div>
          <div><span class="text-muted-foreground">User ID:</span> {{ connection.userId }}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Uptime</CardTitle>
          <CardDescription>Server uptime</CardDescription>
        </CardHeader>
        <CardContent>
          <Skeleton v-if="loading" class="h-16 w-full" />
          <pre v-else class="whitespace-pre-wrap text-sm">{{ uptime }}</pre>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Server Stats</CardTitle>
          <CardDescription>Current server statistics</CardDescription>
        </CardHeader>
        <CardContent>
          <Skeleton v-if="loading" class="h-16 w-full" />
          <pre v-else class="whitespace-pre-wrap text-sm">{{ stats }}</pre>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
