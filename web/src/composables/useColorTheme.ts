import { watch } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { api } from '@/lib/api'
import {
  type ThemeDefinition,
  generateThemeCSS,
  parseThemeSettings,
  resolveTheme,
  isDefaultTheme,
  BUILTIN_THEMES,
} from '@/lib/themes'

const STYLE_ID = 'uwu-theme-overrides'
const CACHE_KEY_CSS = 'uwu-theme-css'
const CACHE_KEY_DEF = 'uwu-theme-def'

function injectCSS(css: string) {
  // Always remove and re-append to ensure the override is the LAST style
  // in <head>, so it wins over the base style.css by cascade order.
  let el = document.getElementById(STYLE_ID) as HTMLStyleElement | null
  if (el) el.remove()
  el = document.createElement('style')
  el.id = STYLE_ID
  el.textContent = css
  document.head.appendChild(el)
}

function removeCSS() {
  document.getElementById(STYLE_ID)?.remove()
}

function cacheTheme(theme: ThemeDefinition, css: string) {
  try {
    localStorage.setItem(CACHE_KEY_CSS, css)
    localStorage.setItem(CACHE_KEY_DEF, JSON.stringify(theme))
  } catch {
    // quota exceeded, ignore
  }
}

function clearCache() {
  localStorage.removeItem(CACHE_KEY_CSS)
  localStorage.removeItem(CACHE_KEY_DEF)
}

function applyThemeFromRaw(raw: string | undefined) {
  const themeSettings = parseThemeSettings(raw)
  const theme = resolveTheme(themeSettings)

  if (isDefaultTheme(themeSettings) && theme.id === BUILTIN_THEMES[0]!.id) {
    removeCSS()
    clearCache()
    return
  }

  const css = generateThemeCSS(theme)
  injectCSS(css)
  cacheTheme(theme, css)
}

/**
 * Called before app mount to re-inject cached theme CSS.
 *
 * The inline <script> in index.html already injects the same CSS for zero-flash
 * before any JS bundle loads. This runtime call re-appends the <style> so it
 * stays last in <head> after Vite injects its own style tags (which could push
 * the override above style.css and lose specificity).
 *
 * Also kicks off a fetch of public settings to update/populate the cache.
 */
export function applyCachedTheme() {
  // 1. Apply from localStorage cache immediately (synchronous)
  try {
    const css = localStorage.getItem(CACHE_KEY_CSS)
    if (css) {
      injectCSS(css)
    }
  } catch {
    // ignore
  }

  // 2. Fetch public settings from server to update/populate cache.
  //    This covers incognito (no localStorage) and stale caches.
  fetchAndApplyPublicSettings()
}

async function fetchAndApplyPublicSettings() {
  try {
    const settings = await api.getPublicSettings()
    applyThemeFromRaw(settings.theme)

    // Also cache flavour text for login page
    try {
      if (settings.flavour_text !== undefined) {
        localStorage.setItem('uwu-flavour-text', settings.flavour_text)
        document.title = settings.flavour_text !== 'false' ? 'uwu admin' : 'Admin Panel'
      }
    } catch {
      // ignore
    }
  } catch {
    // Server unreachable — keep using cached theme if any
  }
}

/**
 * Returns the cached ThemeDefinition, if any.
 */
export function getCachedTheme(): ThemeDefinition | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY_DEF)
    if (raw) return JSON.parse(raw) as ThemeDefinition
  } catch {
    // ignore
  }
  return null
}

/**
 * Composable: watches settings store and applies theme CSS.
 */
export function useColorTheme() {
  const settingsStore = useSettingsStore()

  watch(
    () => settingsStore.settings.theme,
    (raw) => {
      // Don't clear the cached theme when settings haven't been fetched yet.
      // The public settings fetch in applyCachedTheme() handles the pre-auth case.
      if (raw === undefined) return

      applyThemeFromRaw(raw)
    },
    { immediate: true },
  )
}
