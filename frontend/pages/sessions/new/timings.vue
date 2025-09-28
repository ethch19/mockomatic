<template>
    <div class="flex-column py-[1rem] px-[3rem] text h-full">
        <h2 class="subtitle">Create Session</h2>
        <div>
            <Stepper>
                <StepperItem :step=1>
                    <StepperTitle>Session Details</StepperTitle>
                    <StepperSeparator />
                </StepperItem>
                <StepperItem :step=2>
                    <StepperTitle>Stations</StepperTitle>
                    <StepperSeparator />
                </StepperItem>
                <StepperItem :step=3>
                    <StepperTitle>Timings</StepperTitle>
                    <StepperSeparator />
                </StepperItem>
                <StepperItem :step=4>
                    <StepperTitle>Summary</StepperTitle>
                    <StepperSeparator />
                </StepperItem>
            </Stepper>
        </div>
        <!-- <StationCreationTable
            class="self-center max-w-md w-[30rem] my-auto"
            :columns="columns"
            v-model:data="stations"
            @update-station="handleUpdateStation"
        /> -->
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
import { columns } from "~/components/station-creation-table/columns";
import type { SlotPayload } from "~/utils/types";

const sessionStore = useSessionCreationStore();
const router = useRouter();
const loading = ref(false);
const { payload } = storeToRefs(sessionStore);
const slots = computed(() => payload.value.slots);
const alert_open = ref(false);

const navigate = (path: string) => {
    return navigateTo(path);
};

// function handleUpdateStation(rowIndex: number, columnId: string, value: any) {
//     sessionStore.updateStation(rowIndex, columnId, value);
//     // https://github.com/TanStack/table/pull/5687#issuecomment-2281067245
//     // data is shadowRef, must mutate full data
// }

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