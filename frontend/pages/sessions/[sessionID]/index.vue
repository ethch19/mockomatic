<template>
  <div class="session-container text">
    <div>
      <h1>Progress</h1>
      <Card class="timeline-card">
        <template #content>
          <Timeline :value="session_progress" align="right" class="session-timeline">
            <template #marker="slotProps">
              <span class="flex-row timeline-marker">
                  <i v-if="slotProps.item.completed" class="pi pi-check-circle"></i>
                  <i v-else class="pi pi-circle"></i>
              </span>
            </template>
            <template #content="slotProps">
              {{ slotProps.item.topic }}
            </template>
          </Timeline>
        </template>
      </Card>
    </div>
    <div class="right-column">
      <div class="flex-column">
        <h1>Session Details</h1>
        <Card>
          <template #content>
            <div v-if="sessionStore.loading" class="loading">
              <ProgressSpinner />
            </div>
            <div v-else-if="sessionStore.error" class="error">
              <Message severity="error" :closable="false">
                {{ sessionStore.error }}
              </Message>
            </div>
            <div v-else class="session-details">
              <p><strong>ID:</strong> {{ sessionId }}</p>
              <p><strong>Organisation:</strong> {{ sessionStore.session?.organisation }}</p>
              <p><strong>Location:</strong> {{ sessionStore.session?.location }}</p>
              <p><strong>Scheduled Date:</strong> {{ formatDate(sessionStore.session?.scheduled_date) }}</p>
              <p><strong>Total Stations:</strong> {{ sessionStore.session?.total_stations }}</p>
              <p><strong>Intermission Duration:</strong> {{ formatInterval(sessionStore.session?.intermission_duration) }}</p>
              <p v-if="sessionStore.session?.feedback"><strong>Feedback Duration:</strong> {{ formatInterval(sessionStore.session?.feedback_duration) }}</p>
              <p><strong>Created At:</strong> {{ formatDate(sessionStore.session?.created_at) }}</p>
            </div>
          </template>
        </Card>
        <h1>Stations</h1>
        <Card>
          <template #content>
            <div v-if="sessionStore.loading" class="loading">
              <ProgressSpinner />
            </div>
            <div v-else-if="sessionStore.error" class="error">
              <Message severity="error" :closable="false">
                {{ sessionStore.error }}
              </Message>
            </div>
            <div v-else class="session-stations">
                <DataTable :value="sessionStore.stations" :scrollable="true">
                    <Column field="title" header="Title"/>
                    <Column field="duration" header="Duration (min)">
                        <template #body="{ data }">
                            {{ formatInterval(data.duration) }}
                        </template>
                    </Column>
                </DataTable>
            </div>
          </template>
        </Card>
        <br/>
        <SelectButton
            v-model="slot_selection"
            :options="slot_options"
            optionLabel="label"
            optionValue="value"
            :multiple="false"
            :allowEmpty="false"
        />
        <Card>
          <template #content>
            <div v-if="sessionStore.loading" class="loading">
              <ProgressSpinner />
            </div>
            <div v-else-if="sessionStore.error" class="error">
              <Message severity="error" :closable="false">
                {{ sessionStore.error }}
              </Message>
            </div>
            <h3>{{ "Slot " + slot_selection }}</h3>
            <DataTable :value="selected_run" :scrollable="true">
                <template #header>
                    <h4>Runs</h4>
                </template>
                <Column field="scheduled_start" header="Scheduled Start">
                    <template #body="{ data }">
                    {{ formatTimeFromISO(data.scheduled_start) }}
                    </template>
                </Column>
                <Column field="scheduled_end" header="Scheduled End">
                    <template #body="{ data }">
                    {{ formatTimeFromISO(data.scheduled_end) }}
                    </template>
                </Column>
                <Column field="flip_allocation" header="Flip Allocation">
                    <template #body="{ data }">
                    {{ data.flip_allocation ? "Yes" : "No" }}
                    </template>
                </Column>
            </DataTable>
            <DataTable :value="selected_circuit" :scrollable="true">
                <template #header>
                    <h4>Circuits</h4>
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
        </Card>
      </div>
      <Card v-if="!sessionStore.loading && !sessionStore.error">
        <template #content>
          <div class="session-actions flex-column">
            <Button label="Edit Session" icon="pi pi-pencil" severity="primary" outlined @click="navigateTo(`/sessions/${sessionId}/edit`)" />
            <Button label="Upload Details" icon="pi pi-upload" severity="primary" outlined @click="upload_visible = true" />
            <Button v-if="sessionStore.session?.uploaded" label="View People" icon="pi pi-users" severity="primary" outlined @click="navigateTo(`/sessions/${sessionId}/people`)" />
            <Button v-if="sessionStore.session?.uploaded" label="Auto-Allocate" icon="pi pi-objects-column" severity="primary" outlined @click="navigateTo(`/sessions/${sessionId}/allocate`)" />
            <Button v-if="sessionStore.session?.allocated" label="Start" icon="pi pi-play" severity="primary" @click="navigateTo(`/sessions/${sessionId}/start`)" />
          </div>
        </template>
      </Card>
    </div>
  </div>
  <Dialog v-model:visible="upload_visible" class="upload-dialog" modal :draggable=false >
    <template #header>
        Upload Candidates/Examiners
    </template>
    <div class="flex-row">
      <FileUpload
        name="details_upload"
        accept=".xlsx"
        :maxFileSize="10000000"
        :customUpload="true"
        :multiple="false"
        @uploader="onUpload($event)"
        chooseLabel="Select XLSX File"
        uploadLabel="Upload"
        cancelLabel="Cancel"
      >
        <template #empty>
          <div class="flex-column upload-empty">
            <i class="pi pi-cloud-upload upload-icon" />
            <p>Drag and drop an XLSX file here to upload.</p>
          </div>
        </template>
      </FileUpload>
    </div>
    <template #footer>
        <Button label="Cancel" text severity="secondary" @click="upload_visible = false" autofocus />
    </template>
