<template>
    <div class="flex-column justify-start py-4 px-12 text h-full">
        <div class="flex-row justify-between items-center mb-4">
            <h1 class="subtitle">Sessions</h1>
            <div class="flex-row gap-4">
                <Button @click="navigate('/sessions/new/')">
                    <iconify-icon class="text-primary-foreground" icon="lucide:plus" width="24" height="24"></iconify-icon>
                    New Session
                </Button>
                <Button @click="navigate('/templates/new/')" variant="secondary">
                    <iconify-icon class="text-secondary-foreground" icon="lucide:grid-2x2-plus" width="24" height="24"></iconify-icon>
                    New Template
                </Button>
                <Button @click="navigate('/templates/')" variant="outline">View Templates</Button>
            </div>
        </div>
        <SessionTable class="h-full" :columns="columns" :data="sessionBrowserStore.sessions" :loading="loading"/>
    </div>
</template>

<script lang="ts" setup>
import type { ISession } from "~/utils/types";
import { columns } from "~/components/session-table/columns.ts";
import { toast } from "vue-sonner";
import { useSessionBrowserStore } from "~/stores/sessionBrowser";

const sessionBrowserStore = useSessionBrowserStore();
const loading = ref(false);

const navigate = (path: string) => {
    return navigateTo(path);
};

onMounted(async () => {
    if (loading.value) return;
    loading.value = true;
    try {
        await sessionBrowserStore.fetchAll();
    } catch (err) {
        console.log(err);
    }
    loading.value = false;
});
</script>