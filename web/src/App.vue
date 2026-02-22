<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'

const count = ref(0)
</script>

<template>
  <div class="flex min-h-screen items-center justify-center">
    <div class="flex flex-col items-center gap-4">
      <h1 class="text-3xl font-bold">uwu-admin</h1>
      <p class="text-muted-foreground">shadcn-vue is working! Count: {{ count }}</p>
      <div class="flex gap-2">
        <Button @click="count++">Increment</Button>
        <Button variant="secondary" @click="count = 0">Reset</Button>
        <Button variant="destructive" @click="count--">Decrement</Button>
        <Button variant="outline">Outline</Button>
        <Button variant="ghost">Ghost</Button>
      </div>
    </div>
  </div>
</template>
