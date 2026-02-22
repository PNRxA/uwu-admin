<script setup lang="ts">
import { computed } from 'vue'
import { parseResponse, type ParsedResponse } from '@/lib/response-parser'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

const props = defineProps<{ response: string }>()

const parsed = computed<ParsedResponse>(() => parseResponse(props.response))
</script>

<template>
  <div class="max-h-[60vh] overflow-auto">
    <template v-if="parsed.type === 'table'">
      <p v-if="parsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ parsed.header }}</p>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead v-for="col in parsed.columns" :key="col">{{ col }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(row, i) in parsed.rows" :key="i">
            <TableCell
              v-for="(cell, j) in row"
              :key="j"
              :class="j === 0 ? 'font-mono text-sm max-w-64 truncate' : ''"
            >
              {{ cell }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>

    <template v-else-if="parsed.type === 'list'">
      <p v-if="parsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ parsed.header }}</p>
      <Table>
        <TableBody>
          <TableRow v-for="item in parsed.items" :key="item">
            <TableCell class="font-mono text-sm">{{ item }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>

    <template v-else-if="parsed.type === 'kv'">
      <p v-if="parsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ parsed.header }}</p>
      <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
        <template v-for="entry in parsed.entries" :key="entry.key">
          <dt class="text-muted-foreground font-medium">{{ entry.key }}</dt>
          <dd class="font-mono truncate">{{ entry.value }}</dd>
        </template>
      </dl>
    </template>

    <template v-else>
      <pre class="whitespace-pre-wrap text-sm rounded-md bg-muted p-4">{{ parsed.text }}</pre>
    </template>
  </div>
</template>
