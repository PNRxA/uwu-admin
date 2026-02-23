<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { watch } from 'vue'
import { useI18n } from 'vue-i18n'
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
import AppLogo from '@/components/AppLogo.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const auth = useAuthStore()
const { setOpenMobile } = useSidebar()

watch(() => route.path, () => {
  setOpenMobile(false)
})

const navItems = [
  { titleKey: 'sidebar.overview', icon: LayoutDashboard, to: '/' },
  { titleKey: 'sidebar.users', icon: Users, to: '/users' },
  { titleKey: 'sidebar.rooms', icon: DoorOpen, to: '/rooms' },
  { titleKey: 'sidebar.federation', icon: Globe, to: '/federation' },
  { titleKey: 'sidebar.server', icon: Server, to: '/server' },
  { titleKey: 'sidebar.console', icon: Terminal, to: '/console' },
]

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
              <SidebarMenuButton as-child :tooltip="t(item.titleKey)" :is-active="item.to === '/' ? route.path === '/' : route.path.startsWith(item.to)">
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
    <SidebarFooter class="p-4">
      <Button variant="ghost" class="w-full justify-start gap-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:gap-0" @click="handleLogout">
        <LogOut class="size-4" />
        <span class="group-data-[collapsible=icon]:hidden">{{ $t('sidebar.logout') }}</span>
      </Button>
    </SidebarFooter>
  </Sidebar>
</template>
