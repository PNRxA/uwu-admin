<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { getSuggestions, applySuggestion, type Suggestion } from '@/composables/useCommandAutocomplete'

const props = defineProps<{
  modelValue: string
  disabled?: boolean
  placeholder?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'submit'): void
}>()

const inputRef = ref<InstanceType<typeof Input> | null>(null)
const selectedIndex = ref(0)
const isFocused = ref(false)
// Track which input value was active when Escape was pressed;
// dropdown auto-reopens as soon as the input changes.
const dismissedAt = ref<string | null>(null)

const result = computed(() => getSuggestions(props.modelValue))
const suggestions = computed(() => result.value.suggestions)
const argHints = computed(() => result.value.argHints)
const hasContent = computed(() => suggestions.value.length > 0 || argHints.value.length > 0)
const showDropdown = computed(() =>
  isFocused.value && hasContent.value && props.modelValue !== dismissedAt.value,
)

watch(suggestions, () => {
  selectedIndex.value = 0
})

function acceptSuggestion(suggestion: Suggestion) {
  const newValue = applySuggestion(props.modelValue, suggestion)
  emit('update:modelValue', newValue)
  nextTick(() => {
    const el = inputRef.value?.$el as HTMLInputElement | undefined
    el?.focus()
  })
}

function onKeydown(e: KeyboardEvent) {
  if (!hasContent.value) {
    if (e.key === 'Enter') {
      e.preventDefault()
      emit('submit')
    }
    return
  }

  switch (e.key) {
    case 'ArrowUp':
      e.preventDefault()
      if (suggestions.value.length > 0) {
        selectedIndex.value =
          (selectedIndex.value - 1 + suggestions.value.length) % suggestions.value.length
        scrollToSelected()
      }
      break
    case 'ArrowDown':
      e.preventDefault()
      if (suggestions.value.length > 0) {
        selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length
        scrollToSelected()
      }
      break
    case 'Tab':
      e.preventDefault()
      if (suggestions.value.length > 0) {
        acceptSuggestion(suggestions.value[selectedIndex.value]!)
      }
      break
    case 'Enter':
      e.preventDefault()
      if (suggestions.value.length > 0 && showDropdown.value) {
        acceptSuggestion(suggestions.value[selectedIndex.value]!)
      } else {
        emit('submit')
      }
      break
    case 'Escape':
      e.preventDefault()
      dismissedAt.value = props.modelValue
      break
  }
}

function scrollToSelected() {
  nextTick(() => {
    const el = document.querySelector('[data-autocomplete-selected="true"]')
    el?.scrollIntoView({ block: 'nearest' })
  })
}

function onInput(value: string | number) {
  emit('update:modelValue', String(value))
}

function onFocus() {
  isFocused.value = true
  dismissedAt.value = null
}

function onBlur() {
  // Delay to allow mousedown on suggestions to fire first
  setTimeout(() => {
    isFocused.value = false
  }, 150)
}
</script>

<template>
  <div class="relative flex-1">
    <!-- Dropdown (appears above the input) -->
    <div
      v-if="showDropdown"
      class="absolute bottom-full left-0 right-0 mb-1 z-50 rounded-md border bg-popover text-popover-foreground shadow-md"
    >
      <!-- Suggestions list -->
      <div v-if="suggestions.length > 0" class="max-h-64 overflow-hidden">
        <ScrollArea class="max-h-64">
          <div class="p-1">
          <button
            v-for="(suggestion, i) in suggestions"
            :key="suggestion.name"
            :data-autocomplete-selected="i === selectedIndex"
            class="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none cursor-pointer"
            :class="i === selectedIndex ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'"
            @mousedown.prevent="acceptSuggestion(suggestion)"
            @mouseenter="selectedIndex = i"
          >
            <code class="font-medium">{{ suggestion.name }}</code>
            <span class="text-muted-foreground text-xs truncate">{{ suggestion.description }}</span>
            <span v-if="suggestion.hasChildren" class="ml-auto text-muted-foreground text-xs">&rsaquo;</span>
          </button>
          </div>
        </ScrollArea>
      </div>

      <!-- Arg hints -->
      <div v-else-if="argHints.length > 0" class="p-2">
        <div class="flex flex-wrap gap-1.5 text-xs">
          <span
            v-for="hint in argHints"
            :key="hint.name"
            class="inline-flex items-center rounded px-1.5 py-0.5"
            :class="hint.required ? 'bg-accent text-accent-foreground' : 'bg-muted text-muted-foreground'"
          >
            {{ hint.required ? `&lt;${hint.name}&gt;` : `[${hint.name}]` }}
          </span>
        </div>
      </div>
    </div>

    <!-- Input -->
    <Input
      ref="inputRef"
      :model-value="modelValue"
      :disabled="disabled"
      :placeholder="placeholder"
      class="flex-1"
      @update:model-value="onInput"
      @keydown="onKeydown"
      @focus="onFocus"
      @blur="onBlur"
    />
  </div>
</template>
