import { createRouter, createWebHistory } from 'vue-router'
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
  const connection = useConnectionStore()

  if (!connection.connected) {
    await connection.checkStatus()
  }

  if (!connection.connected && to.name !== 'setup') {
    return { name: 'setup' }
  }

  if (connection.connected && to.name === 'setup') {
    return { name: 'overview' }
  }
})

export default router
