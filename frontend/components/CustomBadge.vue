<template>
    <Label class="border-3 h-6 px-1.5 rounded-full text text-xs!" :style="{ borderColor : border_color }">{{ name }}</Label>
</template>

<script setup lang="ts">
interface BadgeProps {
    name: string;
    border_color: string;
}
const props = defineProps<BadgeProps>();
</script>