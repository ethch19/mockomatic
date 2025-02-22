<template>
  <div class="wizard-container">
    <Card class="session-form">
      <template #title>Set up Session - Step 5: Review & Submit</template>
      <template #content>
        <div class="form-section">
          <h3>Configuration</h3>
          <p><strong>Organisation:</strong> {{ sessionStore.form.session.organisation }}</p>
          <p><strong>Scheduled Date:</strong> {{ formatDate(sessionStore.form.session.scheduled_date) }}</p>
          <p><strong>Location:</strong> {{ sessionStore.form.session.location }}</p>
          <p><strong>Intermission Duration:</strong> {{ formatInterval(sessionStore.form.session.intermission_duration) }}</p>
          <p><strong>Static at End:</strong> {{ sessionStore.form.session.static_at_end ? 'Yes' : 'No' }}</p>

          <h3>Stations</h3>
          <DataTable :value="sessionStore.form.stations" :scrollable="true">
            <Column field="title" header="Title" />
            <Column field="durationMinutes" header="Duration (min)" />
          </DataTable>

          <h3>Circuits</h3>
          <DataTable :value="sessionStore.form.slots[0].circuits" :scrollable="true">
            <Column field="key" header="Key" />
            <Column field="female_only" header="Female Only">
              <template #body="{ data }">
                {{ data.female_only ? 'Yes' : 'No' }}
              </template>
            </Column>
          </DataTable>

          <h3>Slots</h3>
          <h4>AM Slot</h4>
          <DataTable :value="sessionStore.form.slots.find(slot => slot.slot_time === 'AM')?.runs || []" :scrollable="true">
            <Column field="scheduled_start" header="Scheduled Start" />
            <Column field="scheduled_end" header="Scheduled End" />
          </DataTable>
          <h4>PM Slot</h4>
          <DataTable :value="sessionStore.form.slots.find(slot => slot.slot_time === 'PM')?.runs || []" :scrollable="true">
            <Column field="scheduled_start" header="Scheduled Start" />
            <Column field="scheduled_end" header="Scheduled End" />
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
            label="Submit"
            icon="pi pi-check"
            class="p-button-primary"
            @click="submitForm"
            :disabled="!isValid"
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
import { computed } from 'vue';
import Card from 'primevue/card';
import Button from 'primevue/button';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import { apiFetch } from '~/composables/apiFetch';

definePageMeta({
  layout: 'default',
});

const sessionStore = useSessionCreationStore();
const router = useRouter();

const isValid = computed(() => {
  return !!sessionStore.form.session.organisation &&
         !!sessionStore.form.session.scheduled_date &&
         !!sessionStore.form.session.location &&
         sessionStore.form.stations.length > 0 &&
         sessionStore.form.slots[0].circuits.length > 0 &&
         sessionStore.form.slots.some(slot => slot.runs.length > 0);
});

const formatDate = (date) => {
  return date ? new Date(date).toLocaleDateString() : 'N/A';
};

const formatInterval = (interval) => {
  if (!interval || typeof interval !== 'object') return 'N/A';
  const minutes = Math.floor(interval.microseconds / (1_000_000 * 60));
  const seconds = Math.floor((interval.microseconds % (1_000_000 * 60)) / 1_000_000);
  return `${minutes}:${seconds.toString().padStart(2, '0')} min`;
};

const previousStep = () => {
  sessionStore.step = 4;
  router.push('/new-session/slots');
};

const submitForm = async () => {
  try {
    const response = await apiFetch('/sessions/create', {
      method: 'POST',
      body: sessionStore.form,
    });
    sessionStore.resetForm();
    router.push(`/sessions/${response.id}`);
  } catch (error) {
    console.error('Submit error:', error);
  }
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