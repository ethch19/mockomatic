import { h } from 'vue';
import type { ColumnDef } from '@tanstack/vue-table';
import SessionTableOption from './SessionTableOption.vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox'
import { formatInterval, formatDate } from '@/composables/formatting';
import "iconify-icon";

export interface Session {
    id: string;
    organiser_id: string;
    organisation: string;
    scheduled_date: string;
    location: string;
    total_stations: number;
    feedback: boolean;
    feedback_duration: PgInterval;
    intermission_duration: PgInterval;
    static_at_end: boolean;
    status: "new" | "prep" | "ready" | "pending" | "running" | "completed";
    created_at: string;
};

export const columns: ColumnDef<Session>[] = [
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
            return h(Button, {
                variant: "ghost",
                onClick: () => column.toggleSorting(column.getIsSorted() === "asc"),
            }, () => ["Created At", h('iconify-icon', { class: "ml-2", icon: "lucide:arrow-up-down", width: "24", height: "24" })])
        },
        cell: ({ row }) => {
            const created_at = formatDate(row.getValue("created_at"));

            return h('div', { class: 'text-right font-medium' }, created_at)
        },
    },
    {
        accessorKey: "scheduled_date",
        header: () => h('div', { class: 'text-right' }, "Date"),
        cell: ({ row }) => {
            const scheduled_date = formatDate(row.getValue("scheduled_date"));
            return h('div', { class: 'text-right font-medium' }, scheduled_date)
        },
    },
    {
        id: "location",
        header: () => h('div', { class: 'text-right' }, "Location"),
        cell: ({ row }) => {
            return h('div', { class: 'text-right font-medium' }, row.getValue("location"))
        },
    },
    {
        id: "total_stations",
        header: () => h('div', { class: 'text-right' }, "Total Stations"),
        cell: ({ row }) => {
            return h('div', { class: 'text-right font-medium' }, row.getValue("total_stations"))
        },
    },
    {
        id: "intermission_duration",
        header: () => h('div', { class: 'text-right' }, "Intermission"),
        cell: ({ row }) => {
            const intermission_duration = formatInterval(row.getValue("intermission_duration"));
            const intermission = intermission_duration != 0 ? `${intermission_duration} minutes` : "N/A";
            return h('div', { class: 'text-right font-medium' }, intermission)
        },
    },
    {
        id: "feedback",
        header: () => h('div', { class: 'text-right' }, "Feedback"),
        cell: ({ row }) => {
            const feedback_duration = row.getValue("feedback") == true ? formatInterval(row.getValue("feedback_duration")) : "N/A";
            return h('div', { class: 'text-right font-medium' }, feedback_duration)
        },
    },
    {
        id: "static_at_end",
        header: () => h('div', { class: 'text-right' }, "Static at End"),
        cell: ({ row }) => {
            const staticAtEnd = row.getValue("static_at_end") ? "Yes" : "No";
            return h('div', { class: 'text-right font-medium' }, staticAtEnd)
        },
    },
    {
        accessorKey: "status",
        header: () => h('div', { class: 'text-right' }, "Status"),
        cell: ({ row }) => {
            // add icons
            return h('div', { class: 'text-right font-medium' }, row.getValue("status"))
        },
    },
    {
        id: 'actions',
        enableHiding: false,
        cell: ({ row }) => {
            const session = row.original
            return h('div', { class: 'relative' }, h(SessionTableOption, {
                session,
            }))
        },
    },
]
            
