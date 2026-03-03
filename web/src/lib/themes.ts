export interface ThemeDefinition {
  id: string
  name: string
  builtin?: boolean
  hue: number
  chromaScale: number
}

export interface ThemeSettings {
  activeThemeId: string
  customThemes: ThemeDefinition[]
}

interface ThemeVariableBase {
  name: string
  lightness: number
  chroma: number
  /** Hue offset from primary (0 = same as primary hue) */
  hueOffset: number
  /** If true, use opacity syntax like `oklch(L C H / alpha%)` */
  alpha?: number
}

// Extracted from style.css - the current uwu theme uses hue=350 (light) and hue=335 (dark).
// All values below are relative to the primary hue. The dark mode hue offset is -15.
const LIGHT_HUE = 350
const DARK_HUE = 335
const DARK_HUE_OFFSET = DARK_HUE - LIGHT_HUE // -15

const LIGHT_VARIABLES: ThemeVariableBase[] = [
  { name: 'background', lightness: 0.975, chroma: 0.015, hueOffset: 0 },
  { name: 'foreground', lightness: 0.2, chroma: 0.025, hueOffset: 0 },
  { name: 'card', lightness: 0.985, chroma: 0.01, hueOffset: 0 },
  { name: 'card-foreground', lightness: 0.2, chroma: 0.025, hueOffset: 0 },
  { name: 'popover', lightness: 0.995, chroma: 0.005, hueOffset: 0 },
  { name: 'popover-foreground', lightness: 0.2, chroma: 0.025, hueOffset: 0 },
  { name: 'primary', lightness: 0.58, chroma: 0.2, hueOffset: 0 },
  { name: 'primary-foreground', lightness: 0.98, chroma: 0.01, hueOffset: 0 },
  { name: 'secondary', lightness: 0.95, chroma: 0.025, hueOffset: 0 },
  { name: 'secondary-foreground', lightness: 0.3, chroma: 0.04, hueOffset: 0 },
  { name: 'muted', lightness: 0.94, chroma: 0.02, hueOffset: 0 },
  { name: 'muted-foreground', lightness: 0.52, chroma: 0.05, hueOffset: 0 },
  { name: 'accent', lightness: 0.94, chroma: 0.03, hueOffset: 0 },
  { name: 'accent-foreground', lightness: 0.3, chroma: 0.04, hueOffset: 0 },
  { name: 'border', lightness: 0.91, chroma: 0.025, hueOffset: 0 },
  { name: 'input', lightness: 0.91, chroma: 0.025, hueOffset: 0 },
  { name: 'ring', lightness: 0.58, chroma: 0.2, hueOffset: 0 },
  { name: 'chart-1', lightness: 0.58, chroma: 0.2, hueOffset: 0 },
  { name: 'chart-2', lightness: 0.62, chroma: 0.14, hueOffset: -30 },
  { name: 'chart-3', lightness: 0.48, chroma: 0.15, hueOffset: 0 },
  { name: 'chart-4', lightness: 0.68, chroma: 0.13, hueOffset: 20 },
  { name: 'chart-5', lightness: 0.65, chroma: 0.12, hueOffset: -15 },
  { name: 'sidebar', lightness: 0.965, chroma: 0.018, hueOffset: 0 },
  { name: 'sidebar-foreground', lightness: 0.2, chroma: 0.025, hueOffset: 0 },
  { name: 'sidebar-primary', lightness: 0.58, chroma: 0.2, hueOffset: 0 },
  { name: 'sidebar-primary-foreground', lightness: 0.98, chroma: 0.01, hueOffset: 0 },
  { name: 'sidebar-accent', lightness: 0.93, chroma: 0.03, hueOffset: 0 },
  { name: 'sidebar-accent-foreground', lightness: 0.3, chroma: 0.04, hueOffset: 0 },
  { name: 'sidebar-border', lightness: 0.91, chroma: 0.025, hueOffset: 0 },
  { name: 'sidebar-ring', lightness: 0.58, chroma: 0.2, hueOffset: 0 },
]

