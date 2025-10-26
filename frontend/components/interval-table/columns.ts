import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import { formatInterval } from '@/composables/formatting';
import type { ISessionInterval, PgInterval } from '@/utils/types';

export const columns: ColumnDef<ISessionInterval>[] = [
    {
        accessorKey: "name",
        header: ({ column }) => {
            return h('div', { class: "text-center p-2" }, "Intervals")
        },
        cell: ({ row }) => {
            const name = row.getValue("name");
            return h('div', { class: 'text-center font-medium px-2' }, name)
        },
    },
    {
        accessorKey: "duration",
        header: () => h('div', { class: 'text-center p-2' }, "Duration (MM:SS)"),
        cell: ({ row, table}) => {
            const duration: PgInterval | false = row.getValue("duration");
            const duration_formatted = duration != false ? formatInterval(duration) : 'False';
            return h('div', { class: 'text-center font-medium' }, duration_formatted);
        },
    },
]