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
        <div class="flex-column self-center max-w-md w-[20rem] my-auto border-1 rounded-md border-(border-muted) gap-6 p-5">
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
                <Input id="int-duration" type="number" v-ref="intDuration" placeholder="Duration (seconds)"/>
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
                    <Input id="feed-duration" type="number" v-ref="feedDuration" placeholder="Duration (seconds)"/>
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

const session_templates = ref([]);
const sessionStore = useSessionCreationStore();
const router = useRouter();
const alert_open = ref(false);

const intDuration = ref<number | null>(null);
const feedDuration = ref<number | null>(null);

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