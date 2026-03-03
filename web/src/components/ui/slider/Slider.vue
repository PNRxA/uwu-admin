<script setup lang="ts">
import type { SliderRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<
  SliderRootProps & {
    class?: HTMLAttributes['class']
    trackClass?: HTMLAttributes['class']
    rangeClass?: HTMLAttributes['class']
    thumbClass?: HTMLAttributes['class']
  }
>()

const emits = defineEmits<{
  'update:modelValue': [value: number[]]
}>()

const delegatedProps = reactiveOmit(props, 'class', 'trackClass', 'rangeClass', 'thumbClass')
</script>

<template>
  <SliderRoot
    v-bind="delegatedProps"
    :class="
      cn(
        'relative flex w-full touch-none select-none items-center',
        props.class,
      )
    "
    @update:model-value="emits('update:modelValue', $event!)"
  >
    <SliderTrack
      :class="
        cn(
          'relative h-1.5 w-full grow overflow-hidden rounded-full bg-primary/20',
          props.trackClass,
        )
      "
    >
      <SliderRange
        :class="
          cn(
            'absolute h-full bg-primary',
            props.rangeClass,
          )
        "
      />
    </SliderTrack>
    <SliderThumb
      v-for="(_, i) in (modelValue ?? [0]).length"
      :key="i"
      :class="
        cn(
          'block h-4 w-4 rounded-full border border-primary/50 bg-background shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50',
          props.thumbClass,
        )
      "
    />
  </SliderRoot>
</template>
