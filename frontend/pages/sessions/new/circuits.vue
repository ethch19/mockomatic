<template>
  <div class="wizard-container text">
    <div class="main-container flex-column">
      <h2>Create New Session</h2>
      <Card class="session-form">
        <template #title>Step 4: Circuits</template>
        <template #content>
          <ConfirmPopup class="text" />
          <template v-for="cur_slot in sessionStore.form.slots" :key="cur_slot.key">
            <h4>{{ "Slot " + cur_slot.key }}</h4>
            <DataTable :value="cur_slot.circuits" :scrollable="true" selectionMode="multiple" v-model:selection="selectedCircuits[cur_slot.key]">
              <template #header>
                <div class="flex-row table-header">
                  <Button label="Add Circuit" icon="pi pi-plus" severity="secondary" @click="sessionStore.addCircuit(cur_slot.key); selectedCircuits[cur_slot.key] = []" />
                  <Button v-if="selectedCircuits[cur_slot.key].length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" outlined @click="confirmDelete(cur_slot.key, $event)" />
                </div>
              </template>
              <Column field="key" header="Key">
                <template #body="{ data }">
                  {{ data.key }}
                </template>
              </Column>
              <Column field="female_only" header="Female Only">
                <template #body="{ data }">
                  <ToggleSwitch v-model="data.female_only" inputId="female_only" @update:modelValue="sessionStore.setDirty" fluid />
                </template>
              </Column>
            </DataTable>
          </template>
        </template>
        <template #footer>
          <div class="wizard-actions">
            <Button label="Previous" icon="pi pi-arrow-left" severity="secondary" @click="previousStep" />
            <Button label="Next" icon="pi pi-arrow-right" @click="nextStep" :disabled="!hasCircuits" />
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
const router = useRouter();
const selectedCircuits = ref({});
const confirm = useConfirm();
const toast = useToast();

const hasCircuits = computed(() => sessionStore.form.slots[0].circuits.length > 0);

const confirmDelete = (slot_key: string, event) => {
  confirm.require({
    target: event.currentTarget,
    message: `Are you sure you want to delete ${selectedCircuits.value[slot_key].length} circuit(s)?`,
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
      toast.add({ severity: "info", summary: "Confirmed", detail: "Circuits Deleted", life: 3000 });
      sessionStore.removeCircuits(slot_key, selectedCircuits.value[slot_key])
      selectedCircuits.value[slot_key] = [];
    },
  });
}

const previousStep = () => {
  sessionStore.step = 3;
  router.push("/sessions/new/slots");
};

const nextStep = () => {
  sessionStore.step = 5;
  router.push("/sessions/new/review");
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
    selectedCircuits.value[slot.key] = [];
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