<script lang="ts" setup>
import type { ToasterProps } from "vue-sonner"
import { CircleCheckIcon, InfoIcon, Loader2Icon, OctagonXIcon, TriangleAlertIcon, XIcon } from "lucide-vue-next"
import { Toaster as Sonner } from "vue-sonner"
import { cn } from "@/lib/utils"

const props = defineProps<ToasterProps>()
</script>

<template>
  <Sonner
    :class="cn('toaster group', props.class)"
    :style="{
      '--normal-bg': 'var(--popover)',
      '--normal-text': 'var(--popover-foreground)',
      '--normal-border': 'var(--border)',
      '--success-bg': 'var(--toast-success-bg)',
      '--success-text': 'var(--toast-success-text)',
      '--success-border': 'var(--toast-success-border)',
      '--error-bg': 'var(--toast-error-bg)',
      '--error-text': 'var(--toast-error-text)',
      '--error-border': 'var(--toast-error-border)',
      '--warning-bg': 'var(--toast-warning-bg)',
      '--warning-text': 'var(--toast-warning-text)',
      '--warning-border': 'var(--toast-warning-border)',
      '--info-bg': 'var(--toast-info-bg)',
      '--info-text': 'var(--toast-info-text)',
      '--info-border': 'var(--toast-info-border)',
      '--border-radius': 'var(--radius)',
    }"
    v-bind="props"
  >
    <template #success-icon>
      <CircleCheckIcon class="size-4" />
    </template>
    <template #info-icon>
      <InfoIcon class="size-4" />
    </template>
    <template #warning-icon>
      <TriangleAlertIcon class="size-4" />
    </template>
    <template #error-icon>
      <OctagonXIcon class="size-4" />
    </template>
    <template #loading-icon>
      <div>
        <Loader2Icon class="size-4 animate-spin" />
      </div>
    </template>
    <template #close-icon>
      <XIcon class="size-4" />
    </template>
  </Sonner>
</template>
