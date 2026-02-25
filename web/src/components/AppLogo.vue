<script setup lang="ts">
import { RouterLink } from 'vue-router'

withDefaults(defineProps<{
  size?: 'sm' | 'lg'
}>(), {
  size: 'sm',
})
</script>

<template>
  <RouterLink to="/" class="uwu-logo flex items-center gap-1.5 overflow-visible whitespace-nowrap no-underline">
    <span
      class="uwu-text relative font-extrabold tracking-tight bg-gradient-to-r from-primary to-pink-400 dark:from-primary dark:to-pink-300 bg-clip-text text-transparent group-data-[collapsible=icon]:text-sm"
      :class="size === 'lg' ? 'text-3xl' : 'text-xl'"
    >uwu</span>
    <span
      class="font-light tracking-wide text-muted-foreground group-data-[collapsible=icon]:hidden"
      :class="size === 'lg' ? 'text-3xl' : 'text-xl'"
    >admin</span>
  </RouterLink>
</template>

<style scoped>
.uwu-text::before {
  content: '';
  position: absolute;
  top: -2px;
  right: -8px;
  width: 10px;
  height: 10px;
  background: currentColor;
  clip-path: polygon(
    50% 0%, 61% 35%, 98% 35%, 68% 57%,
    79% 91%, 50% 70%, 21% 91%, 32% 57%,
    2% 35%, 39% 35%
  );
  background: linear-gradient(135deg, oklch(0.72 0.18 350), oklch(0.68 0.16 330));
  opacity: 0.8;
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.uwu-logo:hover .uwu-text::before {
  transform: scale(1.3) rotate(15deg);
  opacity: 1;
}

.uwu-logo:hover .uwu-text {
  animation: uwu-wiggle 0.5s ease-in-out;
}

@keyframes uwu-wiggle {
  0%, 100% { transform: rotate(0deg); }
  20% { transform: rotate(-3deg); }
  40% { transform: rotate(3deg); }
  60% { transform: rotate(-2deg); }
  80% { transform: rotate(2deg); }
}
</style>
