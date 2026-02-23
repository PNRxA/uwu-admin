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
      path: '/',
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

  // If authenticated and trying to visit login/setup, redirect to overview
  if (auth.authenticated && (to.name === 'login' || to.name === 'setup')) {
    return { name: 'overview' }
  }

  // If authenticated and going to dashboard, ensure servers are loaded
  if (auth.authenticated && to.name !== 'login' && to.name !== 'setup') {
    const connection = useConnectionStore()
    if (!connection.loaded) {
      await connection.fetchServers()
    }
  }
})

export default router
