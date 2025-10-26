<template>
    <ContextMenu>
        <ContextMenuTrigger class="flex items-center justify-between space-x-2">
                <Label :for="`${row_id}-cir-${circuit.key}`">{{ circuit.key }}</Label>
                <Switch
                    :id="`${row_id}-cir-tog-${circuit.key}`"
                    :model-value="circuit.female_only"
                    @update:model-value="(payload) => emit('update', payload, circuit.key)"
                />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuLabel>
                {{ "Circuit " + circuit.key }}
            </ContextMenuLabel>
            <ContextMenuItem @select="emit('delete', circuit.key)">
                Delete
            </ContextMenuItem>
        </ContextMenuContent>
    </ContextMenu>
</template>

<script setup lang="ts">
import { boolean, string } from 'zod';


interface PropTypes {
    row_id: string;
    circuit: { key: string, female_only: boolean };
}
const props = defineProps<PropTypes>();

const emit = defineEmits<{
    (e: "update", payload: boolean, key: string): void
    (e: "delete", key: string): void
}>();
</script>