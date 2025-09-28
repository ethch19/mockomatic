<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { watchEffect } from "vue"
import { cn } from "@/lib/utils"
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine'
import { monitorForElements } from '@atlaskit/pragmatic-drag-and-drop/element/adapter'
import { type Instruction, extractInstruction } from '@atlaskit/pragmatic-drag-and-drop-hitbox/list-item'

const props = defineProps<{
  class?: HTMLAttributes["class"]
}>()

const emit = defineEmits<{
  (e: "reorder", { rowId, targetRowId, instruction }: { 
      rowId: number, 
      targetRowId: number, 
      instruction: Instruction['operation']
  }): void
}>();

watchEffect((onCleanup) => {
    const dndFunction = combine(
        monitorForElements({
            onDrop(args) {
                const { location, source } = args
                // didn't drop on anything
                if (!location.current.dropTargets.length)
                return

                const itemId: number = source.data.id
                const target = location.current.dropTargets[0]
                const targetId: number = target.data.id

                const instruction: Instruction | null = extractInstruction(
                    target.data,
                )

                if (instruction !== null) {
                    // update data here
                    console.log("Dropped item", itemId, "on target", targetId, "to", instruction.operation);
                    emit('reorder', {
                        rowId: itemId,
                        targetRowId: targetId,
                        instruction: instruction.operation.toString(),
                    });
                }
            },
        }),
    )

    onCleanup(() => {
        dndFunction()
    })
})
</script>

<template>
  <tbody
    data-slot="table-body"
    :class="cn('[&_tr:last-child]:border-0', props.class)"
  >
    <slot />
  </tbody>
</template>