<template>
    <Stepper v-model="model" :default-value="start_step" class="gap-0">
        <StepperItem v-for="(item, index) in items" :step=index class="w-60 gap-0">
            <StepperIndicator class="flex flex-row w-full gap-2 rounded-none opacity-100!" :class="{ 'bg-primary! text-primary-foreground!': index<=start_step, 'rounded-l-lg': index==0, 'bg-muted! text-muted-foreground!': index>start_step, 'rounded-r-lg': index==items.length-1}">
                <iconify-icon :icon=item.icon width="24" height="24"></iconify-icon>
                {{ item.title }}
            </StepperIndicator>
            <StepperSeparator
                v-if="index !== items.length -1"
                class="w-10 h-full relative rounded-none opacity-100!"
                :class="{ 'arrow': index==start_step, 'bg-primary!': index<=start_step, 'bg-muted!': index > start_step }"
            />
        </StepperItem>
    </Stepper>
</template>

<script setup lang="ts">
import "iconify-icon"

interface BarProps {
    start_step: number;
    items: [{ title: string, icon: string }];
}
const props = defineProps<BarProps>();

const model = defineModel<number>();
</script>

<style scoped>
.arrow::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 100%;
  aspect-ratio: 1 / 1;
  height: 70%;
  background-color: var(--primary);
  transform: translateY(-50%) translateX(-50%) rotate(45deg);
  z-index: 0;
  border-top-right-radius: 8px;
  border-bottom-left-radius: 8px;
}
</style>