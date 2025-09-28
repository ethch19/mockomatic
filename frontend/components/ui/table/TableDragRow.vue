<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"
import { watchEffect, render, h } from "vue"
import { draggable, dropTargetForElements, monitorForElements } from '@atlaskit/pragmatic-drag-and-drop/element/adapter'
import { type Instruction, attachInstruction, extractInstruction } from '@atlaskit/pragmatic-drag-and-drop-hitbox/list-item'
import { pointerOutsideOfPreview } from '@atlaskit/pragmatic-drag-and-drop/element/pointer-outside-of-preview'
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview'
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine'
import { unrefElement } from '@vueuse/core'

const elRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const instruction = ref<Extract<Instruction, { type: 'reorder-before' | 'reorder-after' }> | null>(null)

const props = defineProps<{
    class?: HTMLAttributes["class"]
    item: { [key: string]: any };
    idRef: string;
}>()

watchEffect((onCleanup) => {
    const currentElement = unrefElement(elRef)
    if (!currentElement) return;


    console.log(props.idRef);
    const rowData = { ...props.item };
    const rowId: string = rowData[props.idRef];

    console.log("Row ID:", rowId);
    console.log("Row Data:", rowData);

    const dragHandleElement = currentElement.querySelector<HTMLElement>('[data-drag-handle="true"]');
    if (!dragHandleElement) {
        return;
    } else {
        console.log('Drag handle found');
        console.log(dragHandleElement);
    };

    const dndFunction = combine(
        draggable({
            element: currentElement,
            dragHandle: dragHandleElement,
            getInitialData: () => ({ id: rowId, item: rowData }),
            onDragStart: () => {
                isDragging.value = true
            },
            onDrop: () => {
                isDragging.value = false
            },
            onGenerateDragPreview({ nativeSetDragImage }) {
                setCustomNativeDragPreview({
                    getOffset: pointerOutsideOfPreview({ x: '16px', y: '8px' }),
                    render: ({ container }) => {
                        return render(h(
                            'div',
                            { class: 'bg-(--foreground) text-(--background) rounded-md text-sm font-medium px-3 py-1.5' },
                            "Row " + rowId
                        ), container)
                    },
                    nativeSetDragImage,
                })
            }
        }),
        dropTargetForElements({
            element: currentElement,
            getData: ({ input, element }) => {
                const data = { id: rowId, item: rowData };
                return attachInstruction(data, {
                    input,
                    element,
                    axis: 'vertical',
                    operations: {
                        'reorder-before': 'available',
                        'reorder-after': 'available',
                        'combine': 'not-available',
                    }
                });
            },
            canDrop: ({ source }) => {
                return source.data.id !== rowId
            },
            onDrag: ({ self }) => {
                instruction.value = extractInstruction(self.data) as typeof instruction.value
            },
            onDragLeave: () => {
                instruction.value = null
            },
            onDrop: ({ location }) => {
                instruction.value = null
            }
        }),
        monitorForElements({
            canMonitor: ({ source }) => {
                return source.data.id !== rowId;
            },
        }),
    );

    // Cleanup dnd function
    onCleanup(() => dndFunction())
})
</script>

<template>
    <tr
        ref="elRef"
        data-slot="table-row"
        :class="cn('relative hover:bg-muted/50 data-[state=selected]:bg-muted border-b transition-colors', { 'opacity-50': isDragging }, props.class)"
    >
        <slot />
        <!-- <div class="absolute border-t-2 border-(--secondary) top-0 left-0 right-0"></div> -->
        <div v-if="instruction"
        :class="cn('absolute h-full left-0 right-0 bottom-0 top-0 border-(--secondary)', 
        { 
            '!border-t-2': instruction?.operation === 'reorder-before',
            '!border-b-2': instruction?.operation === 'reorder-after',
        })" />
    </tr>
</template>
