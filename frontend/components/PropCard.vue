<template>
    <div class="rounded-md p-4 bg-(--background-2)">
        <div v-for="row in data" class="flex-row gap-2 justify-between text-xl">
            <div class="subhead">{{ row.title + ":" }}</div>
            <div class="text font-medium!">{{ row.value }}</div>
        </div>
    </div>
</template>

<script setup lang="ts">
interface PropCards {
    data: { title: string, value: string, }[];
}
const props = defineProps<PropCards>()
</script>