import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query'

import App from './App.vue'
import router from './router'
import i18n from './i18n'
import './style.css'
import { applyCachedTheme } from './composables/useColorTheme'

// Re-inject cached theme CSS so the override <style> stays last in <head>,
// surviving Vite's own style-tag injection which can reorder elements.
// The inline <script> in index.html handles the initial zero-flash case.
applyCachedTheme()

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)
app.use(VueQueryPlugin, {
  queryClientConfig: {
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        retry: false,
      },
    },
  },
})

app.mount('#app')
