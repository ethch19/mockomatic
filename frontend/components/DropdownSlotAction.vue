<template>
    <DropdownMenu>
        <DropdownMenuTrigger as-child>
            <Button class="p-2" variant="ghost">
                <iconify-icon icon="lucide:more-horizontal" width="24" height="24"></iconify-icon>
            </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
            <DropdownMenuLabel>{{ "Slot " + slot_key }}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem class="text" @select="sessionStore.addRun(slot_index)">
                <iconify-icon class="text-foreground" icon="lucide:plus" width="24" height="24"></iconify-icon>
                Add Run
            </DropdownMenuItem>
            <DropdownMenuItem class="text" @select="sessionStore.deleteSlot(slot_index);">
                <iconify-icon class="text-foreground" icon="lucide:trash-2" width="24" height="24"></iconify-icon>
                Delete
            </DropdownMenuItem>
        </DropdownMenuContent>
    </DropdownMenu>
</template>

<script setup lang="ts">
import "iconify-icon"
import { useSessionCreationStore } from '~/stores/sessionCreation';

const sessionStore = useSessionCreationStore();

interface SlotActionProps {
    slot_key: string;
    slot_index: number;
}
const props = defineProps<SlotActionProps>();
</script>