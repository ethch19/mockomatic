<template>
  <div class="wizard-container">
    <Card class="session-form">
      <template #title>Set up Session - Step 4: Slots</template>
      <template #content>
        <div class="form-section">
          <h4>AM Slot</h4>
          <Button
            label="Add Run"
            icon="pi pi-plus"
            class="p-button-secondary"
            @click="addRun('AM')"
          />
          <DataTable
            :value="sessionStore.form.slots.find(slot => slot.slot_time === 'AM')?.runs || []"
            :scrollable="true"
            class="editable-table"
            :sortField="'scheduled_start'"
            :sortOrder="1"
          >
            <Column field="scheduled_start" header="Scheduled Start (Time)">
              <template #body="{ data }">
                <InputMask
                  v-model="data.scheduled_start"
                  mask="99:99"
                  placeholder="HH:mm"
                  slotChar="0"
                  required
                  @update:modelValue="sessionStore.setDirty"
                />
              </template>
            </Column>
            <Column field="scheduled_end" header="Scheduled End (Time)">
              <template #body="{ data }">
                <InputMask
                  v-model="data.scheduled_end"
                  mask="99:99"
                  placeholder="HH:mm"
                  slotChar="0"
                  required
                  @update:modelValue="sessionStore.setDirty"
                />
              </template>
            </Column>
          </DataTable>
          <h4>PM Slot</h4>
          <Button
            label="Add Run"
            icon="pi pi-plus"
            class="p-button-secondary"
            @click="addRun('PM')"
          />
          <DataTable
            :value="sessionStore.form.slots.find(slot => slot.slot_time === 'PM')?.runs || []"
            :scrollable="true"
            class="editable-table"
            :sortField="'scheduled_start'"
            :sortOrder="1"
          >
            <Column field="scheduled_start" header="Scheduled Start (Time)">
              <template #body="{ data }">
                <InputMask
                  v-model="data.scheduled_start"
                  mask="99:99"
                  placeholder="HH:mm"
                  slotChar="0"
                  required
                  @update:modelValue="sessionStore.setDirty"
                />
              </template>
            </Column>
            <Column field="scheduled_end" header="Scheduled End (Time)">
              <template #body="{ data }">
                <InputMask
                  v-model="data.scheduled_end"
                  mask="99:99"
                  placeholder="HH:mm"
                  slotChar="0"
                  required
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
            :disabled="!hasRuns"
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
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import InputMask from 'primevue/inputmask';

definePageMeta({
  layout: 'default',
});

const sessionStore = useSessionCreationStore();
const router = useRouter();

const hasRuns = computed(() => {
  return sessionStore.form.slots.some(slot => slot.runs.length > 0);
});

const addRun = (slotTime: string) => {
  const slot = sessionStore.form.slots.find(s => s.slot_time === slotTime);
  if (slot) {
    slot.runs.push({ scheduled_start: '09:00', scheduled_end: '10:00' });
    sessionStore.setDirty();
  }
};

const previousStep = () => {
  sessionStore.step = 3;
  router.push('/new-session/circuits');
};

const nextStep = () => {
  sessionStore.step = 5;
  router.push('/new-session/review');
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

.wizard-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}
</style>