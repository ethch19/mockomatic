<template>
    <div class="flex-column justify-start py-[1rem] px-[3rem] text h-full">
        <div class="flex-row justify-between items-center mb-[1rem]">
            <h1 class="subtitle">Sessions</h1>
            <div class="flex-row gap-[1rem]">
                <Button @click="navigate('/sessions/new/')">
                    <iconify-icon class="text-(--primary-foreground)" icon="lucide:plus" width="24" height="24"></iconify-icon>
                    New Session
                </Button>
                <Button @click="navigate('/templates/new/')" variant="secondary">
                    <iconify-icon class="text-(--secondary-foreground)" icon="lucide:grid-2x2-plus" width="24" height="24"></iconify-icon>
                    New Template
                </Button>
                <Button @click="navigate('/templates/')" variant="outline">View Templates</Button>
            </div>
        </div>
        <SessionTable class="h-full" :columns="columns" :data="data" />
    </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~~/composables/apiFetch"
import type { ISession } from "~/utils/types";
import { columns } from "~/components/session-table/columns";
import SessionTable from "~/components/session-table/SessionTable.vue";
import { toast } from "vue-sonner";

const data = ref<ISession[]>([]);
const loading = ref(false);
const router = useRouter();

const loadSessions: Promise<ISession[]> = async () => {
    if (loading.value) return;
    loading.value = true;
    try {
        const response = await apiFetch("/sessions/get-all");
        data.value = response;
    } catch (err) {
        toast.error("Failed to get sessions: ", err.data);
    } finally {
        loading.value = false;
    }
};

const navigate = (path: string) => {
    return navigateTo(path);
};

onMounted(async () => {
    await loadSessions();
});
</script>