<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Session</h2>
        <ProgressBar class="py-5" :start_step=3 :items="sessionStore.session_stepper" />
        <div class="mb-auto w-full grid grid-flow-col-dense auto-cols-min gap-x-5">
            <div class="flex flex-col gap-y-5">
                <PropCard class="min-w-60 w-fit h-fit" :data="sessionData" />
                <IntervalTable class="w-fit h-fit" :columns="intervalColumn" :data="intervalData" />
            </div>
            <StationCreationTable class="w-fit h-fit" :columns="stationColumn" :view_only="true" />
            <SlotCreationTable class="w-fit h-fit" :columns="slotColumn" :sub_columns="runColumn" :view_only="true" />
        </div>
        <div class="flex-row justify-between">
            <Button @click="navigate('/sessions/new/timings')">
                <iconify-icon icon="lucide:chevron-left" width="24" height="24"></iconify-icon>
                Previous
            </Button>
            <Button @click="pushCreateSession">
                Submit
                <iconify-icon v-show="!loading" icon="lucide:send-horizontal" width="24" height="24"></iconify-icon>
                <iconify-icon v-show="loading" icon="svg-spinners:180-ring" width="24" height="24" ></iconify-icon>
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
import { useAuthStore } from "~/stores/auth";
import { DateFormatter, getLocalTimeZone } from "@internationalized/date";
import { columns as slotColumn } from "~/components/slot-creation-table/columns-view.ts";
import { columns as runColumn } from "~/components/slot-creation-table/columns-runs-view.ts";
import { columns as stationColumn } from "~/components/station-creation-table/columns-view.ts";
import { columns as intervalColumn } from "~/components/interval-table/columns.ts";
import { toast } from "vue-sonner";

const router = useRouter();
const sessionStore = useSessionCreationStore();
const authStore = useAuthStore();
const alert_open = ref(false);
const loading = ref(false);

const df = new DateFormatter("en-UK", {
    dateStyle: "long",
})

const intervalData = computed(() => {
    return [
        { name: "Intermission", duration: sessionStore.payload.session.intermission_duration },
        { name: "Feedback", duration: sessionStore.payload.session.feedback ? sessionStore.payload.session.feedback_duration : false },
        { name: "Static At End", duration: sessionStore.payload.session.static_at_end ? sessionStore.payload.stations[sessionStore.payload.stations.length - 1] : false },
    ];
});

const sessionData = computed(() => {
    return [
        { title: "Date", value: sessionStore.payload.session.scheduled_date != null ? df.format(sessionStore.payload.session.scheduled_date.toDate(getLocalTimeZone())) : "None" },
        { title: "Location", value: sessionStore.payload.session.location },
        { title: "Organisation", value: authStore.organisation },
    ];
});

const pushCreateSession = async () => {
    if (loading.value) { return; }
    loading.value = true;
    const response = await sessionStore.pushSession();
    loading.value = false;
    if (response) {
        navigateTo("/");
    }
}

const navigate = (path: string) => {
    return navigateTo(path);
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