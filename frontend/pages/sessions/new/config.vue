<template>
  <div class="wizard-container text">
    <div class="main-container flex-column">
      <h2>Create New Session</h2>
      <Card class="session-form">
        <template #title>Step 1: Configuration</template>
        <template #content>
          <div class="form-section flex-row">
            <InputGroup>
              <InputGroupAddon>
                <i class="pi pi-building"></i>
              </InputGroupAddon>
              <FloatLabel variant="in" class="flex-row">
                <AutoComplete id="organisation" v-model="sessionStore.form.session.organisation" :suggestions="filtered_soc" :pt="{ overlay: { style: 'font: var(--p);' } }" @complete="soc_search" required @update:modelValue="sessionStore.setDirty" fluid />
                <label for="organisation">Organisation</label>
              </FloatLabel>
            </InputGroup>
            <InputGroup>
              <InputGroupAddon>
                <i class="pi pi-calendar"></i>
              </InputGroupAddon>
              <FloatLabel variant="in" class="flex-row">
                <Calendar id="scheduled_date" showButtonBar :pt="{ panel: { style: 'font: var(--p)' } }" v-model="sessionStore.form.session.scheduled_date" dateFormat="dd/mm/yy" :showTime="false" :yearRange="`2025:${new Date().getFullYear() + 10}`" fluid required @update:modelValue="sessionStore.setDirty" />
                <label for="scheduled_date">Scheduled Date</label>
              </FloatLabel>
            </InputGroup>
            <InputGroup>
              <InputGroupAddon>
                <i class="pi pi-map-marker"></i>
              </InputGroupAddon>
              <FloatLabel variant="in" class="flex-row">
                <InputText id="location" v-model="sessionStore.form.session.location" required @update:modelValue="sessionStore.setDirty" fluid />
                <label for="location">Location</label>
              </FloatLabel>
            </InputGroup>
            <div class="flex-row toggle-group">
              <label class="field">Intermission Duration:</label>
              <FloatLabel variant="in" class="duration-field flex-row">
                <InputNumber id="intermission_duration_sec" :useGrouping="false" fluid v-model="sessionStore.intermissionSeconds" :min="1" @update:modelValue="sessionStore.updateIntermissionDuration(); sessionStore.setDirty" />
                <label for="intermission_duration_sec">Second(s)</label>
              </FloatLabel>
            </div>
            <div class="flex-row toggle-group">
              <div class="flex-row form-field toggle">
                <label class="field" for="feedback">Feedback:</label>
                <ToggleSwitch v-model="sessionStore.form.session.feedback" inputId="feedback"/>
              </div>
              <FloatLabel class="flex-row duration-field" v-if="sessionStore.form.session.feedback" variant="in">
                <InputNumber inputId="feedback_duration" v-model="sessionStore.feedbackSeconds" :min="1" :useGrouping="false" fluid @update:modelValue="sessionStore.updateFeedbackDuration(); sessionStore.setDirty" />
                <label for="feedback_duration">Second(s)</label>
              </FloatLabel>
            </div>
            <div class="flex-row toggle">
              <label class="field" for="static_at_end">Static at End:</label>
              <ToggleSwitch inputId="static_at_end" v-model="sessionStore.form.session.static_at_end" @update:modelValue="sessionStore.setDirty" />
            </div>
          </div>
        </template>
        <template #footer>
          <div class="wizard-actions">
            <Button label="Next" icon="pi pi-arrow-right" @click="nextStep" :disabled="!isValid" />
            <Button label="Cancel" icon="pi pi-times" severity="secondary" @click="cancel" />
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useSessionCreationStore } from "~/stores/sessionCreation";

definePageMeta({
  layout: "default",
});

const soc_list = ref(["AMSA", "MedEd", "SurgSoc", "MM"]);
const filtered_soc = ref([]);
const sessionStore = useSessionCreationStore();
const router = useRouter();

const isValid = computed(() => {
  return !!sessionStore.form.session.organisation &&
         !!sessionStore.form.session.scheduled_date &&
         !!sessionStore.form.session.location;
});

const soc_search = (event) => {
  setTimeout(() => {
      if (!event.query.trim().length) {
        filtered_soc.value = [...soc_list.value];
      } else {
        filtered_soc.value = soc_list.value.filter((soc) => {
            return soc.toLowerCase().startsWith(event.query.toLowerCase());
        });
      }
  }, 250);
}

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

.toggle {
  gap: 1rem;
  justify-content: center;
  align-items: center;
  align-self: center;
}

.toggle-group {
  gap: 0.5rem;
  align-items: center;
}

.duration-field {
  width: 5rem;
}

.p-inputgroup {
  min-width: 45%;
  max-width: 48%;
  width: auto;
}
</style>