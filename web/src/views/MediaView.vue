<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

const mediaStatus = ref('')
const loading = ref(true)

onMounted(async () => {
  try {
    const res = await api.command('media --help')
    mediaStatus.value = res.response
  } catch {
    mediaStatus.value = 'Failed to fetch media status'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-2xl font-bold">Media</h1>

    <Card>
      <CardHeader>
        <CardTitle>Media Management</CardTitle>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="loading" class="h-32 w-full" />
        <pre v-else class="whitespace-pre-wrap text-sm max-h-[60vh] overflow-auto">{{ mediaStatus }}</pre>
      </CardContent>
    </Card>
  </div>
</template>
