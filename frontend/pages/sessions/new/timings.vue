<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Session</h2>
        <ProgressBar class="py-5" :start_step=2 :items="sessionStore.session_stepper" />
        <SlotCreationTable
            class="self-center max-w-md w-120 my-auto" :columns="slotColumns" :sub_columns="runColumns"
        />
        <div class="flex-row justify-between">
            <Button @click="navigate('/sessions/new/stations')">
                <iconify-icon icon="lucide:chevron-left" width="24" height="24"></iconify-icon>
                Previous
            </Button>
            <Button @click="navigate('/sessions/new/review')">
                Next
                <iconify-icon icon="lucide:chevron-right" width="24" height="24"></iconify-icon>
            </Button>
        </div>
        <AlertDialog v-model:open="alert_open">
            <AlertDialogContent>
            <AlertDialogHeader>
                <AlertDialogTitle>You have unsaved changes</AlertDialogTitle>
                <AlertDialogDescription>
                This action cannot be undone. Are you sure you want to cancel and lose progress?
                </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction>Continue</AlertDialogAction>
            </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    </div>
</template>

<script lang="ts" setup>
import { useSessionCreationStore } from "~/stores/sessionCreation";
import SlotCreationTable from "~/components/slot-creation-table/SlotCreationTable.vue"
import { columns as slotColumns } from "~/components/slot-creation-table/columns.ts";
import { columns as runColumns } from "~/components/slot-creation-table/columns-runs.ts";

const router = useRouter();
const sessionStore = useSessionCreationStore();
const alert_open = ref(false);

const navigate = (path: string) => {
    return navigateTo(path);
};

function handleUpdateRun(slotIndex: number, runIndex: number, columnId: string, new_duration: TimeDuration) {
    sessionStore.onRunTimeChanged(slotIndex, runIndex, columnId, new_duration);
}

provide("update-runs", handleUpdateRun);

const return_home = () => {
    sessionStore.resetpayload();
    return navigateTo("/");
};

const cancel = () => {
    if (sessionStore.isDirty) {
        alert_open.value = true;
    } else {
        return return_home();
    }
};

onBeforeMount(async () => {
    window.onbeforeunload = () => {
        if (sessionStore.isDirty) {
            return "You have unsaved changes. Are you sure you want to leave?";
        }
    };
    if (!sessionStore.fetchedTemplate) {
        await sessionStore.fetchTemplates();
    }
});

onUnmounted(() => {
    window.onbeforeunload = null;
    if (!router.currentRoute.value.path.startsWith("/sessions/new")) {
        sessionStore.resetpayload();
    }
});
</script>