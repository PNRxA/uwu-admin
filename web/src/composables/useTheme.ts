import { ref, computed, watchEffect } from 'vue'

type Theme = 'light' | 'dark' | 'system'

const theme = ref<Theme>((localStorage.getItem('theme') as Theme) || 'system')

const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
const systemIsDark = ref(mediaQuery.matches)

mediaQuery.addEventListener('change', (e) => {
  systemIsDark.value = e.matches
})

const resolvedTheme = computed(() =>
  theme.value === 'system' ? (systemIsDark.value ? 'dark' : 'light') : theme.value,
)

watchEffect(() => {
  document.documentElement.classList.toggle('dark', resolvedTheme.value === 'dark')
})

export function useTheme() {
  function setTheme(value: Theme) {
    theme.value = value
    localStorage.setItem('theme', value)
  }

  return { theme, resolvedTheme, setTheme }
}
