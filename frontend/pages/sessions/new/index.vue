<template>
  <div class="wizard-container text">
    <Card class="session-form">
      <template #title>Set up Session - Step 1: Configuration</template>
      <template #content>
        <div class="form-section">
          <div class="form-group">
            <label for="organisation">Organisation:</label>
            <InputText
              id="organisation"
              v-model="sessionStore.form.session.organisation"
              placeholder="Enter organisation"
              required
              @update:modelValue="sessionStore.setDirty"
            />
          </div>
          <div class="form-group">
            <label for="scheduled_date">Scheduled Date:</label>
            <Calendar
              id="scheduled_date"
              v-model="sessionStore.form.session.scheduled_date"
              dateFormat="dd/mm/yy"
              :showTime="false"
              :yearRange="`2025:${new Date().getFullYear() + 10}`"
              required
              @update:modelValue="sessionStore.setDirty"
            />
          </div>
          <div class="form-group">
            <label for="location">Location:</label>
            <InputText
              id="location"
              v-model="sessionStore.form.session.location"
              placeholder="Enter location"
              required
              @update:modelValue="sessionStore.setDirty"
            />
          </div>
          <div class="form-group">
            <label for="intermission_duration">Intermission Duration (min:sec):</label>
            <div class="duration-input">
              <InputNumber
                id="intermission_duration_min"
                v-model="sessionStore.intermissionMinutes"
                :min="0"
                placeholder="Minutes"
                @update:modelValue="sessionStore.updateIntermissionDuration(); sessionStore.setDirty"
              />
              <span>:</span>
              <InputNumber
                id="intermission_duration_sec"
                v-model="sessionStore.intermissionSeconds"
                :min="0"
                :max="59"
                placeholder="Seconds"
                @update:modelValue="sessionStore.updateIntermissionDuration(); sessionStore.setDirty"
              />
            </div>
          </div>
          <div class="form-group">
            <label for="static_at_end">Static at End:</label>
            <Checkbox
              id="static_at_end"
              v-model="sessionStore.form.session.static_at_end"
              :binary="true"
              name="static_at_end"
              @update:modelValue="sessionStore.setDirty"
            />
          </div>
        </div>
        <div class="wizard-actions">
          <Button
            label="Next"
            icon="pi pi-arrow-right"
            class="p-button-primary"
            @click="nextStep"
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
import { useSessionCreationStore } from "~/stores/sessionCreation";

definePageMeta({
  layout: "default",
});

const sessionStore = useSessionCreationStore();
const router = useRouter();

const isValid = computed(() => {
  return !!sessionStore.form.session.organisation &&
         !!sessionStore.form.session.scheduled_date &&
         !!sessionStore.form.session.location;
});

const nextStep = () => {
  if (isValid.value) {
    sessionStore.step = 2;
    router.push("/sessions/new/stations");
  }
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

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: bold;
}

.duration-input {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.wizard-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}
</style>