<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { toast } from 'vue-sonner'
import { useSettingsStore } from '@/stores/settings'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Slider } from '@/components/ui/slider'
import { Plus, Download, Upload, Trash2 } from 'lucide-vue-next'
import {
  type ThemeDefinition,
  type ThemeSettings,
  BUILTIN_THEMES,
  parseThemeSettings,
  resolveTheme,
  generateThemeVariables,
  exportTheme,
  validateImport,
} from '@/lib/themes'

const { t } = useI18n()
const settingsStore = useSettingsStore()

onMounted(() => {
  settingsStore.fetchSettings()
})

// --- Theme state ---

const themeSettings = computed(() => parseThemeSettings(settingsStore.settings.theme))
const activeTheme = computed(() => resolveTheme(themeSettings.value))
const allThemes = computed(() => [...BUILTIN_THEMES, ...themeSettings.value.customThemes])

const editing = ref(false)
const editingExisting = ref(false)
const editorName = ref('')
const editorHue = ref(350)
const editorChroma = ref(1.0)
const editorOriginalId = ref('')

const editorPreviewColors = computed(() => {
  const theme: ThemeDefinition = {
    id: 'preview',
    name: 'Preview',
    hue: editorHue.value,
    chromaScale: editorChroma.value,
  }
  const { light } = generateThemeVariables(theme)
  return {
    primary: light['--primary'],
    secondary: light['--secondary'],
    accent: light['--accent'],
    muted: light['--muted'],
  }
})

async function selectTheme(id: string) {
  const newSettings: ThemeSettings = {
    ...themeSettings.value,
    activeThemeId: id,
  }
  await saveThemeSettings(newSettings)
}

function startCreateTheme() {
  editing.value = true
  editingExisting.value = false
  editorName.value = ''
  editorHue.value = 200
  editorChroma.value = 1.0
  editorOriginalId.value = ''
}

function startEditTheme(theme: ThemeDefinition) {
  editing.value = true
  editingExisting.value = true
  editorName.value = theme.name
  editorHue.value = theme.hue
  editorChroma.value = theme.chromaScale
  editorOriginalId.value = theme.id
}

function cancelEditor() {
  editing.value = false
}

async function saveEditor() {
  const name = editorName.value.trim()
  if (!name) return

  const id = editingExisting.value
    ? editorOriginalId.value
    : name.toLowerCase().replace(/\s+/g, '-') + '-' + Date.now().toString(36)

  const theme: ThemeDefinition = {
    id,
    name,
    hue: editorHue.value,
    chromaScale: editorChroma.value,
  }

  const customs = editingExisting.value
    ? themeSettings.value.customThemes.map((t) => (t.id === id ? theme : t))
    : [...themeSettings.value.customThemes, theme]

  const newSettings: ThemeSettings = {
    activeThemeId: id,
    customThemes: customs,
  }
  await saveThemeSettings(newSettings)
  editing.value = false
}

async function deleteTheme(id: string) {
  const customs = themeSettings.value.customThemes.filter((t) => t.id !== id)
  const activeId = themeSettings.value.activeThemeId === id ? 'uwu' : themeSettings.value.activeThemeId
  await saveThemeSettings({ activeThemeId: activeId, customThemes: customs })
  if (editing.value && editorOriginalId.value === id) {
    editing.value = false
  }
  toast.success(t('settings.themeDeleted'))
}

