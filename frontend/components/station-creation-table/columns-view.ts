import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import { Button } from '@/components/ui/button';
import { formatInterval } from '@/composables/formatting';
import "iconify-icon";
import type { IStationPayload, PgInterval } from '@/utils/types';

export const columns: ColumnDef<IStationPayload>[] = [
    {
        accessorKey: "index",
        header: ({ column }) => {
            return h(Button, {
                variant: "ghost",
                onClick: () => column.toggleSorting(column.getIsSorted() === "asc"),
            }, () => ["#", h('iconify-icon', { class: "ml-2", icon: "lucide:arrow-up-down", width: "24", height: "24" })])
        },
        cell: ({ row }) => {
            const index = row.getValue("index");
            return h('div', { class: 'text-center font-medium' }, index)
        },
    },
    {
        accessorKey: "title",
        header: () => h('div', { class: 'text-left' }, "Stations"),
        cell: ({ row, table }) => {
            const title = row.getValue("title");
            return h('div', { class: 'text-left font-medium' }, title);
        },
    },
    {
        accessorKey: "duration",
        header: () => h('div', { class: 'text-left' }, "Duration (MM:SS)"),
        cell: ({ row, table}) => {
            const duration: PgInterval = row.getValue("duration");
            const duration_formatted = formatInterval(duration);
            return h('div', { class: 'text-left font-medium' }, duration_formatted);
        },
    },
]