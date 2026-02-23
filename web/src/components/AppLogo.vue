<script setup lang="ts">
withDefaults(defineProps<{
  size?: 'sm' | 'lg'
}>(), {
  size: 'sm',
})
</script>

<template>
  <div class="flex items-center gap-1.5 overflow-hidden whitespace-nowrap">
    <span
      class="font-extrabold tracking-tight bg-gradient-to-r from-primary to-pink-400 dark:from-primary dark:to-pink-300 bg-clip-text text-transparent group-data-[collapsible=icon]:text-sm"
      :class="size === 'lg' ? 'text-3xl' : 'text-xl'"
    >uwu</span>
    <span
      class="font-light tracking-wide text-muted-foreground group-data-[collapsible=icon]:hidden"
      :class="size === 'lg' ? 'text-3xl' : 'text-xl'"
    >admin</span>
  </div>
</template>
