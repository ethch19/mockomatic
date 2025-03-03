<template>
  <div class="home-container text">
    <div class="button-container">
      <Button label="New Session" icon="pi pi-plus" @click="navigateTo('/sessions/new/')" />
      <Button label="New Template" icon="pi pi-plus" severity="secondary" @click="navigateTo('/templates/new/')" />
    </div>
    <DataTable
      :value="sessions"
      :loading="loading"
      :paginator="true"
      :rows="15"
      :scrollable="true"
      :lazy="true"
      :totalRecords="totalRecords"
      @page="onPage"
      v-model:selection="selectedSessions"
      selectionMode="multiple"
      dataKey="id"
      :sortField="sortField"
      :sortOrder="sortOrder"
      @sort="onSort"
    >
      <template #header>
        <div class="table-header">
          <Button v-if="selectedSessions.length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" @click="confirmDelete" />
        </div>
      </template>
      <Column selectionMode="multiple" headerStyle="width: 3rem" />
      <Column field="id" header="ID" headerStyle="width: 15rem">
        <template #body="{ data }"> {{ data.id }} </template>
      </Column>
      <Column field="organiser_id" header="Organiser ID" headerStyle="width: 15rem">
        <template #body="{ data }"> {{ data.organiser_id }} </template>
      </Column>
      <Column field="organisation" header="Organisation" headerStyle="width: 5rem">
        <template #body="{ data }"> {{ data.organisation }} </template>
      </Column>
      <Column field="scheduled_date" header="Scheduled Date" sortable>
        <template #body="{ data }"> {{ formatDate(data.scheduled_date) }} </template>
      </Column>
      <Column field="location" header="Location" headerStyle="width: 10rem">
        <template #body="{ data }"> {{ data.location }} </template>
      </Column>
      <Column field="total_stations" header="Total Stations" headerStyle="width: 10rem">
        <template #body="{ data }"> {{ data.total_stations }} </template>
      </Column>
      <Column field="intermission_duration" header="Intermission Duration" headerStyle="width: 10rem">
        <template #body="{ data }"> {{ formatInterval(data.intermission_duration) }} </template>
      </Column>
      <Column field="static_at_end" header="Static At End" headerStyle="width: 10rem">
        <template #body="{ data }"> {{ data.static_at_end }} </template>
      </Column>
      <Column field="created_at" header="Created At" sortable>
        <template #body="{ data }"> {{ formatDate(data.created_at) }} </template>
      </Column>
      <Column headerStyle="width: 8rem">
        <template #body="{ data }">
          <Button icon="pi pi-pencil" class="p-button-rounded" @click="navigateTo(`/sessions/${data.id}`)" />
        </template>
      </Column>
    </DataTable>
    <ConfirmDialog />
  </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~/composables/apiFetch"
import { useToast } from "primevue/usetoast";
import { useConfirm } from "primevue/useconfirm";

const router = useRouter();
const toast = useToast();
const confirm = useConfirm();

const sessions = ref([]);
const loading = ref(false);
const totalRecords = ref(0);
const selectedSessions = ref([]);
const sortField = ref("scheduled_date");
const sortOrder = ref(-1); // -1 for descending, 1 for ascending

const lazyParams = ref({
  first: 0,
  rows: 15,
  sortField: "scheduled_date",
  sortOrder: -1,
});

const loadSessions = async (params = lazyParams.value) => {
  if (loading.value) return;
  loading.value = true;
  try {
    const response = await apiFetch("/sessions/get-page", {
      params: {
        first: lazyParams.value.first,
        rows: lazyParams.value.rows,
        sortField: lazyParams.value.sortField,
        sortOrder: lazyParams.value.sortOrder,
      },
    });
    sessions.value = response.sessions;
    totalRecords.value = response.total;
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

const onPage = (event) => {
  lazyParams.value = { ...event };
  loadSessions(lazyParams.value);
};

const onSort = (event) => {
  lazyParams.value = { ...lazyParams.value, sortField: event.sortField, sortOrder: event.sortOrder };
  loadSessions(lazyParams.value);
};

const formatDate = (dateString) => {
  return new Date(dateString).toLocaleDateString();
};

const formatInterval = (interval) => {
  if (!interval || typeof interval !== "object") return 0;

  const { months = 0, days = 0, microseconds = 0 } = interval;

  const daysFromMonths = months * 30;
  const totalDays = daysFromMonths + days;
  const minutesFromDays = totalDays * 24 * 60;
  const minutesFromMicroseconds = microseconds / (1_000_000 * 60);

  return Math.round(minutesFromDays + minutesFromMicroseconds) + " min"; // in minutes
};

const navigateTo = (path) => {
  router.push(path);
};

const confirmDelete = () => {
  confirm.require({
    message: `Are you sure you want to delete ${selectedSessions.value.length} session(s)?`,
    header: "Confirm Deletion",
    icon: "pi pi-exclamation-triangle",
    accept: async () => {
      try {
        await $fetch("/sessions/delete", {
          method: "POST",
          body: { ids: selectedSessions.value.map(s => s.id) },
        });
        toast.add({
          severity: "success",
          summary: "Success",
          detail: "Sessions deleted successfully",
          life: 3000,
        });
        selectedSessions.value = [];
        loadSessions();
      } catch (error) {
        toast.add({
          severity: "error",
          summary: "Error",
          detail: "Failed to delete sessions",
          life: 3000,
        });
      }
    },
  });
}

onMounted(() => {
  loadSessions();
});
</script>

<style scoped>
.home-container {
  padding: 2rem;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.button-container {
  margin-bottom: 1rem;
  width: 100%;
  max-width: 1200px;
  display: flex;
  justify-content: flex-start;
  gap: 1rem;
}

:deep(.p-datatable) {
  width: 100%;
  max-width: 1200px;
}

.table-header {
  display: flex;
  justify-content: flex-end;
  padding: 0.5rem;
}
</style>