</Dialog>
</template>

<script lang="ts" setup>
import { useSessionStore } from "~/stores/session";
import { apiFetch } from "~/composables/apiFetch"
import { formatInterval, formatTimeFromISO } from "~/composables/formatting"
import { useToast } from "primevue/usetoast";

definePageMeta({
  layout: "session",
});

const upload_visible = ref(false);
const router = useRouter();
const route = useRoute();
const sessionId = computed(() => {
    if (route.params.sessionID && (!sessionStore.session || !sessionStore.stations || !sessionStore.slots ||  sessionStore.session.id !== route.params.sessionID)) {
      sessionStore.fetchSession(route.params.sessionID);
    }
    return route.params.sessionID;
});
const sessionStore = useSessionStore();
const toast = useToast();

const slot_selection = ref("A");
const slot_options = computed(() => { // when selecting, the value inside keeps getting changed
    return sessionStore.slots.map(slot => ({
        label: slot?.data.key.toString(),
        value: slot?.data.key.toString(),
    }));
});
const selected_slot = computed(() => {
    return sessionStore.slots.find(slot => slot.data.key === slot_selection.value) || null;
});
const selected_run = computed(() => {
    return selected_slot.value?.runs || [];
});
const selected_circuit = computed(() => {
    return selected_slot.value?.circuits || [];
});
const session_progress = computed(() => [
    { topic: "Creation", completed: true },
    { topic: "Upload", completed: sessionStore.session?.uploaded ?? false },
    { topic: "Allocation", completed: sessionStore.session?.allocated ?? false },
]);

const onUpload = async (event) => {
  const file = event.files[0];
  if (!file) {
    toast.add({
      severity: "error",
      summary: "Error",
      detail: "No file selected",
      life: 3000
    });
    return;
  }

  const formData = new FormData();
  formData.append("file", file);
  
  try {
    const response = await apiFetch(`/files/upload-xlsx?id=${sessionId.value}`, {
      method: "POST",
      body: formData,
    });
    toast.add({
      severity: "success",
      summary: "Success",
      detail: "File uploaded successfully",
      life: 3000
    });
    await sessionStore.fetchSession(sessionId.value);
  } catch (error) {
    toast.add({
      severity: "error",
      summary: "Upload Failed",
      detail: error.message || "An error occurred during upload",
      life: 3000
    });
  }
};

const navigateTo = (path) => {
  router.push(path);
};
</script>

<style>
.upload-dialog {
  font: var(--p);
}
</style>

<style scoped>
.session-container {
  margin: 0 auto;
  display: grid;
  grid-template-columns: 1fr 5fr;
}

.session-details p {
  margin: 0.5rem 0;
}

.timeline-card {
  margin-right: 2rem;
  display: flex;
  justify-content: center;
}

.session-timeline {
  align-self: center;
}

.timeline-marker {
  justify-content: center;
  align-items: center;
  border-radius: 100%;
}

:deep(.p-timeline-event-separator) {
  transform: translateY(5px);
}

.upload-empty {
  justify-content: center;
  align-items: center;
}

.upload-icon {
  border-radius: 9999px !important;
  border-color: var(--p-surface-100);
  border-width: 2px !important;
  border-style: solid;
  font-size: 3rem !important;
  color: var(--p-surface-200);
  padding: 2rem;
}

.right-column {
  display: grid;
  grid-template-columns: auto 15rem;
  column-gap: 2rem;
}

.session-actions {
  gap: .5rem;
}

.loading {
  display: flex;
  justify-content: center;
  padding: 2rem;
}

.error {
  padding: 1rem;
}
</style>