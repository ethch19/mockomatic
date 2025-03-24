<template>
  <div class="wizard-container text">
    <div class="main-container flex-column">
      <h2>Create New Session</h2>
      <Card class="session-form">
        <template #title>Step 3: Slots</template>
        <template #content>
          <ConfirmPopup class="text" />
          <div class="button-actions">
            <Button label="Add Slot" icon="pi pi-plus" severity="secondary" @click="addSlot()"/>
          </div>
          <div class="form-section">
            <template v-for="cur_slot in sessionStore.form.slots" :key="cur_slot.key">
              <h4>{{ "Slot " + cur_slot.key }}</h4>
              <DataTable :value="cur_slot.runs" :scrollable="true" selectionMode="multiple" v-model:selection="selectedRuns[cur_slot.key]">
                <template #header>
                  <div class="flex-row table-header">
                    <Button label="Add Run" icon="pi pi-plus" severity="secondary" @click="sessionStore.addRun(cur_slot.key)" />
                    <div class="flex-row table-subheader">
                      <Button v-if="selectedRuns[cur_slot.key].length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" outlined @click="confirmDeleteRuns(cur_slot.key, $event)" />
                      <Button label="Delete Slot" icon="pi pi-trash" severity="danger" @click="confirmDeleteSlot(cur_slot.key, $event)" />
                    </div>
                  </div>
                </template>
                <Column field="scheduled_start" header="Scheduled Start (Time)">
                  <template #body="{ data, index }">
                    <InputMask v-model="data.scheduled_start" mask="99:99" placeholder="HH:MM" fluid @update:modelValue="sessionStore.setDirty; sessionStore.updateScheduledEnd($event, cur_slot.key, index)" />
                  </template>
                </Column>
                <Column field="scheduled_end" header="Scheduled End (Time)">
                  <template #body="{ data }">
                    {{ data.scheduled_end }}
                  </template>
                </Column>
                <Column field="flip_allocation" header="Flip Allocation">
                  <template #body="{ data }">
                    <ToggleSwitch v-model="data.flip_allocation" @update:modelValue="sessionStore.setDirty" fluid />
                  </template>
                </Column>
              </DataTable>
            </template>
          </div>
          <div class="wizard-actions">
            <Button label="Previous" icon="pi pi-arrow-left" class="p-button-secondary" @click="previousStep" />
            <Button label="Next" icon="pi pi-arrow-right" class="p-button-primary" @click="nextStep" :disabled="!hasRuns" />
            <Button label="Cancel" icon="pi pi-times" class="p-button-secondary p-button-text" @click="cancel" />
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
const router = useRouter();
const selectedRuns = ref({});
const confirm = useConfirm();
const toast = useToast();

const hasRuns = computed(() => {
  return sessionStore.form.slots.some(slot => slot.runs.length > 0);
});

const addSlot = () => {
  let key = sessionStore.addSlot();
  selectedRuns.value[key] = [];
}

const confirmDeleteSlot = (slot_time: string, event) => {
  confirm.require({
    target: event.currentTarget,
    message: `Are you sure you want to delete Slot ${slot_time}?`,
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
      toast.add({ severity: "info", summary: "Confirmed", detail: "Slot Deleted", life: 3000 });
      sessionStore.removeSlot(slot_time);
      selectedRuns.value[slot_time] = [];
    },
  });
}

const confirmDeleteRuns = (slot_time: string, event) => {
  confirm.require({
    target: event.currentTarget,
    message: `Are you sure you want to delete ${selectedRuns.value[slot_time].length} run(s)?`,
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
      toast.add({ severity: "info", summary: "Confirmed", detail: "Runs Deleted", life: 3000 });
      sessionStore.removeRuns(slot_time, selectedRuns.value[slot_time]);
      selectedRuns.value[slot_time] = [];
    },
  });
}

const previousStep = () => {
  sessionStore.step = 2;
  router.push("/sessions/new/stations");
};

const nextStep = () => {
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
  for (const slot of sessionStore.form.slots) {
    selectedRuns.value[slot.key] = [];
  }
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
.table-subheader {
  justify-content: space-between;
  gap: 0.5rem;
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