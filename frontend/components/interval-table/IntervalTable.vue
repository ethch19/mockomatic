<template>
    <div class="border rounded-md">
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
            <TableDragBody @reorder="reorderStation">
                <template v-if="table.getRowModel().rows?.length">
                    <TableRow
                        v-for="row in table.getRowModel().rows" :key="row.id"
                    >
                        <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
                            <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                        </TableCell>
                    </TableRow>
                </template>
                <template v-else>
                    <TableRow>
                        <TableCell :colspan="columns.length" class="h-24 text-center">
                        No intervals.
                        </TableCell>
                    </TableRow>
                </template>
            </TableDragBody>
        </Table>
    </div>
</template>

<script setup lang="ts">
import { Table, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import type { ColumnDef } from '@tanstack/vue-table'
import {
  FlexRender,
  getCoreRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import { valueUpdater } from '@/lib/utils'
import type { ISessionInterval } from '~/utils/types';

interface StationProps {
    columns: ColumnDef<ISessionInterval>[];
    data: ISessionInterval[];
}
const props = defineProps<StationProps>()

const table = useVueTable({
    get data() { return props.data },
    get columns() { return props.columns },
    getRowId: row => row.name,
    getCoreRowModel: getCoreRowModel(),
})
</script>