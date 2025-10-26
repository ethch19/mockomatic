<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Session</h2>
        <ProgressBar class="py-5" :start_step=1 :items="sessionStore.session_stepper" />
        <StationCreationTable
            class="self-center max-w-md w-120 my-auto"
            :columns="columns"
            @update-station="handleUpdateStation"
        />
        <div class="flex-row justify-between">
            <Button @click="navigate('/sessions/new')">
                <iconify-icon icon="lucide:chevron-left" width="24" height="24"></iconify-icon>
                Previous
            </Button>
            <Button @click="navigate('/sessions/new/timings')">
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
import { columns } from "~/components/station-creation-table/columns";
import StationCreationTable from "~/components/station-creation-table/StationCreationTable.vue";
import type { StationPayload } from "~/utils/types";
import { toast } from "vue-sonner";

const sessionStore = useSessionCreationStore();
const router = useRouter();
const loading = ref(false);
const alert_open = ref(false);

const navigate = (path: string) => {
    const valid = sessionStore.validateStations();
    if (valid == true){
        sessionStore.recalculateTimings();
        return navigateTo(path);
    } else {
        toast.error(valid);
    }
};

function handleUpdateStation(rowIndex: number, columnId: string, value: any) {
    sessionStore.updateStation(rowIndex, columnId, value);
    // https://github.com/TanStack/table/pull/5687#issuecomment-2281067245
    // data is shadowRef, must mutate full data
}

const templateSelected = (event) => {
    sessionStore.applyTemplate(event);
    toast.success("Template applied");
};

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