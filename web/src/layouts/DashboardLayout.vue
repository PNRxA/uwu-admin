<script setup lang="ts">
import { useConnectionStore } from '@/stores/connection'
import { SidebarProvider, SidebarInset, SidebarTrigger } from '@/components/ui/sidebar'
import { Separator } from '@/components/ui/separator'
import { Toaster } from '@/components/ui/sonner'
import AppSidebar from '@/components/AppSidebar.vue'

const connection = useConnectionStore()
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <SidebarInset>
      <header class="flex h-14 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger class="-ml-1" />
        <Separator orientation="vertical" class="mr-2 !h-4" />
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <span>{{ connection.homeserver }}</span>
          <span v-if="connection.userId" class="text-foreground font-medium">{{ connection.userId }}</span>
        </div>
      </header>
      <main class="flex-1 p-6">
        <RouterView />
      </main>
    </SidebarInset>
    <Toaster />
  </SidebarProvider>
</template>
