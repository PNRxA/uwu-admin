<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

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
          <Skeleton v-if="loading" class="h-16 w-full" />
          <pre v-else class="whitespace-pre-wrap text-sm">{{ uptime }}</pre>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Statistics</CardTitle>
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
