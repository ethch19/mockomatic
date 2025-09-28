<template>
  <div class="flex-column gap-[1rem]">
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
            <TableDragBody @reorder="reorderStation">
                <template v-if="table.getRowModel().rows?.length">
                    <TableDragRow
                        v-for="row in table.getRowModel().rows" :key="row.id"
                        :data-state="row.getIsSelected() ? 'selected' : undefined"
                        :item="row.original as TData"
                        id-ref="index"
                    >
                        <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
                            <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                        </TableCell>
                    </TableDragRow>
                </template>
                <template v-else>
                    <TableRow>
                        <TableCell :colspan="columns.length" class="h-24 text-center">
                        No stations.
                        </TableCell>
                    </TableRow>
                </template>
            </TableDragBody>
        </Table>
    </div>
    <div class="flex-row justify-between items-center py-1 h-[3rem]">
        <div class="text-sm text-(--text-2)">
            {{ table.getSelectedRowModel().rows.length }} of
            {{ table.getRowModel().rows.length }} station(s) selected.
        </div>
        <div class="flex-row justify-end gap-[0.5rem] h-full">
            <Button
                class="h-full text-(--text-2)"
                variant="destructive"
                size="sm"
                :disabled="table.getSelectedRowModel().rows.length < 1"
                @click="deleteSelected"
            >
                Delete Selected
            </Button>
            <Button
                class="h-full text-(--text-2)"
                variant="outline"
                size="sm"
                @click="addStation"
            >
                Add Station
            </Button>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts" generic="TData, TValue">
import { Table, TableCell, TableHead, TableHeader, TableRow, TableDragRow, TableDragBody } from '@/components/ui/table'
import type { ColumnDef, SortingState } from '@tanstack/vue-table'
import {
  FlexRender,
  getCoreRowModel,
  getSortedRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import { valueUpdater } from '@/lib/utils'
import { useSessionCreationStore } from '~/stores/sessionCreation';
import { toast } from "vue-sonner";
import TableCaption from '../ui/table/TableCaption.vue';

const sessionStore = useSessionCreationStore();

const props = defineProps<{
    columns: ColumnDef<TData, TValue>[]
}>()
const data = defineModel<TData[]>('data', { required: true })
const emit = defineEmits(["update-station"]);

const rowSelection = ref({})
const sorting = ref<SortingState>([])

const table = useVueTable({
    get data() { return data.value },
    get columns() { return props.columns },
    getRowId: row => row.index,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: updaterOrValue => valueUpdater(updaterOrValue, sorting),
    onRowSelectionChange: updaterOrValue => valueUpdater(updaterOrValue, rowSelection),
    state: {
        get sorting() { return sorting.value },
        get rowSelection() { return rowSelection.value },
    },
    meta: {
        updateData: (rowIndex: number, columnId: string, value: any) => {
            emit('update-station', rowIndex, columnId, value);
        },
    },
})

const addStation = () => {
    sessionStore.addStation();
    toast.success("Station added");
};

const deleteSelected = () => {
    console.log("Delete selected stations", table.getSelectedRowModel().rows);
    // remove all selection after deletion
    sessionStore.deleteSelectedStations(table.getSelectedRowModel().rows);
};

const reorderStation = ({ rowId, targetRowId, instruction }) => {
    sessionStore.reorderStation(rowId, targetRowId, instruction);
};
</script>