<script setup lang="ts">
import { ref } from 'vue'
import { Copy, Check } from 'lucide-vue-next'

const props = withDefaults(
  defineProps<{
    value: string
    align?: 'left' | 'right'
  }>(),
  {
    align: 'left',
  },
)

const copied = ref(false)

function showTitleIfTruncated(e: MouseEvent) {
  const el = e.currentTarget as HTMLElement
  el.title = el.scrollWidth > el.clientWidth ? props.value : ''
}

async function copyValue() {
  await navigator.clipboard.writeText(props.value)
  copied.value = true
  setTimeout(() => {
    copied.value = false
  }, 1500)
}
</script>

<template>
  <div :class="['group/copy flex items-center gap-1 min-w-0', align === 'right' && 'justify-end']">
    <span class="min-w-0 truncate cursor-pointer" @mouseenter="showTitleIfTruncated" @click.stop="copyValue">{{ value }}</span>
    <button
      type="button"
      aria-label="Copy"
      class="shrink-0 invisible group-hover/copy:visible text-muted-foreground hover:text-foreground cursor-pointer"
      @click.stop="copyValue"
    >
      <Check v-if="copied" class="size-3" />
      <Copy v-else class="size-3" />
    </button>
  </div>
</template>
