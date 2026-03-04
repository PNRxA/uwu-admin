import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useConnectionStore } from '@/stores/connection'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
    },
    {
      path: '/servers/:serverId(\\d+)',
      component: () => import('@/layouts/DashboardLayout.vue'),
      children: [
        {
          path: '',
          name: 'overview',
          component: () => import('@/views/OverviewView.vue'),
        },
        {
          path: 'users',
          name: 'users',
          component: () => import('@/views/UsersView.vue'),
        },
        {
          path: 'rooms',
          name: 'rooms',
          component: () => import('@/views/RoomsView.vue'),
        },
        {
          path: 'federation',
          name: 'federation',
          component: () => import('@/views/FederationView.vue'),
        },
        {
          path: 'server',
          name: 'server',
          component: () => import('@/views/ServerView.vue'),
        },
        {
          path: 'console',
          name: 'console',
          component: () => import('@/views/ConsoleView.vue'),
        },
      ],
    },
    {
      path: '/settings',
      component: () => import('@/layouts/DashboardLayout.vue'),
      children: [
        {
          path: '',
          name: 'settings',
          component: () => import('@/views/SettingsView.vue'),
        },
      ],
    },
    {
      path: '/',
      component: () => import('@/layouts/DashboardLayout.vue'),
      children: [
        {
          path: '',
          name: 'root',
          // Guard redirects to /servers/:id when servers exist.
          // When no servers exist, renders overview shell so the user can add one.
          component: () => import('@/views/OverviewView.vue'),
        },
      ],
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: { name: 'root' },
    },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()

  // Check auth status on first load
  if (!auth.initialized) {
    await auth.checkAuthStatus()
  }

  // If setup is required, redirect to setup page
  if (auth.setupRequired && to.name !== 'setup') {
    return { name: 'setup' }
  }

  // If setup is done but not authenticated, redirect to login
  if (!auth.setupRequired && !auth.authenticated && to.name !== 'login') {
    return { name: 'login' }
  }

  // If authenticated and trying to visit login/setup, redirect to root (guard will handle server redirect)
  if (auth.authenticated && (to.name === 'login' || to.name === 'setup')) {
    return { name: 'root' }
  }

  // If authenticated and going to dashboard, ensure servers are loaded
  if (auth.authenticated && to.name !== 'login' && to.name !== 'setup') {
    const connection = useConnectionStore()
    if (!connection.loaded) {
      await connection.fetchServers()
    }

    // Root path: redirect to first server
    if (to.name === 'root') {
      const firstId = connection.servers[0]?.id
      if (firstId != null) {
        return { name: 'overview', params: { serverId: firstId } }
      }
    }

    // Validate serverId param
    if (to.params.serverId) {
      const serverId = Number(to.params.serverId)
      const serverExists = connection.servers.some((s) => s.id === serverId)
      if (serverExists) {
        connection.setActiveServer(serverId)
      } else {
        // Invalid server ID: redirect to first server, preserving sub-route
        const firstId = connection.servers[0]?.id
        if (firstId != null) {
          return { name: to.name ?? 'overview', params: { serverId: firstId } }
        }
      }
    }
  }
})

export default router
