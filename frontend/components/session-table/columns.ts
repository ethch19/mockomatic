import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import SessionTableOption from './SessionTableOption.vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import CustomBadge from '../CustomBadge.vue';
import { formatInterval } from '@/composables/formatting';
import type { ISession } from '@/utils/types';
import { CalendarDate, DateFormatter, toLocalTimeZone, type ZonedDateTime } from '@internationalized/date';
import "iconify-icon";

export const columns: ColumnDef<ISession>[] = [
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
        accessorKey: "created_at",
        header: ({ column }) => {
            return h("div", { class: "flex h-full items-center justify-center" },
                h(Button,{
                    variant: "ghost",
                    onClick: () => column.toggleSorting(column.getIsSorted() === "asc"),
                }, () => ["Created At", h('iconify-icon', { class: "ml-2", icon: "lucide:arrow-up-down", width: "24", height: "24" })]
            ));
        },
        cell: ({ row }) => {
            const df = new DateFormatter("en-GB", { dateStyle: "short", timeStyle: "long", });
            const created_at: ZonedDateTime = row.getValue("created_at");
            return h('div', { class: 'text-center font-medium' }, df.format(created_at.toDate()))
        },
    },
    {
        accessorKey: "scheduled_date",
        header: () => h('div', { class: 'text-center' }, "Date"),
        cell: ({ row }) => {
            const df = new DateFormatter("en-GB", {dateStyle: "short" });
            const scheduled_date: CalendarDate = row.getValue("scheduled_date");
            return h('div', { class: 'text-center font-medium' }, df.format(scheduled_date.toDate()))
        },
    },
    {
        accessorKey: "location",
        header: () => h('div', { class: 'text-center' }, "Location"),
        cell: ({ row }) => {
            return h('div', { class: 'text-center font-medium' }, row.getValue("location"))
        },
    },
    {
        accessorKey: "total_stations",
        header: () => h('div', { class: 'text-center' }, "Total Stations"),
        cell: ({ row }) => {
            return h('div', { class: 'text-center font-medium' }, row.getValue("total_stations"))
        },
    },
    {
        accessorKey: "intermission_duration",
        header: () => h('div', { class: 'text-center' }, "Intermission (min:sec)"),
        cell: ({ row }) => {
            const intermission_duration = formatInterval(row.getValue("intermission_duration"));
            return h('div', { class: 'text-center font-medium' }, intermission_duration)
        },
    },
    {
        accessorKey: "feedback",
        header: () => h('div', { class: 'text-center' }, "Feedback (min:sec)"),
        cell: ({ row }) => {
            const feedback_duration = row.getValue("feedback") ? formatInterval(row.original.feedback_duration) : "N/A";
            return h('div', { class: 'text-center font-medium' }, feedback_duration)
        },
    },
    {
        accessorKey: "static_at_end",
        header: () => h('div', { class: 'text-center' }, "Static at End"),
        cell: ({ row }) => {
            const staticAtEnd = row.getValue("static_at_end") ? "Yes" : "No";
            return h('div', { class: 'text-center font-medium' }, staticAtEnd)
        },
    },
    {
        accessorKey: "status",
        header: () => h('div', { class: 'text-center' }, "Status"),
        cell: ({ row }) => {
            const status_colours = {
                new: "stage-100",
                prep: "stage-200",
                ready: "stage-300",
                pending: "stage-600",
                running: "stage-500",
                completed: "stage-300",
            };
            const status: string = row.getValue("status");
            const upper_status = status.slice(0, 1).toUpperCase().concat(status.slice(1));
            return h('div', { class: "flex justify-center items-center" },
                h(CustomBadge, { class: "w-fit px-3 justify-center", name: upper_status, border_colour: status_colours[status] })
            );
        },
    },
    {
        id: 'actions',
        enableHiding: false,
        cell: ({ row }) => {
            const session = row.original;
            return h('div', { class: 'relative' },
                h(SessionTableOption, { session })
            )
        },
    },
]
            
