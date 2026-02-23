<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { SidebarProvider, SidebarInset, SidebarTrigger } from '@/components/ui/sidebar'
import { Separator } from '@/components/ui/separator'
import { Toaster } from '@/components/ui/sonner'
import AppSidebar from '@/components/AppSidebar.vue'
import ConsolePanel from '@/components/ConsolePanel.vue'
import ServerSelector from '@/components/ServerSelector.vue'

const route = useRoute()
const isConsolePage = computed(() => route.name === 'console')
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <SidebarInset class="flex flex-col h-dvh overflow-hidden">
      <header class="flex h-14 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger class="-ml-1" />
        <Separator orientation="vertical" class="mr-2 !h-4" />
        <ServerSelector />
      </header>
      <main class="flex-1 overflow-auto p-6">
        <RouterView />
      </main>
      <ConsolePanel v-if="!isConsolePage" />
    </SidebarInset>
    <Toaster />
  </SidebarProvider>
</template>