const DARK_VARIABLES: ThemeVariableBase[] = [
  { name: 'background', lightness: 0.2, chroma: 0.04, hueOffset: 0 },
  { name: 'foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'card', lightness: 0.24, chroma: 0.045, hueOffset: 0 },
  { name: 'card-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'popover', lightness: 0.27, chroma: 0.035, hueOffset: 0 },
  { name: 'popover-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'primary', lightness: 0.72, chroma: 0.18, hueOffset: 0 },
  { name: 'primary-foreground', lightness: 0.15, chroma: 0.04, hueOffset: 0 },
  { name: 'secondary', lightness: 0.3, chroma: 0.055, hueOffset: 0 },
  { name: 'secondary-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'muted', lightness: 0.3, chroma: 0.05, hueOffset: 0 },
  { name: 'muted-foreground', lightness: 0.67, chroma: 0.07, hueOffset: 0 },
  { name: 'accent', lightness: 0.32, chroma: 0.055, hueOffset: 0 },
  { name: 'accent-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'border', lightness: 1, chroma: 0.045, hueOffset: 0, alpha: 14 },
  { name: 'input', lightness: 1, chroma: 0.045, hueOffset: 0, alpha: 17 },
  { name: 'ring', lightness: 0.72, chroma: 0.18, hueOffset: 0 },
  { name: 'chart-1', lightness: 0.72, chroma: 0.18, hueOffset: 0 },
  { name: 'chart-2', lightness: 0.62, chroma: 0.14, hueOffset: -15 },
  { name: 'chart-3', lightness: 0.78, chroma: 0.12, hueOffset: 35 },
  { name: 'chart-4', lightness: 0.55, chroma: 0.16, hueOffset: 0 },
  { name: 'chart-5', lightness: 0.75, chroma: 0.14, hueOffset: 5 },
  { name: 'sidebar', lightness: 0.22, chroma: 0.045, hueOffset: 0 },
  { name: 'sidebar-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'sidebar-primary', lightness: 0.72, chroma: 0.18, hueOffset: 0 },
  { name: 'sidebar-primary-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'sidebar-accent', lightness: 0.32, chroma: 0.055, hueOffset: 0 },
  { name: 'sidebar-accent-foreground', lightness: 0.93, chroma: 0.02, hueOffset: 0 },
  { name: 'sidebar-border', lightness: 1, chroma: 0.045, hueOffset: 0, alpha: 14 },
  { name: 'sidebar-ring', lightness: 0.72, chroma: 0.18, hueOffset: 0 },
]

// Shadow uses primary hue too
function shadowCuteLight(hue: number, chroma: number): string {
  const c = Math.round(chroma * 0.7 * 1000) / 1000
  return `0 2px 16px -2px oklch(0.72 ${c} ${hue} / 0.18)`
}

function shadowCuteDark(hue: number, chroma: number): string {
  const c = Math.round(chroma * 0.7 * 1000) / 1000
  return `0 2px 16px -2px oklch(0.55 ${c} ${hue} / 0.2)`
}

function formatOklch(l: number, c: number, h: number, alpha?: number): string {
  const cl = Math.round(l * 1000) / 1000
  const cc = Math.round(c * 1000) / 1000
  const ch = Math.round(h * 10) / 10
  if (alpha !== undefined) {
    return `oklch(${cl} ${cc} ${ch} / ${alpha}%)`
  }
  return `oklch(${cl} ${cc} ${ch})`
}

function wrapHue(h: number): number {
  return ((h % 360) + 360) % 360
}

function generateVariables(
  bases: ThemeVariableBase[],
  hue: number,
  chromaScale: number,
): Record<string, string> {
  const vars: Record<string, string> = {}
  for (const v of bases) {
    const h = wrapHue(hue + v.hueOffset)
    const c = v.chroma * chromaScale
    vars[`--${v.name}`] = formatOklch(v.lightness, c, h, v.alpha)
  }
  return vars
}

export function generateThemeVariables(theme: ThemeDefinition): {
  light: Record<string, string>
  dark: Record<string, string>
} {
  const darkHue = theme.hue + DARK_HUE_OFFSET
  const light = generateVariables(LIGHT_VARIABLES, theme.hue, theme.chromaScale)
  const dark = generateVariables(DARK_VARIABLES, darkHue, theme.chromaScale)

  // Add shadow-cute
  light['--shadow-cute'] = shadowCuteLight(theme.hue, theme.chromaScale)
  dark['--shadow-cute'] = shadowCuteDark(darkHue, theme.chromaScale)

  return { light, dark }
}

export function generateThemeCSS(theme: ThemeDefinition): string {
  const { light, dark } = generateThemeVariables(theme)

  const lightRules = Object.entries(light)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join('\n')
  const darkRules = Object.entries(dark)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join('\n')

  return `:root {\n${lightRules}\n}\n.dark {\n${darkRules}\n}`
}

export const BUILTIN_THEMES: ThemeDefinition[] = [
  { id: 'uwu', name: 'uwu pink', builtin: true, hue: 350, chromaScale: 1.0 },
  { id: 'slate', name: 'Slate', builtin: true, hue: 220, chromaScale: 0.6 },
  { id: 'grey', name: 'Grey', builtin: true, hue: 0, chromaScale: 0.0 },
]

export function resolveTheme(settings: ThemeSettings): ThemeDefinition {
  const all = [...BUILTIN_THEMES, ...settings.customThemes]
  return all.find((t) => t.id === settings.activeThemeId) ?? BUILTIN_THEMES[0]!
}

export function isValidThemeDefinition(t: unknown): t is ThemeDefinition {
  if (!t || typeof t !== 'object') return false
  const obj = t as Record<string, unknown>
  if (typeof obj.id !== 'string' || !obj.id) return false
  if (typeof obj.name !== 'string' || !obj.name) return false
  if (typeof obj.hue !== 'number' || obj.hue < 0 || obj.hue > 360) return false
  if (typeof obj.chromaScale !== 'number' || obj.chromaScale < 0.3 || obj.chromaScale > 1.5) return false
  return true
}

export function parseThemeSettings(raw: string | undefined): ThemeSettings {
  if (!raw) return { activeThemeId: 'uwu', customThemes: [] }
  try {
    const parsed = JSON.parse(raw)
    if (typeof parsed.activeThemeId === 'string' && Array.isArray(parsed.customThemes)) {
      return {
        activeThemeId: parsed.activeThemeId,
        customThemes: parsed.customThemes.filter(isValidThemeDefinition),
      }
    }
  } catch {
    // fall through
  }
  return { activeThemeId: 'uwu', customThemes: [] }
}

export function isDefaultTheme(settings: ThemeSettings): boolean {
  return settings.activeThemeId === 'uwu'
}

export interface ThemeExport {
  version: 1
  theme: Omit<ThemeDefinition, 'builtin'>
}

export function exportTheme(theme: ThemeDefinition): ThemeExport {
  return {
    version: 1,
    theme: { id: theme.id, name: theme.name, hue: theme.hue, chromaScale: theme.chromaScale },
  }
}

export function validateImport(data: unknown): ThemeDefinition | null {
  if (!data || typeof data !== 'object') return null
  const obj = data as Record<string, unknown>
  if (obj.version !== 1 || !obj.theme || typeof obj.theme !== 'object') return null
  if (!isValidThemeDefinition(obj.theme)) return null
  return {
    id: obj.theme.id,
    name: obj.theme.name,
    hue: obj.theme.hue,
    chromaScale: obj.theme.chromaScale,
  }
}
