<script setup lang="ts">
import { Sun, Moon, SunMoon } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useTheme } from '@/composables/useTheme'

const { theme, setTheme } = useTheme()
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon">
        <Sun v-if="theme === 'light'" class="size-4" />
        <Moon v-else-if="theme === 'dark'" class="size-4" />
        <SunMoon v-else class="size-4" />
        <span class="sr-only">Toggle theme</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end">
      <DropdownMenuItem @click="setTheme('light')" :class="{ 'bg-accent': theme === 'light' }">
        <Sun class="size-4" />
        Light
      </DropdownMenuItem>
      <DropdownMenuItem @click="setTheme('dark')" :class="{ 'bg-accent': theme === 'dark' }">
        <Moon class="size-4" />
        Dark
      </DropdownMenuItem>
      <DropdownMenuItem @click="setTheme('system')" :class="{ 'bg-accent': theme === 'system' }">
        <SunMoon class="size-4" />
        System
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
