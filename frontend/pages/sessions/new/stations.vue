<template>
  <ConfirmDialog/>
  <div class="wizard-container text">
    <div class="main-container flex-column">
      <h2>Create New Session</h2>
      <Card class="session-form">
        <template #title>Step 2: Stations</template>
        <template #content>
          <ConfirmPopup class="text" /> 
          <DataTable :value="sessionStore.stationsMinutes" :scrollable="true" v-model:selection="selectedStations" selectionMode="multiple" @row-reorder="onRowReorder" >
            <template #header>
              <div class="flex-row table-header">
                <Button label="Add Station" icon="pi pi-plus" severity="secondary" @click="addStation" />
                <Button v-if="selectedStations.length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" outlined @click="confirmDelete($event)" />
              </div>
            </template>
            <Column :rowReorder="true" headerStyle="width: 3rem" />
            <Column field="index" header="Index">
              <template #body="{ data, index }">
                <label class="field">{{ index }}</label>
              </template>
            </Column>
            <Column field="title" header="Title">
              <template #body="{ data, index }">
                <InputText v-model="data.title" placeholder="Station title" required @update:modelValue="sessionStore.setDirty" />
              </template>
            </Column>
            <Column field="durationMinutes" header="Duration (min)">
              <template #body="{ data, index }">
                <InputNumber v-model="data.duration" :min="1" placeholder="Minutes" @update:modelValue="sessionStore.updateStationDuration(index, $event); sessionStore.setDirty" />
              </template>
            </Column>
          </DataTable>
        </template>
        <template #footer>
          <div class="wizard-actions">
            <Button label="Previous" icon="pi pi-arrow-left" severity="secondary" @click="previousStep" />
            <Button label="Next" icon="pi pi-arrow-right" @click="nextStep" :disabled="!hasStations" />
            <Button label="Cancel" icon="pi pi-times" severity="secondary" @click="cancel" />
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useSessionCreationStore } from "~/stores/sessionCreation";
import { useToast } from "primevue/usetoast";
import { useConfirm } from "primevue/useconfirm";

definePageMeta({
  layout: "default",
});

const sessionStore = useSessionCreationStore();
const selectedStations = ref([]);
const router = useRouter();
const confirm = useConfirm();
const toast = useToast();

const hasStations = computed(() => sessionStore.form.stations.length > 0);

const addStation = () => {
  sessionStore.addStation();
};

const onRowReorder = (event) => {
  sessionStore.onStationRowReorder(event);
};

const previousStep = () => {
  sessionStore.step = 1;
  router.push("/sessions/new/config");
};

const nextStep = () => {
  sessionStore.step = 3;
  router.push("/sessions/new/slots");
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

const confirmDelete = (event) => {
  confirm.require({
    target: event.currentTarget,
    message: `Are you sure you want to delete ${selectedStations.value.length} stations(s)?`,
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
    accept: () => {
      toast.add({ severity: "info", summary: "Confirmed", detail: "Stations Deleted", life: 3000 });
      sessionStore.removeStations(selectedStations.value)
      selectedStations.value = [];
    },
  });
}

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

.table-header {
  justify-content: space-between;
}

.main-container {
  width: 50%;
}

.session-form {
  width: 100%;
}

.form-section {
  flex-wrap: wrap;
  gap: 1rem;
  justify-content: space-between;
  align-items: flex-start;
  align-content: flex-start;
}

.editable-table :deep(.p-datatable-tbody > tr) {
  cursor: move;
}

.wizard-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}
</style>