function exportCurrentTheme(theme: ThemeDefinition) {
  const data = exportTheme(theme)
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${theme.name.toLowerCase().replace(/\s+/g, '-')}-theme.json`
  a.click()
  URL.revokeObjectURL(url)
  toast.success(t('settings.themeExported'))
}

const fileInput = ref<HTMLInputElement>()

function triggerImport() {
  fileInput.value?.click()
}

async function handleImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  try {
    const text = await file.text()
    const data = JSON.parse(text)
    const theme = validateImport(data)
    if (!theme) {
      toast.error(t('settings.importFailed'))
      return
    }

    // Ensure unique id
    const existingIds = new Set(allThemes.value.map((t) => t.id))
    if (existingIds.has(theme.id)) {
      theme.id = theme.id + '-' + Date.now().toString(36)
    }

    const customs = [...themeSettings.value.customThemes, theme]
    await saveThemeSettings({ activeThemeId: theme.id, customThemes: customs })
    toast.success(t('settings.themeImported'))
  } catch {
    toast.error(t('settings.importFailed'))
  }

  // Reset file input
  if (fileInput.value) fileInput.value.value = ''
}

async function saveThemeSettings(settings: ThemeSettings) {
  try {
    await settingsStore.updateSetting('theme', JSON.stringify(settings))
    toast.success(t('settings.updateSuccess'))
  } catch {
    toast.error(t('settings.updateFailed'))
  }
}

// --- Flavour text ---

const flavourText = computed(() => settingsStore.settings.flavour_text !== 'false')

watch(flavourText, (val) => {
  try {
    localStorage.setItem('uwu-flavour-text', String(val))
    document.title = val ? 'uwu admin' : 'Admin Panel'
  } catch {
    // ignore
  }
})

async function onFlavourToggle(checked: boolean) {
  try {
    await settingsStore.updateSetting('flavour_text', String(checked))
    toast.success(t('settings.updateSuccess'))
  } catch {
    toast.error(t('settings.updateFailed'))
  }
}

// --- Redact ---

async function onRedactToggle(checked: boolean) {
  try {
    await settingsStore.updateSetting('redact_messages', String(checked))
    toast.success(t('settings.updateSuccess'))
  } catch {
    toast.error(t('settings.updateFailed'))
  }
}

function themeSwatchColor(theme: ThemeDefinition): string {
  const { light } = generateThemeVariables(theme)
  return light['--primary']!
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div>
      <h1 class="text-2xl font-bold">{{ t('settings.title') }}</h1>
      <p class="text-sm text-muted-foreground">{{ t('settings.subtitle') }}</p>
    </div>

    <!-- Appearance -->
    <Card>
      <CardHeader>
        <CardTitle>{{ t('settings.appearance') }}</CardTitle>
        <CardDescription>{{ t('settings.appearanceDescription') }}</CardDescription>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="settingsStore.loading" class="h-20 w-full" />
        <div v-else class="flex flex-col gap-6">
          <!-- Theme picker -->
          <div class="flex flex-wrap items-center gap-3">
            <button
              v-for="theme in allThemes"
              :key="theme.id"
              :aria-label="theme.name"
              class="group relative flex flex-col items-center gap-1.5"
              @click="selectTheme(theme.id)"
            >
              <div
                class="h-10 w-10 rounded-full border-2 transition-all"
                :class="activeTheme.id === theme.id ? 'border-foreground scale-110' : 'border-transparent hover:border-muted-foreground/40'"
                :style="{ backgroundColor: themeSwatchColor(theme) }"
              />
              <span class="text-xs text-muted-foreground max-w-[4rem] truncate">{{ theme.name }}</span>
              <!-- Edit button for custom themes -->
              <button
                v-if="!theme.builtin"
                :aria-label="t('settings.editTheme')"
                class="absolute -top-1 -right-1 hidden group-hover:flex h-4 w-4 items-center justify-center rounded-full bg-muted text-muted-foreground text-xs hover:bg-accent"
                @click.stop="startEditTheme(theme)"
              >
                &hellip;
              </button>
            </button>

            <!-- Create + Import buttons -->
            <div class="flex flex-col items-center gap-1.5">
              <button
                :aria-label="t('settings.createTheme')"
                class="flex h-10 w-10 items-center justify-center rounded-full border-2 border-dashed border-muted-foreground/30 text-muted-foreground hover:border-muted-foreground/60 hover:text-foreground transition-colors"
                @click="startCreateTheme"
              >
                <Plus class="h-4 w-4" />
              </button>
              <span class="text-xs text-muted-foreground">{{ t('settings.createTheme') }}</span>
            </div>
          </div>

          <!-- Theme editor -->
          <div v-if="editing" class="rounded-lg border bg-card p-4 flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium">{{ editingExisting ? t('settings.editTheme') : t('settings.createTheme') }}</span>
            </div>

            <div class="flex flex-col gap-2">
              <Label>{{ t('settings.themeName') }}</Label>
              <Input v-model="editorName" :placeholder="t('settings.themeNamePlaceholder')" class="max-w-xs" />
            </div>

            <div class="flex flex-col gap-2">
              <Label>{{ t('settings.themeHue') }} ({{ Math.round(editorHue) }}°)</Label>
              <Slider
                :model-value="[editorHue]"
                :min="0"
                :max="360"
                :step="1"
                class="max-w-sm"
                track-class="hue-track"
                range-class="!bg-transparent"
                @update:model-value="editorHue = $event[0]!"
              />
            </div>

            <div class="flex flex-col gap-2">
              <Label>{{ t('settings.themeChroma') }} ({{ editorChroma.toFixed(1) }})</Label>
              <Slider
                :model-value="[editorChroma * 100]"
                :min="30"
                :max="150"
                :step="5"
                class="max-w-sm"
                @update:model-value="editorChroma = $event[0]! / 100"
              />
            </div>

            <!-- Preview swatches -->
            <div class="flex gap-2">
              <div class="h-8 w-8 rounded-full" :style="{ backgroundColor: editorPreviewColors.primary }" />
              <div class="h-8 w-8 rounded-full" :style="{ backgroundColor: editorPreviewColors.accent }" />
              <div class="h-8 w-8 rounded-full" :style="{ backgroundColor: editorPreviewColors.secondary }" />
              <div class="h-8 w-8 rounded-full" :style="{ backgroundColor: editorPreviewColors.muted }" />
            </div>

            <div class="flex gap-2">
              <Button size="sm" @click="saveEditor" :disabled="!editorName.trim()">{{ t('settings.save') }}</Button>
              <Button size="sm" variant="outline" @click="cancelEditor">{{ t('common.cancel') }}</Button>
              <Button v-if="editingExisting" size="sm" variant="destructive" @click="deleteTheme(editorOriginalId)">
                <Trash2 class="h-3.5 w-3.5 mr-1" />
                {{ t('settings.delete') }}
              </Button>
            </div>
          </div>

          <!-- Export / Import -->
          <div class="flex gap-2">
            <Button
              v-if="!activeTheme.builtin"
              size="sm"
              variant="outline"
              @click="exportCurrentTheme(activeTheme)"
            >
              <Download class="h-3.5 w-3.5 mr-1.5" />
              {{ t('settings.exportTheme') }}
            </Button>
            <Button size="sm" variant="outline" @click="triggerImport">
              <Upload class="h-3.5 w-3.5 mr-1.5" />
              {{ t('settings.importTheme') }}
            </Button>
            <input
              ref="fileInput"
              type="file"
              accept=".json"
              class="hidden"
              @change="handleImport"
            />
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- Flavour text -->
    <Card>
      <CardHeader>
        <CardTitle>{{ t('settings.flavour') }}</CardTitle>
        <CardDescription>{{ t('settings.flavourDescription') }}</CardDescription>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="settingsStore.loading" class="h-6 w-full" />
        <div v-else class="flex items-center justify-between gap-4">
          <Label for="uwu-mode" class="flex flex-col items-start gap-1">
            <span>{{ t('settings.uwuMode') }}</span>
            <span class="text-sm font-normal text-muted-foreground">
              {{ t('settings.uwuModeDescription') }}
            </span>
          </Label>
          <Switch
            id="uwu-mode"
            :model-value="flavourText"
            @update:model-value="onFlavourToggle"
          />
        </div>
      </CardContent>
    </Card>

    <!-- Admin Room -->
    <Card>
      <CardHeader>
        <CardTitle>{{ t('settings.adminRoom') }}</CardTitle>
        <CardDescription>{{ t('settings.adminRoomDescription') }}</CardDescription>
      </CardHeader>
      <CardContent>
        <Skeleton v-if="settingsStore.loading" class="h-6 w-full" />
        <div v-else class="flex items-center justify-between gap-4">
          <Label for="redact-messages" class="flex flex-col items-start gap-1">
            <span>{{ t('settings.redactMessages') }}</span>
            <span class="text-sm font-normal text-muted-foreground">
              {{ t('settings.redactMessagesDescription') }}
            </span>
          </Label>
          <Switch
            id="redact-messages"
            :model-value="settingsStore.settings.redact_messages !== 'false'"
            @update:model-value="onRedactToggle"
          />
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<style scoped>
:deep(.hue-track) {
  background: linear-gradient(to right,
    oklch(0.65 0.2 0),
    oklch(0.65 0.2 60),
    oklch(0.65 0.2 120),
    oklch(0.65 0.2 180),
    oklch(0.65 0.2 240),
    oklch(0.65 0.2 300),
    oklch(0.65 0.2 360)
  ) !important;
}
</style>
