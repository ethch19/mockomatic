<template>
  <div class="wizard-container text">
    <div class="main-container flex-column">
      <h2>Create New Session</h2>
      <Card class="session-form">
        <template #title>Use Template</template>
        <template #content>
          <div class="form-section flex-row">
            <Listbox v-model="selectedTemplate" :options="session_templates" optionLabel="name" checkmark @update:modelValue="templateSelected($event); sessionStore.setDirty" fluid>
              <template #option="slotProps">
                {{ slotProps.option.name }}
              </template>
            </Listbox>
          </div>
        </template>
        <template #footer>
          <div class="wizard-actions">
            <Button label="Next" icon="pi pi-arrow-right" @click="nextStep" />
            <Button label="Cancel" icon="pi pi-times" severity="secondary" @click="cancel" />
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useSessionCreationStore } from "~/stores/sessionCreation";
import { apiFetch } from "~/composables/apiFetch"
import { useToast } from "primevue/usetoast";

definePageMeta({
  layout: "default",
});

const selectedTemplate = ref();
const session_templates = ref([]);
const sessionStore = useSessionCreationStore();
const toast = useToast();
const router = useRouter();

const templateSelected = (event) => {
  sessionStore.applyTemplate(event);
  toast.add({ severity: "info", summary: "Confirmed", detail: "Template applied", life: 3000 });
}

const nextStep = () => {
  router.push("/sessions/new/config");
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

onMounted(async () => {
  try {
    const response = await apiFetch("/templates/get-all", {
      method: "GET",
    });
    session_templates.value = response;
    console.log(response);
  } catch (error) {
    console.error("GET templates error:", error);
  }
});

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