<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useSidebar } from '@/components/ui/sidebar/utils'

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
  Terminal,
  LogOut,
} from 'lucide-vue-next'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()
const { setOpenMobile } = useSidebar()

watch(() => route.path, () => {
  setOpenMobile(false)
})

const navItems = [
  { title: 'Overview', icon: LayoutDashboard, to: '/' },
  { title: 'Users', icon: Users, to: '/users' },
  { title: 'Rooms', icon: DoorOpen, to: '/rooms' },
  { title: 'Federation', icon: Globe, to: '/federation' },
  { title: 'Server', icon: Server, to: '/server' },
  { title: 'Console', icon: Terminal, to: '/console' },
]

async function handleLogout() {
  await auth.logout()
  router.push({ name: 'login' })
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
              <SidebarMenuButton as-child :tooltip="item.title" :is-active="item.to === '/' ? route.path === '/' : route.path.startsWith(item.to)">
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
      <Button variant="ghost" class="w-full justify-start gap-2" @click="handleLogout">
        <LogOut class="size-4" />
        Logout
      </Button>
    </SidebarFooter>
  </Sidebar>
</template>
