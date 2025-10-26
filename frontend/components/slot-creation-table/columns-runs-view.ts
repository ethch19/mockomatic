import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import "iconify-icon";
import type { IRunPayload } from '@/utils/types';
import { Time } from '@internationalized/date';

export const columns: ColumnDef<IRunPayload>[] = [
    {
        id: "run_index",
        cell: ({ row, table }) => {
            let text: string = String(row.index + 1);
            let flip: boolean = row.original.flip_allocation;
            const first = text[0];
            if (text.length == 2 && text[1] == "1") {
                text += "th"
            } else {
                switch (first) {
                    case "1":
                        text += "st"
                        break
                    case "2":
                        text += "nd"
                        break
                    case "3":
                        text += "rd"
                        break
                    default:
                        text += "th"
                }
            }
            return h("div", { class: "flex h-full items-center justify-start w-15" },
                h("div", { class: 'text-center text-sm flex flex-row justify-center items-center gap-1 relative' },
                    [
                        h('iconify-icon', { icon: "lucide:corner-down-right", width: "24", height: "24", class: "mb-1"}),
                        text,
                        flip
                            ? h('iconify-icon', { icon: "lucide:arrow-left-right", width: "18", height: "18", class: "absolute top-0 -right-4"})
                            : null
                    ]
            ));
        },
    },
    {
        accessorKey: "scheduled_start",
        cell: ({ row, table }) => {
            const time: Time = row.getValue("scheduled_start");
            const time_str = time != null ? `${time}` : "~";
            return h('div', { class: 'text-center' }, time_str);
        },
    },
    {
        accessorKey: "scheduled_end",
        cell: ({ row, table }) => {
            const time: Time = row.getValue("scheduled_end");
            const time_str = time != null ? `${time}` : "~";
            return h('div', { class: 'text-center' }, time_str);
        },
    },
]