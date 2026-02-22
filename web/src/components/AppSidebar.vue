<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useConnectionStore } from '@/stores/connection'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'
import { Button } from '@/components/ui/button'
import {
  LayoutDashboard,
  Users,
  DoorOpen,
  Globe,
  Server,
  Image,
  Terminal,
  LogOut,
} from 'lucide-vue-next'

const router = useRouter()
const connection = useConnectionStore()

const navItems = [
  { title: 'Overview', icon: LayoutDashboard, to: '/' },
  { title: 'Users', icon: Users, to: '/users' },
  { title: 'Rooms', icon: DoorOpen, to: '/rooms' },
  { title: 'Federation', icon: Globe, to: '/federation' },
  { title: 'Server', icon: Server, to: '/server' },
  { title: 'Media', icon: Image, to: '/media' },
  { title: 'Console', icon: Terminal, to: '/console' },
]

async function handleDisconnect() {
  await connection.disconnect()
  router.push({ name: 'setup' })
}
</script>

<template>
  <Sidebar>
    <SidebarHeader class="p-4">
      <span class="text-lg font-bold">uwu-admin</span>
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>Navigation</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in navItems" :key="item.title">
              <SidebarMenuButton as-child :tooltip="item.title">
                <RouterLink :to="item.to">
                  <component :is="item.icon" />
                  <span>{{ item.title }}</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>

          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter class="p-4">
      <Button variant="ghost" class="w-full justify-start gap-2" @click="handleDisconnect">
        <LogOut class="size-4" />
        Disconnect
      </Button>
    </SidebarFooter>
  </Sidebar>
</template>
