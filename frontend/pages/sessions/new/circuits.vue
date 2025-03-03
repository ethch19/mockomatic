<template>
  <div class="wizard-container">
    <Card class="session-form">
      <template #title>Set up Session - Step 3: Circuits</template>
      <template #content>
        <div class="form-section">
          <Button
            label="Add Circuit"
            icon="pi pi-plus"
            class="p-button-secondary"
            @click="addCircuit"
          />
          <DataTable :value="sessionStore.form.slots[0].circuits" :scrollable="true" class="editable-table">
            <Column field="key" header="Key">
              <template #body="{ data }">
                {{ data.key }}
              </template>
            </Column>
            <Column field="female_only" header="Female Only">
              <template #body="{ data }">
                <Checkbox
                  v-model="data.female_only"
                  :binary="true"
                  name="female_only"
                  @update:modelValue="sessionStore.setDirty"
                />
              </template>
            </Column>
          </DataTable>
        </div>
        <div class="wizard-actions">
          <Button
            label="Previous"
            icon="pi pi-arrow-left"
            class="p-button-secondary"
            @click="previousStep"
          />
          <Button
            label="Next"
            icon="pi pi-arrow-right"
            class="p-button-primary"
            @click="nextStep"
            :disabled="!hasCircuits"
          />
          <Button
            label="Cancel"
            icon="pi pi-times"
            class="p-button-secondary p-button-text"
            @click="cancel"
          />
        </div>
      </template>
    </Card>
  </div>
</template>

<script lang="ts" setup>
import { useSessionCreationStore } from "~/stores/sessionCreation";
import { useRouter } from "vue-router";
import Card from "primevue/card";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import DataTable from "primevue/datatable";
import Column from "primevue/column";

definePageMeta({
  layout: "default",
});

const sessionStore = useSessionCreationStore();
const router = useRouter();

const hasCircuits = computed(() => sessionStore.form.slots[0].circuits.length > 0);

const addCircuit = () => {
  sessionStore.addCircuit();
};

const previousStep = () => {
  sessionStore.step = 2;
  router.push("/sessions/new/stations");
};

const nextStep = () => {
  sessionStore.step = 4;
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

// Warn on beforeunload if there are unsaved changes
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
  min-height: 100vh;
  padding: 2rem;
}

.session-form {
  width: 100%;
  max-width: 800px;
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