<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Template</h2>
        <ProgressBar class="py-5" :start_step=3 :items="templateStore.template_stepper" />
        <div class="mb-auto w-full grid grid-flow-col-dense auto-cols-min gap-x-5">
            <div class="flex flex-col gap-y-5">
                <PropCard class="min-w-60 w-fit h-fit" :data="templateData" />
                <IntervalTable class="w-fit h-fit" :columns="intervalColumn" :data="intervalData" />
            </div>
            <TemplateStationCreationTable class="w-fit h-fit" :columns="stationColumn" :view_only="true" />
        </div>
        <div class="flex-row justify-between">
            <Button @click="navigate('/templates/new/stations')">
                <iconify-icon icon="lucide:chevron-left" width="24" height="24"></iconify-icon>
                Previous
            </Button>
            <Button @click="pushCreateTemplate">
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
import { useTemplateCreationStore } from "~/stores/templateCreation";
import { useAuthStore } from "~/stores/auth";
import { columns as stationColumn } from "~/components/template-station-creation-table/columns-view.ts";
import { columns as intervalColumn } from "~/components/interval-table/columns.ts";
import { toast } from "vue-sonner";
import TemplateStationCreationTable from "~/components/template-station-creation-table/TemplateStationCreationTable.vue";

const router = useRouter();
const templateStore = useTemplateCreationStore();
const authStore = useAuthStore();
const alert_open = ref(false);
const loading = ref(false);

const intervalData = computed(() => {
    return [
        { name: "Intermission", duration: templateStore.payload.template_session.intermission_duration },
        { name: "Feedback", duration: templateStore.payload.template_session.feedback ? templateStore.payload.template_session.feedback_duration : false },
        { name: "Static At End", duration: templateStore.payload.template_session.static_at_end ? templateStore.payload.template_stations[templateStore.payload.template_stations.length - 1].duration : false },
    ];
});

const templateData = computed(() => {
    return [
        { title: "Template Name:", value: templateStore.payload.template_session.name },
        { title: "Organisation", value: authStore.organisation },
    ];
});

const pushCreateTemplate = async () => {
    if (loading.value) { return; }
    loading.value = true;
    const response = await templateStore.pushTemplate();
    loading.value = false;
    if (response) {
        navigateTo("/");
    }
}

const navigate = (path: string) => {
    return navigateTo(path);
};

const return_home = () => {
    templateStore.resetpayload();
    return navigateTo("/");
};

const cancel = () => {
    if (templateStore.isDirty) {
        alert_open.value = true;
    } else {
        return return_home();
    }
};

onBeforeMount(async () => {
    window.onbeforeunload = () => {
        if (templateStore.isDirty) {
            return "You have unsaved changes. Are you sure you want to leave?";
        }
    };
});

onUnmounted(() => {
    window.onbeforeunload = null;
    if (!router.currentRoute.value.path.startsWith("/sessions/new")) {
        templateStore.resetpayload();
    }
});
</script>