<template>
    <Label class="border-3 h-6 px-1.5 rounded-full text text-xs!" :style="{ borderColor : 'var(--' + border_colour + ')' }">{{ name }}</Label>
</template>

<script setup lang="ts">
interface BadgeProps {
    name: string;
    border_colour: string;
}
const props = defineProps<BadgeProps>();
</script>