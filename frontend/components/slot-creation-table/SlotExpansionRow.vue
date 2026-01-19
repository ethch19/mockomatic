<template>
    <TableRow
        :data-state="row.getIsSelected() ? 'selected' : undefined"
        :item="row.original as TData"
    >
        <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
            <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
        </TableCell>
    </TableRow>
    <tr v-if="row.getIsExpanded()">
        <template v-if="!view_only">
            <td></td>
        </template>
        <td class="p-2 align-middle whitespace-nowrap" :colspan="row.getAllCells().length">
            <div :class="{ 'ml-8': view_only }">
                <div>
                    <Table>
                        <TableBody>
                            <template v-if="table.getRowModel().rows?.length">
                                <TableRow
                                    v-for="runrow in table.getRowModel().rows"
                                    :key="runrow.id"
                                >
                                    <TableCell v-for="cell in runrow.getVisibleCells()" :key="cell.id">
                                        <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                                    </TableCell>
                                </TableRow>
                            </template>
                            <template v-else>
                                <TableRow>
                                    <TableCell :colspan="columns.length" class="h-24 text-center">
                                    No runs.
                                    </TableCell>
                                </TableRow>
                            </template>
                            <td></td>
                        </TableBody>
                    </Table>
                </div>
                <div class="flex flex-col justify-center p-2">
                    <template v-if="!view_only">
                        <div class="flex flex-row justify-start items-end gap-2">
                            <h4 class="font-semibold text-sm">Circuits</h4>
                            <p class="font-normal text-xs text-(--gray-200)">Toggle = Female Only circuit</p>
                        </div>
                        <div class="space-y-4 py-3 flex flex-row justify-between">
                            <CircuitToggle v-for="circuit in circuits_formatted" :row_id="row.id" :circuit="circuit" @update="circuitChanged" @delete="deleteCircuit" />
                            <Button
                                class="h-full mx-1"
                                size="sm"
                                @click="addCircuit"
                            >
                                <iconify-icon icon="lucide:plus" width="24" height="24"></iconify-icon>
                            </Button>
                        </div>
                    </template>
                    <template v-else>
                        <h4 class="font-semibold text-sm">Circuits</h4>
                        <div class="py-3 flex flex-row justify-start gap-5">
                            <div v-for="circuit in circuits_formatted" class="flex-row gap-2">
                                <Label>{{ circuit.key }}</Label>
                                <CustomBadge :name="circuit.female_only ? 'Female' : 'Mixed'" :border_colour="circuit.female_only ? 'stage-400' : 'stage-200'" />
                            </div>
                        </div>
                    </template>
                </div>
            </div>
        </td>
    </tr>
</template>

<script setup lang="ts" generic="TData">
import type { ColumnDef, Row } from '@tanstack/vue-table'
import { TableRow, TableCell, TableBody, Table } from '@/components/ui/table'
import {
  FlexRender,
  getCoreRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import { valueUpdater } from '@/lib/utils'
import { useSessionCreationStore } from '~/stores/sessionCreation';
import { toast } from "vue-sonner";
import CircuitToggle from '../CircuitToggle.vue';
import type { TimeDuration } from '@internationalized/date';

interface DataRowProps {
    row: Row<TData>;
    slot_index: number;
    columns: ColumnDef<IRunPayload>;
    view_only: boolean;
}
const props = defineProps<DataRowProps>();

const handleUpdateRun = inject("update-runs");
const sessionStore = useSessionCreationStore();
const { payload } = storeToRefs(sessionStore);

const table = useVueTable({
    get data() { return sessionStore.payload.slots[props.slot_index].runs },
    get columns() { return props.columns },
    getCoreRowModel: getCoreRowModel(),
    meta: {
        updateData: (rowIndex: number, columnId: string, value: TimeDuration) => {
            handleUpdateRun(props.slot_index, rowIndex, columnId, value);
        },
        slot_index: props.slot_index,
    },
})

const circuits_formatted = computed(() => {
    const circuits = payload.value.slots[props.slot_index].circuits;
    return circuits.map((circuit, index) => {
        let index_cal: number = index + 1;
        let cir_key = "";
        while (index_cal > 0) {
            const remainder = (index_cal - 1) % 26;
            cir_key = String.fromCharCode(remainder + "A".charCodeAt(0)) + cir_key;
            index_cal = Math.floor((index_cal - 1) / 26)
        }
        return {
            ...circuit,
            key: cir_key
        };
    });
});

const circuitChanged = (value: boolean, circuit_key: string) => {
    const index = circuits_formatted.value.findIndex((x) => x.key == circuit_key);
    if (index != -1) {
        sessionStore.toggleCircuit(props.slot_index, index);
    }
}

const deleteCircuit = (circuit_key: string) => {
    if (circuits_formatted.value.length == 1) {
        toast.error("Slots must have at least 1 circuit!");
        return;
    }
    const index = circuits_formatted.value.findIndex((x) => x.key == circuit_key);
    if (index != -1) {
        sessionStore.removeCircuit(props.slot_index, index);
    }
}

const addCircuit = () => {
    sessionStore.addCircuit(props.slot_index);
    toast.success("Circuit added");
};
</script>