<template>
  <ConfirmDialog class="text" /> 
  <div class="wizard-container text flex-column">
    <h3 class="subhead">Create new template</h3>
    <div class="flex-row field-group">
      <FloatLabel variant="in" class="flex-row form-field">
        <div class="flex-column">
          <InputText v-model="templateStore.template.name" type="text" />
        </div>
        <label for="name">Template Name</label>
      </FloatLabel>
      <FloatLabel variant="in" class="flex-row form-field">
        <div class="flex-column">
          <InputNumber v-model="templateStore.template.intermission_duration" :min="1" suffix=" sec" />
        </div>
        <label for="intermission">Intermission Duration</label>
      </FloatLabel>
    </div>
    <div class="flex-row field-group">
      <div class="flex-row form-field toggle">
        <label class="field" for="feedback">Feedback:</label>
        <ToggleSwitch v-model="templateStore.template.feedback" inputId="feedback"/>
      </div>
      <FloatLabel v-if="templateStore.template.feedback" variant="in" class="flex-row form-field">
        <div class="flex-column">
          <InputNumber inputId="feedback_duration" v-model="templateStore.template.feedback_duration" :min="1" suffix=" sec" />
        </div>
        <label for="feedback_duration">Feedback Duration</label>
      </FloatLabel>
    </div>
    <div class="stations-container">
      <div class="flex-row stations-actions">
        <Button label="Add Station" icon="pi pi-plus" severity="secondary" @click="addStation" />
        <div class="flex-row toggle" style="align-self: center;">
          <label class="field" for="static">Static At End:</label>
          <ToggleSwitch v-model="templateStore.template.static_at_end" inputId="static"/>
        </div>
      </div>
      <DataTable :value="templateStore.stations" v-model:selection="selectedStations" selectionMode="multiple" :scrollable="true" @row-reorder="onRowReorder">
        <template #header>
          <div class="table-header">
            <Button v-if="selectedStations.length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" outlined @click="confirmDelete($event)" />
          </div>
        </template>
        <Column :rowReorder="true" headerStyle="width: 3rem"/>
        <Column field="index" header="Index">
          <template #body="{ data, index }">
            <label class="field">{{ index }}</label>
          </template>
        </Column>
        <Column field="title" header="Title">
          <template #body="{ data, index }">
            <InputText v-model="data.title" @update:modelValue="templateStore.setDirty" placeholder="Station title" fluid />
          </template>
        </Column>
        <Column field="duration" header="Duration (min)">
          <template #body="{ data, index }">
            <InputNumber v-model="data.duration" @update:modelValue="templateStore.setDirty" :min="1" placeholder="Minutes" suffix=" min" :useGrouping="false" />
          </template>
        </Column>
      </DataTable>
    </div>
    <div class="flex-row wizard-actions">
      <Button label="Back" icon="pi pi-arrow-left" @click="cancel()" />
      <Button label="Create" type="submit" @click="submitForm"/>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~~/composables/apiFetch";
import { useTemplateCreationStore } from "~~/stores/templateCreation";
import { useToast } from "primevue/usetoast";
import { useConfirm } from "primevue/useconfirm";

definePageMeta({
  layout: "default",
});

const templateStore = useTemplateCreationStore();
const selectedStations = ref([]);
const router = useRouter();
const confirm = useConfirm();
const toast = useToast();

const addStation = () => {
  templateStore.addStation();
};

const onRowReorder = (event) => {
  templateStore.onRowReorder(event);
};

const submitForm = async () => {
  let apiobj = {
    template_session: {
      ...templateStore.formatDuration(),
    },
    template_stations: templateStore.formatStations(),
  };
  if (!apiobj.template_session.feedback) {
    delete apiobj.template_session.feedback_duration;
  }
  console.log(apiobj);
  try {
    const response = await apiFetch("/templates/create", {
      method: "POST",
      body: apiobj,
    });
    templateStore.resetForm();
    router.push("/");
  } catch (error) {
    console.error("Submit error:", error);
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
      templateStore.removeStations(selectedStations.value)
      selectedStations.value = [];
    },
  });
}

const cancel = () => {
  if (templateStore.isDirty) {
    confirm.require({
        message: "You have unsaved changes. Are you sure you want to cancel and lose progress?",
        header: "Confirm",
        icon: "pi pi-exclamation-triangle",
        rejectProps: {
          label: "Cancel",
          severity: "secondary",
          outlined: true
        },
        acceptProps: {
          label: "Continue",
          severity: "danger"
        },
        accept: () => {
            templateStore.resetForm();
            router.push("/");
        }
    });
  } else {
    templateStore.resetForm();
    router.push("/");
  }
};

onBeforeMount(() => {
  window.onbeforeunload = () => {
    if (templateStore.isDirty) {
      return "You have unsaved changes. Are you sure you want to leave?";
    }
  };
});

onUnmounted(() => {
  window.onbeforeunload = null;
  if (!router.currentRoute.value.path.startsWith("/templates/new")) {
    templateStore.resetForm();
  }
});
</script>

<style scoped>
.subhead {
  font-size: 1.5rem;
  line-height: 2rem;
}

.table-header {
  display: flex;
  justify-content: flex-end;
  padding: 0.5rem;
}

.wizard-container {
  width: 50%;
  align-self: center;
  gap: 1rem;
  background-color: var(--p-surface-0);
  border-radius: var(--radius-m);
  padding: 2rem;
}

.wizard-actions {
  justify-content: space-between;
}

.field-group {
  gap: 1rem;
  justify-content: start;
}

.form-field {
  justify-content: center;
  align-items: center;
  width: 100%;
}

.stations-container {
  margin-top: 2rem;
}

.stations-actions {
  gap: 2rem;
  margin: 0.5rem 0 0.5rem 0;
}

.toggle {
  gap: 1rem;
}
</style>