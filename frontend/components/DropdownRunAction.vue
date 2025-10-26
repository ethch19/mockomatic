<template>
    <DropdownMenu>
        <DropdownMenuTrigger as-child>
            <Button class="p-2" variant="ghost">
                <iconify-icon icon="lucide:more-horizontal" width="24" height="24"></iconify-icon>
            </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
            <DropdownMenuItem class="text" :as-child="true" @select="(event) => event.preventDefault">
                <Toggle v-model="sessionStore.payload.slots[props.slot_index].runs[props.run_index].flip_allocation">
                    <iconify-icon class="text-foreground" icon="lucide:arrow-left-right" width="24" height="24"></iconify-icon>
                    Flip Allocation
                </Toggle>
            </DropdownMenuItem>
            <DropdownMenuItem class="text" @select="triggerDelete">
                <iconify-icon class="text-foreground" icon="lucide:trash-2" width="24" height="24"></iconify-icon>
                Delete
            </DropdownMenuItem>
        </DropdownMenuContent>
    </DropdownMenu>
</template>

<script setup lang="ts">
import "iconify-icon"
import { toast } from "vue-sonner";
import { useSessionCreationStore } from '~/stores/sessionCreation';

const sessionStore = useSessionCreationStore();

interface RunActionProps {
    slot_index: number
    run_index: number
}
const props = defineProps<RunActionProps>();

const triggerDelete = () => {
    if (sessionStore.payload.slots[props.slot_index].runs.length == 1) {
        toast.error("Each slot must have at least 1 run!")
        return;
    }
    sessionStore.deleteRun(props.slot_index, props.run_index);
}
</script>