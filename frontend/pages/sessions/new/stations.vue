<template>
  <div class="wizard-container">
    <Card class="session-form">
      <template #title>Set up Session - Step 2: Stations</template>
      <template #content>
        <div class="form-section">
          <Button
            label="Add Station"
            icon="pi pi-plus"
            class="p-button-secondary"
            @click="addStation"
          />
          <DataTable
            :value="sessionStore.form.stations"
            :scrollable="true"
            class="editable-table"
            @row-reorder="onRowReorder"
          >
            <Column
              :rowReorder="true"
              headerStyle="width: 3rem"
            />
            <Column field="title" header="Title">
              <template #body="{ data, index }">
                <InputText
                  v-model="data.title"
                  placeholder="Station title"
                  required
                  @update:modelValue="sessionStore.setDirty"
                />
              </template>
            </Column>
            <Column field="durationMinutes" header="Duration (min)">
              <template #body="{ data, index }">
                <InputNumber
                  v-model="data.durationMinutes"
                  :min="1"
                  placeholder="Minutes"
                  @update:modelValue="sessionStore.updateStationDuration(index, $event); sessionStore.setDirty"
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
            :disabled="!hasStations"
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
import { useSessionCreationStore } from '~/stores/sessionCreation';
import { useRouter } from 'vue-router';
import Card from 'primevue/card';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';

definePageMeta({
  layout: 'default',
});

const sessionStore = useSessionCreationStore();
const router = useRouter();

const hasStations = computed(() => sessionStore.form.stations.length > 0);

const addStation = () => {
  sessionStore.addStation();
};

const onRowReorder = (event) => {
  sessionStore.onRowReorder(event);
};

const previousStep = () => {
  sessionStore.step = 1;
  router.push('/new-session');
};

const nextStep = () => {
  sessionStore.step = 3;
  router.push('/new-session/circuits');
};

const cancel = () => {
  if (sessionStore.isDirty) {
    if (confirm('You have unsaved changes. Are you sure you want to cancel and lose progress?')) {
      sessionStore.resetForm();
      router.push('/');
    }
  } else {
    sessionStore.resetForm();
    router.push('/');
  }
};

// Warn on beforeunload if there are unsaved changes
onBeforeMount(() => {
  window.onbeforeunload = () => {
    if (sessionStore.isDirty) {
      return 'You have unsaved changes. Are you sure you want to leave?';
    }
  };
});

onUnmounted(() => {
  window.onbeforeunload = null;
  sessionStore.resetForm(); // Clean up on navigation away
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

.editable-table :deep(.p-datatable-tbody > tr) {
  cursor: move;
}

.wizard-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}
</style>