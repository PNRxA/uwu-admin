<script setup lang="ts">
import { computed } from 'vue'
import { parseResponse, type ParsedResponse } from '@/lib/response-parser'
import type { ParsedResponse as ApiParsedResponse } from '@/lib/api'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

const props = defineProps<{ response: string; parsed?: ApiParsedResponse }>()

const resolvedParsed = computed<ParsedResponse>(() =>
  props.parsed ?? parseResponse(props.response),
)
</script>

<template>
  <div class="max-h-[70vh] overflow-auto">
    <template v-if="resolvedParsed.type === 'table'">
      <p v-if="resolvedParsed.header" class="mb-2 text-sm font-medium text-muted-foreground">{{ resolvedParsed.header }}</p>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead v-for="col in resolvedParsed.columns" :key="col">{{ col }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(row, i) in resolvedParsed.rows" :key="i">
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
          <dd class="font-mono truncate">{{ entry.value }}</dd>
        </template>
      </dl>
    </template>

    <template v-else>
      <pre class="whitespace-pre-wrap text-sm rounded-md bg-muted p-4">{{ resolvedParsed.text }}</pre>
    </template>
  </div>
</template>
