<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Template</h2>
        <ProgressBar class="py-5" :start_step=0 :items="templateStore.template_stepper" />
        <div class="flex-column self-center max-w-md w-[20rem] my-auto border rounded-md border-(border-muted) gap-6 p-5">
            <div class="flex-column gap-2">
                <Label for="template_name">Template Name:</Label>
                <Input id="template_name" type="text" v-model="templateStore.payload.template_session.name"/>
            </div>
            <div class="flex-column gap-2">
                <Label for="int-duration">Intermission Duration:</Label>
                <Input id="int-duration" placeholder="Duration (MM:SS)" @keydown.enter="intermissionEnter" @blur="intermissionChanged" v-model="intermission_duration"></Input>
            </div>
            <div class="flex-row justify-between">
                <div class="flex-column gap-2">
                    <Label for="feedback">Feedback:</Label>
                    <Switch id="feedback" v-model="templateStore.payload.template_session.feedback"/>
                </div>
                <div class="flex-column gap-2" v-if="templateStore.payload.template_session.feedback">
                    <Label for="feed-duration">Feedback Duration:</Label>
                    <Input id="feed-duration" placeholder="Duration (MM:SS)" @keydown.enter="feedbackEnter" @blur="feedbackChanged" v-model="feedback_duration"></Input>
                </div>
            </div>
            <div class="flex-row justify-between">
                <div class="flex-column gap-2">
                    <Label for="feedback">Static At End:</Label>
                    <Switch id="feedback" v-model="templateStore.payload.template_session.static_at_end"/>
                </div>
            </div>
        </div>
        <div class="flex-row justify-end">
            <Button @click="navigate('/templates/new/stations')">
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
import { DateFormatter, getLocalTimeZone } from "@internationalized/date"
import { cn } from "~/lib/utils"
import { formatInterval, normalizeMinuteSeconds, formatIntervalFromString } from "~/composables/formatting";
import ProgressBar from "~/components/ProgressBar.vue";

const templateStore = useTemplateCreationStore();
const { payload } = storeToRefs(templateStore);
const alert_open = ref(false);
const router = useRouter();

const intermission_formatted = computed(() => {
    return formatInterval(payload.value.template_session.intermission_duration);
})
const intermission_duration = ref(intermission_formatted.value);

const feedback_formatted = computed(() => {
    return formatInterval(payload.value.template_session.feedback_duration);
})
const feedback_duration = ref(feedback_formatted.value);

function intermissionChanged() {
    const new_value = normalizeMinuteSeconds(intermission_duration.value);
    if (new_value) {
        const new_duration = formatIntervalFromString(new_value);
        templateStore.payload.template_session.intermission_duration = new_duration;
        intermission_duration.value = new_value;
    } else {
        intermission_duration.value = formatInterval(templateStore.payload.template_session.intermission_duration);
    }
}

function intermissionEnter(event: KeyboardEvent) {
    intermissionChanged()
    event.target.blur()
}

function feedbackChanged() {
    const new_value = normalizeMinuteSeconds(feedback_duration.value);
    if (new_value) {
        const new_duration = formatIntervalFromString(new_value);
        templateStore.payload.template_session.feedback_duration = new_duration;
        feedback_duration.value = new_value;
    } else {
        feedback_duration.value = formatInterval(templateStore.payload.template_session.feedback_duration);
    }
}

function feedbackEnter(event: KeyboardEvent) {
    feedbackChanged()
    event.target.blur()
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
    if (!router.currentRoute.value.path.startsWith("/templates/new")) {
        templateStore.resetpayload();
    }
});
</script>