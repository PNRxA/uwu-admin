<script setup lang="ts">
import { computed, ref } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { parseResponse, type ParsedResponse } from '@/lib/response-parser'
import { Table, TableBody, TableCell, TableRow } from '@/components/ui/table'

function showTitleIfTruncated(e: MouseEvent) {
  const el = e.currentTarget as HTMLElement
  el.title = el.scrollWidth > el.clientWidth ? el.textContent ?? '' : ''
}

const props = defineProps<{ response: string }>()

const resolvedParsed = computed<ParsedResponse>(() => parseResponse(props.response))

const tableScrollRef = ref<HTMLElement | null>(null)

const rowCount = computed(() =>
  resolvedParsed.value.type === 'table' ? resolvedParsed.value.rows.length : 0,
)

const virtualizer = useVirtualizer(
  computed(() => ({
    count: rowCount.value,
    getScrollElement: () => tableScrollRef.value,
    estimateSize: () => 37,
    overscan: 20,
  })),
)

const virtualRows = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="max-h-[70vh] overflow-auto" :ref="(el) => { tableScrollRef = resolvedParsed.type === 'table' ? el as HTMLElement : null }">
    <template v-if="resolvedParsed.type === 'table'">
      <p v-if="resolvedParsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ resolvedParsed.header }}</p>
      <div :style="{ height: `${totalSize}px`, position: 'relative' }">
        <table class="w-full caption-bottom text-sm" :style="{ position: 'absolute', top: 0, left: 0, width: '100%', transform: `translateY(${virtualRows[0]?.start ?? 0}px)` }">
          <thead v-if="resolvedParsed.columns.some((c) => c)" class="[&_tr]:border-b">
            <tr class="hover:bg-muted/50 border-b transition-colors">
              <th v-for="col in resolvedParsed.columns" :key="col" class="text-foreground h-10 px-2 text-left align-middle font-medium whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]">{{ col }}</th>
            </tr>
          </thead>
          <tbody class="[&_tr:last-child]:border-0">
            <tr
              v-for="virtualRow in virtualRows"
              :key="virtualRow.index"
              :data-index="virtualRow.index"
              :ref="(el) => virtualizer.measureElement(el as HTMLElement)"
              class="hover:bg-muted/50 border-b transition-colors"
            >
              <td
                v-for="(cell, j) in resolvedParsed.rows[virtualRow.index]"
                :key="j"
                :class="[
                  'p-2 align-middle whitespace-nowrap',
                  j === 0 ? 'font-mono text-sm max-w-64' : 'max-w-64',
                ]"
              >
                <span class="block truncate" @mouseenter="showTitleIfTruncated">{{ cell }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <template v-else-if="resolvedParsed.type === 'list'">
      <p v-if="resolvedParsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ resolvedParsed.header }}</p>
      <Table>
        <TableBody>
          <TableRow v-for="item in resolvedParsed.items" :key="item">
            <TableCell class="font-mono text-sm">{{ item }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>

    <template v-else-if="resolvedParsed.type === 'kv'">
      <p v-if="resolvedParsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ resolvedParsed.header }}</p>
      <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
        <template v-for="entry in resolvedParsed.entries" :key="entry.key">
          <dt class="text-muted-foreground font-medium">{{ entry.key }}</dt>
          <dd class="font-mono truncate" @mouseenter="showTitleIfTruncated">{{ entry.value }}</dd>
        </template>
      </dl>
    </template>

    <template v-else>
      <pre class="whitespace-pre-wrap text-sm rounded-md bg-muted p-4">{{ resolvedParsed.text }}</pre>
    </template>
  </div>
</template>
