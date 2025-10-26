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
            <TableDragBody @reorder="reorderStation">
                <template v-if="table.getRowModel().rows?.length">
                    <SlotExpansionRow
                        v-for="row in table.getRowModel().rows"
                        :key="row.id"
                        :row="row"
                        :slot_index="row.index"
                        :columns="sub_columns"
                        :view_only="view_only"
                    />
                </template>
                <template v-else>
                    <TableRow>
                        <TableCell :colspan="columns.length" class="h-24 text-center">
                        No Slots.
                        </TableCell>
                    </TableRow>
                </template>
            </TableDragBody>
        </Table>
    </div>
    <template v-if="!view_only">
        <div class="flex-row justify-between items-center py-1 h-12">
            <div class="text-sm text-(--text-2)">
                {{ table.getSelectedRowModel().rows.length }} of
                {{ table.getRowModel().rows.length }} slots(s) selected.
            </div>
            <div class="flex-row justify-end gap-2 h-full">
                <Button
                    class="h-full text-(--text-2)"
                    variant="outline"
                    size="sm"
                    @click="addSlot"
                >
                    Add Slot
                </Button>
            </div>
        </div>
    </template>
  </div>
</template>

<script setup lang="ts" generic="TData, TValue">
import { Table, TableCell, TableHead, TableHeader, TableRow, TableDragBody } from '@/components/ui/table'
import SlotExpansionRow from "~/components/slot-creation-table/SlotExpansionRow.vue"
import type { ColumnDef, SortingState, ExpandedTableState, ExpandedState } from '@tanstack/vue-table'
import {
  FlexRender,
  getCoreRowModel,
  getExpandedRowModel,
  getSortedRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import { valueUpdater } from '@/lib/utils'
import { useSessionCreationStore } from '~/stores/sessionCreation';
import { toast } from "vue-sonner";
import type { PropType, Ref } from 'vue';

interface SlotCreationProps {
    columns: ColumnDef<ISlotPayload>;
    sub_columns: ColumnDef<IRunPayload>;
    view_only?: boolean;
}
const { columns, sub_columns, view_only = false } = defineProps<SlotCreationProps>();

const sessionStore = useSessionCreationStore();
const { payload } = storeToRefs(sessionStore);

const rowSelection = ref({})
const sorting = ref<SortingState>([])
const expandedTable: ExpandedTableState = { expanded: true };

const slots_formatted = computed(() => {
    const slots = payload.value.slots;
    return slots.map((slot, index) => {
        let index_cal: number = index + 1;
        let slot_key = "";
        while (index_cal > 0) {
            const remainder = (index_cal - 1) % 26;
            slot_key = String.fromCharCode(remainder + "A".charCodeAt(0)) + slot_key;
            index_cal = Math.floor((index_cal - 1) / 26)
        }
        return {
            ...slot,
            key: slot_key
        };
    });
});

const table = useVueTable({
    get data() { return slots_formatted.value },
    get columns() { return columns },
    getRowId: row => row.key,
    getRowCanExpand: row => true,
    initialState: expandedTable,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getExpandedRowModel: getExpandedRowModel(),
    onSortingChange: updaterOrValue => valueUpdater(updaterOrValue, sorting),
    onRowSelectionChange: updaterOrValue => valueUpdater(updaterOrValue, rowSelection),
    state: {
        get sorting() { return sorting.value },
        get rowSelection() { return rowSelection.value },
    },
})

const addSlot = () => {
    sessionStore.addSlot();
    toast.success("Slot added");
};
</script>