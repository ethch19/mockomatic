import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { formatInterval, formatIntervalFromString, normalizeMinuteSeconds } from '@/composables/formatting';
import "iconify-icon";
import type { IStationPayload, PgInterval } from '@/utils/types';

export const columns: ColumnDef<IStationPayload>[] = [
    {
        id: "drag",
        header: () => h('div', { class: "w-10" }),
        cell: () => {
             return h(Button, {
                variant: "ghost",
                "data-drag-handle": true,
                class: "w-10",
            }, () => [h('iconify-icon', { icon: "lucide:grip-vertical", width: "24", height: "24" })])
        },
    },
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
        accessorKey: "index",
        header: ({ column }) => {
            return h(Button, {
                variant: "ghost",
                onClick: () => column.toggleSorting(column.getIsSorted() === "asc"),
            }, () => ["Order", h('iconify-icon', { class: "ml-2", icon: "lucide:arrow-up-down", width: "24", height: "24" })])
        },
        cell: ({ row }) => {
            const index = row.getValue("index");
            return h('div', { class: 'text-center font-medium' }, index)
        },
    },
    {
        accessorKey: "title",
        header: () => h('div', { class: 'text-left' }, "Title"),
        cell: ({ row, table }) => {
            const title = row.getValue("title");
            return h(Input, {
                type: 'text',
                class: 'text-left font-medium',
                modelValue: title,
                'onUpdate:modelValue': (value) => {
                    table.options.meta?.updateData?.(row.index, 'title', value);
                },
            });
        },
    },
    {
        accessorKey: "duration",
        header: () => h('div', { class: 'text-left' }, "Duration (MM:SS)"),
        cell: ({ row, table}) => {
            const duration: PgInterval = row.getValue("duration");
            const duration_formatted = formatInterval(duration);
            return h(Input, {
                class: 'text-left font-medium',
                modelValue: duration_formatted,
                onBlur: (event) => {
                    const value = (event.target as HTMLInputElement).value;
                    const new_value = normalizeMinuteSeconds(value);
                    if (new_value) {
                        const new_duration = formatIntervalFromString(new_value);
                        table.options.meta?.updateData?.(row.index, 'duration', new_duration);
                    } else {
                        table.options.meta?.updateData?.(row.index, 'duration', duration);
                    }
                },
                onKeydown: (event: KeyboardEvent) => {
                    if (event.key != "Enter") return;
                    const value = (event.target as HTMLInputElement).value;
                    const new_value = normalizeMinuteSeconds(value);
                    if (new_value) {
                        const new_duration = formatIntervalFromString(new_value);
                        table.options.meta?.updateData?.(row.index, 'duration', new_duration);
                    } else {
                        table.options.meta?.updateData?.(row.index, 'duration', duration);
                    }
                    (event.target as HTMLInputElement).blur()
                },
            });
        },
    },
]