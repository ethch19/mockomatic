<template>
  <div class="flex-column gap-4">
    <div class="border rounded-md h-full">
        <Table>
            <TableHeader>
                <TableRow v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
                <TableHead v-for="header in headerGroup.headers" :key="header.id">
                    <FlexRender
                    v-if="!header.isPlaceholder" :render="header.column.columnDef.header"
                    :props="header.getContext()"
                    />
                </TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                <template v-if="table.getRowModel().rows?.length">
                    <TableRow
                        v-for="row in table.getRowModel().rows" :key="row.id"
                        :data-state="row.getIsSelected() ? 'selected' : undefined"
                    >
                        <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
                            <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                        </TableCell>
                    </TableRow>
                </template>
                <template v-else>
                    <TableRow>
                        <TableCell :colspan="columns.length" class="h-24 text-center">
                        No results.
                        </TableCell>
                    </TableRow>
                </template>
            </TableBody>
        </Table>
    </div>
    <div class="flex-row justify-between items-center px-4 py-1 h-12">
        <div class="text-sm text-(--text-2)">
            {{ table.getFilteredSelectedRowModel().rows.length }} of
            {{ table.getFilteredRowModel().rows.length }} session(s) selected.
        </div>
        <div class="flex-row justify-end gap-2 h-full">
            <Button
                class="h-full text-(--text-2)"
                variant="outline"
                size="sm"
                :disabled="!table.getCanPreviousPage()"
                @click="table.previousPage()"
            >
                Previous
            </Button>
            <Button
                class="h-full text-(--text-2)"
                variant="outline"
                size="sm"
                :disabled="!table.getCanNextPage()"
                @click="table.nextPage()"
            >
                Next
            </Button>
            <Input class="h-full w-min min-w-10 max-w-14" type="page" :disabled="!(table.getPageCount() > 1)" placeholder="1" v-model="currentPage" ref="pageInput" @keydown="pagechange"/>
            <div class="flex items-center space-x-2 h-full">
                <p class="text-sm font-medium text-(--text-2)">
                Sessions per page
                </p>
                <Select
                    :model-value="`${table.getState().pagination.pageSize}`"
                    @update:model-value="table.setPageSize"
                    class="h-full"
                >
                    <SelectTrigger class="data-[size=default]:h-full">
                        <SelectValue :placeholder="`${table.getState().pagination.pageSize}`" />
                    </SelectTrigger>
                    <SelectContent side="top">
                        <SelectItem v-for="pageSize in [10, 25, 50]" :key="pageSize" :value="`${pageSize}`">
                            {{ pageSize }}
                        </SelectItem>
                    </SelectContent>
                </Select>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts" generic="TData, TValue">
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { ColumnDef, SortingState } from '@tanstack/vue-table'
import {
  FlexRender,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import { valueUpdater } from '@/lib/utils'

const props = defineProps<{
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
}>()

const sorting = ref<SortingState>([])
const rowSelection = ref({})
const currentPage = defineModel<string>("1")
const pageInput = ref<HTMLInputElement | null>(null)

const pagechange = (event: KeyboardEvent) => {
  if (event.key === "Enter") {
    const pageIndex = Number(currentPage.value) - 1;
    if (!isNaN(pageIndex) && pageIndex >= 0 && pageIndex < table.getPageCount()) {
        table.setPageIndex(pageIndex)
        currentPage.value = String(pageIndex + 1)
    } else {
        currentPage.value = String(table.getState().pagination.pageIndex + 1)
    }
    pageInput.value?.blur()
  }
}

const table = useVueTable({
    get data() { return props.data },
    get columns() { return props.columns },
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: updaterOrValue => valueUpdater(updaterOrValue, sorting),
    onRowSelectionChange: updaterOrValue => valueUpdater(updaterOrValue, rowSelection),
    state: {
        get sorting() { return sorting.value },
        get rowSelection() { return rowSelection.value },
    },
})
</script>