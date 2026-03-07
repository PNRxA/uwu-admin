<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useConnectionStore } from '@/stores/connection'
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
import {
  LayoutDashboard,
  Users,
  DoorOpen,
  Globe,
  Server,
  Terminal,
  Settings,
  LogOut,
} from 'lucide-vue-next'
import AppLogo from '@/components/AppLogo.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const auth = useAuthStore()
const connection = useConnectionStore()
const { setOpenMobile } = useSidebar()

watch(() => route.path, () => {
  setOpenMobile(false)
})

const navItems = computed(() => {
  const serverId = route.params.serverId ?? connection.activeServerId
  return [
    { titleKey: 'sidebar.overview', icon: LayoutDashboard, name: 'overview' as const, to: { name: 'overview' as const, params: { serverId } } },
    { titleKey: 'sidebar.users', icon: Users, name: 'users' as const, to: { name: 'users' as const, params: { serverId } } },
    { titleKey: 'sidebar.rooms', icon: DoorOpen, name: 'rooms' as const, to: { name: 'rooms' as const, params: { serverId } } },
    { titleKey: 'sidebar.federation', icon: Globe, name: 'federation' as const, to: { name: 'federation' as const, params: { serverId } } },
    { titleKey: 'sidebar.server', icon: Server, name: 'server' as const, to: { name: 'server' as const, params: { serverId } } },
    { titleKey: 'sidebar.console', icon: Terminal, name: 'console' as const, to: { name: 'console' as const, params: { serverId } } },
  ]
})

async function handleLogout() {
  await auth.logout()
  router.push({ name: 'login' })
}
</script>

<template>
  <Sidebar collapsible="icon">
    <SidebarHeader class="p-4 group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-2 group-data-[collapsible=icon]:pt-4 group-data-[collapsible=icon]:pb-2">
      <AppLogo />
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>{{ $t('sidebar.navigation') }}</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in navItems" :key="item.titleKey">
              <SidebarMenuButton as-child :tooltip="t(item.titleKey)" :is-active="route.name === item.name">
                <RouterLink :to="item.to">
                  <component :is="item.icon" />
                  <span>{{ t(item.titleKey) }}</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>

          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter class="p-4 group-data-[collapsible=icon]:p-2 flex flex-col gap-1">
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton as-child :tooltip="t('sidebar.settings')" :is-active="route.name === 'settings'">
            <RouterLink :to="{ name: 'settings' }">
              <Settings />
              <span>{{ t('sidebar.settings') }}</span>
            </RouterLink>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton :tooltip="t('sidebar.logout')" class="cursor-pointer hover:bg-primary/10 hover:text-primary dark:hover:bg-primary/15 dark:hover:text-primary" @click="handleLogout">
            <LogOut />
            <span>{{ $t('sidebar.logout') }}</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>
  </Sidebar>
</template>
