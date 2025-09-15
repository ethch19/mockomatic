<template>
    <DropdownMenu>
        <DropdownMenuTrigger as-child>
        <Button variant="ghost" class="w-8 h-8 p-0">
            <span class="sr-only">Open menu</span>
            <iconify-icon icon="lucide:more-horizontal" width="24" height="24"></iconify-icon>
        </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
            <DropdownMenuLabel>Actions</DropdownMenuLabel>
            <DropdownMenuItem @click="copy(session.id)">
                Copy session ID
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem>View customer</DropdownMenuItem>
            <DropdownMenuItem>View payment details</DropdownMenuItem>
        </DropdownMenuContent>
    </DropdownMenu>
</template>

<script setup lang="ts">
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'

defineProps<{
    session: {
        id: string
    }
}>()

function copy(id: string) {
    navigator.clipboard.writeText(id)
}
</script>