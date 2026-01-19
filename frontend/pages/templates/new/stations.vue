<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Template</h2>
        <ProgressBar class="py-5" :start_step=1 :items="templateStore.template_stepper" />
        <TemplateStationCreationTable
            class="self-center max-w-md w-120 my-auto"
            :columns="columns"
            @update-template-station="handleUpdateStation"
        />
        <div class="flex-row justify-between">
            <Button @click="navigate('/templates/new')">
                <iconify-icon icon="lucide:chevron-left" width="24" height="24"></iconify-icon>
                Previous
            </Button>
            <Button @click="navigate('/templates/new/review')">
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
import { useTemplateCreationStore } from "~/stores/templateCreation";
import { toast } from "vue-sonner";
import TemplateStationCreationTable from "~/components/template-station-creation-table/TemplateStationCreationTable.vue";
import { columns } from "~/components/template-station-creation-table/columns";

const templateStore = useTemplateCreationStore();
const loading = ref(false);
const alert_open = ref(false);
const router = useRouter();

const navigate = (path: string) => {
    const valid = templateStore.validateStations();
    if (valid == true){
        return navigateTo(path);
    } else {
        toast.error(valid);
    }
};

function handleUpdateStation(rowIndex: number, columnId: string, value: any) {
    templateStore.updateStation(rowIndex, columnId, value);
    // https://github.com/TanStack/table/pull/5687#issuecomment-2281067245
    // data is shadowRef, must mutate full data
}

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
    if (!router.currentRoute.value.path.startsWith("/templates/new")) {
        templateStore.resetpayload();
    }
});
</script>