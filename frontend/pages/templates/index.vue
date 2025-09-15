<template>
  <div class="template-container text">
    <h3 class="subhead">Templates</h3>
    <div class="main-container flex-column">
      <template v-for="cur_tem in templates" :key="cur_tem.id">
        <h2>{{ cur_tem.name }}</h2>
        <div class="flex-row template-details">
          <div class="flex-column">
            <div class="field">
              <label class="field-label">Total Stations:</label>
              <span>{{ cur_tem.total_stations }}</span>
            </div>
            <div class="field">
              <label class="field-label">Feedback:</label>
              <span>{{ cur_tem.feedback ? "Yes" : "No" }}</span>
            </div>
            <div class="field">
              <label class="field-label">Feedback Duration:</label>
              <span>{{ formatInterval(cur_tem.feedback_duration) }}</span>
            </div>
            <div class="field">
              <label class="field-label">Intermission Duration:</label>
              <span>{{ formatInterval(cur_tem.intermission_duration) }}</span>
            </div>
          </div>
          <ConfirmPopup class="text" /> 
          <div class="header-actions">
            <Button label="Delete Template" icon="pi pi-trash" severity="danger" outlined @click="confirmDelete(cur_tem.id, $event)" />
          </div>
        </div>
        <DataTable :value="cur_tem.stations" :scrollable="true">
          <template #header>
            <h4>Stations</h4>
          </template>
          <Column field="index" header="Index">
            <template #body="{ data }"> {{ data.index }}</template>
          </Column>
          <Column field="title" header="Title">
            <template #body="{ data }"> {{ data.title }}</template>
          </Column>
          <Column field="duration" header="Duration">
            <template #body="{ data }"> {{ formatInterval(data.duration) }}</template>
          </Column>
        </DataTable>
        <br>
      </template>
    </div>
    <div class="template-actions flex-row">
      <Button label="Return" icon="pi pi-arrow-left" severity="secondary" @click="returnMain"/>
      <Button label="Create New" icon="pi pi-plus" @click="createNew"/>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~~/composables/apiFetch";
import { useToast } from "primevue/usetoast";
import { useConfirm } from "primevue/useconfirm";

definePageMeta({
  layout: "default",
});

const templates = ref([]);
const loading = ref(false);
const router = useRouter();
const confirm = useConfirm();
const toast = useToast();

const formatInterval = (interval) => {
  if (!interval || typeof interval !== "object") return 0;

  const { months = 0, days = 0, microseconds = 0 } = interval;

  const daysFromMonths = months * 30;
  const totalDays = daysFromMonths + days;
  const minutesFromDays = totalDays * 24 * 60;
  const minutesFromMicroseconds = microseconds / (1_000_000 * 60);

  return Math.round(minutesFromDays + minutesFromMicroseconds) + " min"; // in minutes
};

const loadTemplates = async () => {
  if (loading.value) return;
  loading.value = true;
  try {
    const response = await apiFetch("/templates/get-all");
    console.log(response);
    templates.value = response;
  } catch (error) {
    if (error.message !== "Unauthorized - Redirecting to login") {
      toast.add({
        severity: "error",
        summary: "Error",
        detail: "Failed to load sessions",
        life: 3000,
      });
    }
  } finally {
    loading.value = false;
  }
};

const confirmDelete = async (tem_id: string, event) => {
  confirm.require({
    target: event.currentTarget,
    message: `Are you sure you want to delete this template?`,
    header: "Confirm Deletion",
    icon: "pi pi-info-circle",
    rejectProps: {
      label: "Cancel",
      severity: "secondary",
      outlined: true
    },
    acceptProps: {
        label: "Delete",
        severity: "danger"
    },
    accept: async () => {
      toast.add({ severity: "info", summary: "Confirmed", detail: "Template Deleted", life: 3000 });
      try {
        const response = await apiFetch("/templates/delete", {
          method: "POST",
          body: {ids: [tem_id]},
        });
        const temp_index = templates.value.findIndex(t => t.id === tem_id);
        templates.value.splice(temp_index, 1);
      } catch (error) {
        console.error("Submit error:", error);
      }
    },
  });
}

const returnMain = () => {
  router.push("/");
};

const createNew = () => {
  router.push("/templates/new/");
};

onMounted(() => {
  loadTemplates();
});
</script>

<style>
.template-container {
  width: 50%;
  align-self: center;
  gap: 1rem;
  background-color: var(--p-surface-0);
  border-radius: var(--radius-m);
  padding: 2rem;
}

.template-details {
  justify-content: space-between;
}

.header-actions {
  margin-bottom: 1rem;
}

.field {
  margin-bottom: 0.5rem;
}

.field-label {
  font-weight: bold;
  margin-right: 0.5rem;
}

.template-actions {
  justify-content: space-between;
}
</style>