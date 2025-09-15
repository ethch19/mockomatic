<template>
  <div class="wizard-container text">
    <div class="main-container flex-column">
      <h2>Create New Session</h2>
      <Card class="session-form">
        <template #title>Step 5: Review</template>
        <template #content>
          <div class="form-section">
            <h3>Configuration</h3>
            <p><strong>Organisation:</strong> {{ sessionStore.form.session.organisation }}</p>
            <p><strong>Scheduled Date:</strong> {{ formatDate(sessionStore.form.session.scheduled_date) }}</p>
            <p><strong>Location:</strong> {{ sessionStore.form.session.location }}</p>
            <p v-if="!sessionStore.form.session.feedback"><strong>Feedback:</strong> {{ sessionStore.form.session.feedback ? "Yes" : "No" }}</p>
            <p v-if="sessionStore.form.session.feedback"><strong>Feedback Duration:</strong> {{ formatInterval(sessionStore.form.session.feedback_duration) }}</p>
            <p><strong>Intermission Duration:</strong> {{ formatInterval(sessionStore.form.session.intermission_duration) }}</p>
            <p><strong>Static at End:</strong> {{ sessionStore.form.session.static_at_end ? "Yes" : "No" }}</p>
            <br>
            <h3>Stations</h3>
            <DataTable :value="sessionStore.stationsMinutes" :scrollable="true">
              <Column field="title" header="Title" />
              <Column field="duration" header="Duration (min)" />
            </DataTable>
            <br>
            <h3>Slots</h3>
            <template v-for="cur_slot in sessionStore.form.slots" :key="cur_slot.key">
              <h4>{{ "Slot " + cur_slot.key }}</h4>
              <DataTable :value="cur_slot.runs" :scrollable="true">
                <Column field="scheduled_start" header="Scheduled Start">
                  <template #body="{ data }">
                    {{ data.scheduled_start }}
                  </template>
                </Column>
                <Column field="scheduled_end" header="Scheduled End">
                  <template #body="{ data }">
                    {{ data.scheduled_end }}
                  </template>
                </Column>
                <Column field="flip_allocation" header="Flip Allocation">
                  <template #body="{ data }">
                    {{ data.flip_allocation ? "Yes" : "No" }}
                  </template>
                </Column>
              </DataTable>
              <DataTable :value="cur_slot.circuits" :scrollable="true">
                <template #header>
                  <h4>Runs</h4>
                </template>
                <Column field="key" header="Key">
                  <template #body="{ data }">
                    {{ data.key }}
                  </template>
                </Column>
                <Column field="female_only" header="Female Only">
                  <template #body="{ data }">
                    {{ data.female_only ? "Yes" : "No" }}
                  </template>
                </Column>
              </DataTable>
            </template>
          </div>
          <div class="wizard-actions">
            <Button label="Previous" icon="pi pi-arrow-left" class="p-button-secondary" @click="previousStep" />
            <Button label="Submit" icon="pi pi-check" class="p-button-primary" @click="submitForm" />
            <Button label="Cancel" icon="pi pi-times" class="p-button-secondary p-button-text" @click="cancel" />
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useSessionCreationStore } from "~/stores/sessionCreation";
import { apiFetch } from "~~/composables/apiFetch";

definePageMeta({
  layout: "default",
});

const sessionStore = useSessionCreationStore();
const router = useRouter();

const prepareFormData = () => {
  const scheduledDate = sessionStore.form.session.scheduled_date;
  return {
    ...sessionStore.form,
    session: {
      ...sessionStore.form.session,
      scheduled_date: formatDateForBackend(scheduledDate),
    },
    slots: sessionStore.form.slots.map(slot => ({
      ...slot,
      runs: slot.runs.map(run => ({
        scheduled_start: formatTimeForBackend(scheduledDate, run.scheduled_start),
        scheduled_end: formatTimeForBackend(scheduledDate, run.scheduled_end),
        flip_allocation: run.flip_allocation,
      })),
    })),
  };
};

const submitForm = async () => {
  try {
    console.log(prepareFormData());
    const response = await apiFetch("/sessions/create", {
      method: "POST",
      body: prepareFormData(),
    });
    sessionStore.resetForm();
    console.log(response.id);
    router.push(`/sessions/${response.id}`);
  } catch (error) {
    console.error("Submit error:", error);
  }
};

const formatDate = (date) => {
  return date ? new Date(date).toLocaleDateString() : "N/A";
};

const formatInterval = (interval) => {
  if (!interval || typeof interval !== "object") return "N/A";
  const minutes = Math.floor(interval.microseconds / (1_000_000 * 60));
  const seconds = Math.floor((interval.microseconds % (1_000_000 * 60)) / 1_000_000);
  return `${minutes}:${seconds.toString().padStart(2, "0")} min`;
};

// Convert HH:mm to ISO 8601 with scheduled_date
const formatTimeForBackend = (date: Date | null, time: string | null): string | null => {
  if (!date || !time) return null;
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const [hours, minutes] = time.split(":");
  // Format as ISO 8601 with UTC (Z)
  return `${year}-${month}-${day}T${hours.padStart(2, "0")}:${minutes.padStart(2, "0")}:00Z`;
};

// Format Date object to YYYY-MM-DD
const formatDateForBackend = (date: Date | null): string | null => {
  if (!date) return null;
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0"); // Months are 0-indexed
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
};

const previousStep = () => {
  sessionStore.step = 4;
  router.push("/sessions/new/circuits");
};

const cancel = () => {
  if (sessionStore.isDirty) {
    if (confirm("You have unsaved changes. Are you sure you want to cancel and lose progress?")) {
      sessionStore.resetForm();
      router.push("/");
    }
  } else {
    sessionStore.resetForm();
    router.push("/");
  }
};

onBeforeMount(() => {
  window.onbeforeunload = () => {
    if (sessionStore.isDirty) {
      return "You have unsaved changes. Are you sure you want to leave?";
    }
  };
});

onUnmounted(() => {
  window.onbeforeunload = null;
  if (!router.currentRoute.value.path.startsWith("/sessions/new")) {
    sessionStore.resetForm();
  }
});
</script>

<style scoped>
.wizard-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 2rem;
  width: 100%;
}

.main-container {
  width: 50%;
}

.session-form {
  width: 100%;
}

.form-section {
  margin-bottom: 2rem;
}

.wizard-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}
</style>