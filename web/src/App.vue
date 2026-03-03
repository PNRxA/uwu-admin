<template>
  <RouterView />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { useColorTheme } from '@/composables/useColorTheme'

useTheme()
useColorTheme()
</script>
