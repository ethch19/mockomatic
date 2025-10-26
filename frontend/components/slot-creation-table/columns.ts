import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox'
import "iconify-icon";
import type { ISlotPayload, RunPayload } from '@/utils/types';
import DropdownSlotAction from '../DropdownSlotAction.vue';

export const columns: ColumnDef<ISlotPayload>[] = [
    {
        id: "select",
        header: ({ table }) => h(Checkbox, {
            "modelValue": table.getIsAllPageRowsSelected(),
            "onUpdate:modelValue": (value: boolean) => table.toggleAllPageRowsSelected(!!value),
            "ariaLabel": "Select all",
        }),
        cell: ({ row }) => h(Checkbox, {
            "modelValue": row.getIsSelected(),
            "onUpdate:modelValue": (value: boolean) => row.toggleSelected(!!value),
            "ariaLabel": "Select row",
        }),
        enableSorting: false,
        enableHiding: false,
    },
    {
        accessorKey: "key",
        header: ({ column }) => {
            return h("div", { class: "flex h-full items-center justify-start" },
                    h(Button, {
                    variant: "ghost",
                    onClick: () => column.toggleSorting(column.getIsSorted() === "asc"),
                }, () => ["Slots", h('iconify-icon', { class: "ml-2", icon: "lucide:arrow-up-down", width: "24", height: "24" })]
            )
        )
        },
        cell: ({ row }) => {
            const key = row.getValue("key");
            return h('div', { class: 'text-left font-medium mx-10' }, key)
        },
    },
    {
        id: "start_time",
        header: () => h("div", { class: "text-left" }, "Start Time"),
        cell: ({ row }) => {
            const runs: RunPayload[] = row.original.runs;
            const time_str = runs[0].scheduled_start != null ? `${runs[0].scheduled_start}` : "~";
            return h('div', { class: 'text-left font-medium' }, time_str)
        },
    },
    {
        id: "end_time",
        header: () => h("div", { class: "text-left" }, "End Time"),
        cell: ({ row }) => {
            const runs: RunPayload[] = row.original.runs;
            const time_str = runs[runs.length-1].scheduled_end != null ? `${runs[runs.length-1].scheduled_end}` : "0";
            return h('div', { class: 'text-left font-medium' }, time_str)
        },
    },
    {
        id: "actions",
        cell: ({ row }) => {
            const key = row.getValue("key");
            return h(DropdownSlotAction, { slot_key: key, slot_index: row.index })
        }
    }
]