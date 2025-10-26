<template>
    <div class="flex-column py-4 px-12 text h-full">
        <h2 class="subtitle">Create Session</h2>
        <ProgressBar class="py-5" :start_step=0 :items="sessionStore.session_stepper" />
        <div class="flex-column self-center max-w-md w-[20rem] my-auto border rounded-md border-(border-muted) gap-6 p-5">
            <div class="flex-column gap-2">
                <Label for="scheduled-date">Scheduled Date:</Label>
                <Popover>
                    <PopoverTrigger as-child>
                        <Button
                            variant="outline"
                            :class="cn(
                            'justify-start text-left font-normal',
                            !value && 'text-muted-foreground',
                            )"
                        >
                            <iconify-icon icon="lucide:calendar" width="24" height="24"></iconify-icon>
                            {{ sessionStore.payload.session.scheduled_date ? df.format(sessionStore.payload.session.scheduled_date.toDate(getLocalTimeZone())) : "Pick a date" }}
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent class="w-auto p-0">
                        <Calendar v-model="sessionStore.payload.session.scheduled_date" initial-focus />
                    </PopoverContent>
                </Popover>
            </div>
            <div class="flex-column gap-2">
                <Label for="int-duration">Intermission Duration:</Label>
                <Input id="int-duration" placeholder="Duration (MM:SS)" @keydown.enter="intermissionEnter" @blur="intermissionChanged" v-model="intermission_duration"></Input>
            </div>
            <div class="flex-column gap-2">
                <Label for="location">Location:</Label>
                <Input id="location" type="text" v-model="sessionStore.payload.session.location"/>
            </div>
            <div class="flex-row justify-between">
                <div class="flex-column gap-2">
                    <Label for="feedback">Feedback:</Label>
                    <Switch id="feedback" v-model="sessionStore.payload.session.feedback"/>
                </div>
                <div class="flex-column gap-2" v-if="sessionStore.payload.session.feedback">
                    <Label for="feed-duration">Feedback Duration:</Label>
                    <Input id="feed-duration" placeholder="Duration (MM:SS)" @keydown.enter="feedbackEnter" @blur="feedbackChanged" v-model="feedback_duration"></Input>
                </div>
            </div>
            <div class="flex-row justify-between">
                <div class="flex-column gap-2">
                    <Label for="feedback">Static At End:</Label>
                    <Switch id="feedback" v-model="sessionStore.payload.session.static_at_end"/>
                </div>
            </div>
        </div>
        <div class="flex-row justify-end">
            <Button @click="navigate('/sessions/new/stations')">
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
import { apiFetch } from "~~/composables/apiFetch"
import { toast } from "vue-sonner";
import { DateFormatter, getLocalTimeZone } from "@internationalized/date"
import { cn } from "~/lib/utils"
import { formatInterval, normalizeMinuteSeconds, formatIntervalFromString } from "~/composables/formatting";
import ProgressBar from "~/components/ProgressBar.vue";

const session_templates = ref([]);
const sessionStore = useSessionCreationStore();
const { payload } = storeToRefs(sessionStore);
const router = useRouter();
const alert_open = ref(false);

const intermission_formatted = computed(() => {
    return formatInterval(payload.value.session.intermission_duration);
})
const intermission_duration = ref(intermission_formatted.value);

const feedback_formatted = computed(() => {
    return formatInterval(payload.value.session.feedback_duration);
})
const feedback_duration = ref(feedback_formatted.value);

function intermissionChanged() {
    const new_value = normalizeMinuteSeconds(intermission_duration.value);
    if (new_value) {
        const new_duration = formatIntervalFromString(new_value);
        sessionStore.payload.session.intermission_duration = new_duration;
        intermission_duration.value = new_value;
    } else {
        intermission_duration.value = formatInterval(sessionStore.payload.session.intermission_duration);
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
        sessionStore.payload.session.feedback_duration = new_duration;
        feedback_duration.value = new_value;
    } else {
        feedback_duration.value = formatInterval(sessionStore.payload.session.feedback_duration);
    }
}

function feedbackEnter(event: KeyboardEvent) {
    feedbackChanged()
    event.target.blur()
}

const df = new DateFormatter("en-GB", {
    dateStyle: "full",
});

const templateSelected = (event) => {
    sessionStore.applyTemplate(event);
    // toast.success("Template applied");
};

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