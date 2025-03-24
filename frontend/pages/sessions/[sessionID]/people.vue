<template>
  <div class="session-container flex-column text">
    <h1>People Details</h1>
    <div class="flex-row session-actions">
      <SelectButton v-model="people_selection" :options="people_options" />
      <Button label="Back to Session" icon="pi pi-arrow-left" severity="primary" @click="navigateTo(`/sessions/${sessionId}`)" />
    </div>
    <DataTable
      v-if="people_selection=='Candidates'"
      :value="candidates"
      :loading="loading"
      :scrollable="true"
      v-model:selection="selectedCandidates"
      selectionMode="multiple"
      dataKey="id"
    >
      <template #header>
        <div class="table-header">
          <Button v-if="selectedCandidates.length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" @click="confirmDeleteCandidate" />
        </div>
      </template>
      <Column selectionMode="multiple" headerStyle="width: 3rem" />
      <Column field="first_name" header="First Name">
        <template #body="{ data }"> {{ data.first_name }} </template>
      </Column>
      <Column field="last_name" header="Last Name">
        <template #body="{ data }"> {{ data.last_name }} </template>
      </Column>
      <Column field="shortcode" header="Shortcode">
        <template #body="{ data }"> {{ data.shortcode }} </template>
      </Column>
      <Column field="female_only" header="Female Only">
        <template #body="{ data }"> {{ data.female_only ? "Yes" : "No" }} </template>
      </Column>
      <Column field="partner_pref" header="Partner Preference">
        <template #body="{ data }"> {{ data.partner_pref ?? "None" }} </template>
      </Column>
      <ColumnGroup type="footer">
        <Row>
            <Column footer="Total:" :colspan="2" footerStyle="text-align:right" />
            <Column :footer="totalCount('Candidate')" />
        </Row>
      </ColumnGroup>
    </DataTable>
    <DataTable
      v-if="people_selection=='Examiners'"
      :value="examiners"
      :loading="loading"
      :scrollable="true"
      v-model:selection="selectedExaminers"
      selectionMode="multiple"
      dataKey="id"
    >
      <template #header>
        <div class="table-header">
          <Button v-if="selectedExaminers.length > 0" label="Delete Selected" icon="pi pi-trash" severity="danger" @click="confirmDeleteExaminer" />
        </div>
      </template>
      <ColumnGroup type="header">
        <Row>
          <Column selectionMode="multiple" headerStyle="width: 3rem" :rowspan="2" />
          <Column header="First Name" :rowspan="2" />
          <Column header="Last Name" :rowspan="2" />
          <Column header="Shortcode" :rowspan="2" />
          <Column header="Female Only" :rowspan="2"/>
          <Column header="Availability" :colspan="2" />
        </Row>
        <Row>
          <Column header="AM" field="am" />
          <Column header="PM" field="pm" />
        </Row>
      </ColumnGroup>
      <Column selectionMode="multiple" />
      <Column field="first_name">
        <template #body="{ data }"> {{ data.first_name }} </template>
      </Column>
      <Column field="last_name">
        <template #body="{ data }"> {{ data.last_name }} </template>
      </Column>
      <Column field="shortcode">
        <template #body="{ data }"> {{ data.shortcode }} </template>
      </Column>
      <Column field="female">
        <template #body="{ data }"> {{ data.female ? "Yes" : "No" }} </template>
      </Column>
      <Column field="am">
        <template #body="{ data }"> {{ data.am ? "Yes" : "No" }} </template>
      </Column>
      <Column field="pm">
        <template #body="{ data }"> {{ data.pm ? "Yes" : "No" }} </template>
      </Column>
      <ColumnGroup type="footer">
        <Row>
            <Column footer="Total:" :colspan="2" footerStyle="text-align:right" />
            <Column :footer="totalCount('Examiner')" />
        </Row>
      </ColumnGroup>
    </DataTable>
    <ConfirmPopup class="text" /> 
  </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~/composables/apiFetch"
import { useToast } from "primevue/usetoast";
import { useConfirm } from "primevue/useconfirm";
import { ColumnGroup } from "primevue";

definePageMeta({
  layout: "session",
});

const router = useRouter();
const toast = useToast();
const confirm = useConfirm();
const route = useRoute();
const sessionStore = useSessionStore();

const people_selection = ref("Candidates");
const people_options = ref(["Candidates", "Examiners"]);
const candidates = ref([]);
const examiners = ref([]);
const loading = ref(false);
const selectedCandidates = ref([]);
const selectedExaminers = ref([]);
const sessionId = computed(() => route.params.sessionID);

const totalCount = (people_type) => {
  if (people_type == "Examiner") {
    return examiners.value.length;
  } else {
    return candidates.value.length;
  }
};

const refreshSessionID = () => {
  if (sessionId.value && (!sessionStore.session || sessionStore.session.id !== sessionId.value)) {
    sessionStore.fetchSession(sessionId.value);
  }
};

const loadPeople = async () => {
  if (loading.value) return;
  loading.value = true;
  try {
    const cand_response = await apiFetch(`/candidates/get-session-all?id=${sessionId.value}`, {
        method: "GET",
    });
    candidates.value = cand_response;
    console.log(cand_response);
    const exam_response = await apiFetch(`/examiners/get-session-all?id=${sessionId.value}`, {
        method: "GET",
    });
    examiners.value = exam_response;
    console.log(exam_response);
  } catch (error) {
    toast.add({
      severity: "error",
      summary: "Error",
      detail: error.message || "Failed to load people",
      life: 3000,
    });
  } finally {
    loading.value = false;
  }
};

const confirmDeleteCandidate = () => {
  confirm.require({
    message: `Are you sure you want to delete ${selectedCandidates.value.length} candidate(s)?`,
    header: "Confirm Deletion",
    icon: "pi pi-exclamation-triangle",
    accept: async () => {
      try {
        await apiFetch("/candidates/delete", {
          method: "POST",
          body: { ids: selectedCandidates.value.map(s => s.id) },
        });
        toast.add({
          severity: "success",
          summary: "Success",
          detail: "Candidates deleted successfully",
          life: 3000,
        });
        selectedCandidates.value = [];
        loadPeople();
      } catch (error) {
        toast.add({
          severity: "error",
          summary: "Error",
          detail: "Failed to delete candidates",
          life: 3000,
        });
      }
    },
  });
}

const confirmDeleteExaminer = () => {
  confirm.require({
    message: `Are you sure you want to delete ${selectedExaminers.value.length} examiner(s)?`,
    header: "Confirm Deletion",
    icon: "pi pi-exclamation-triangle",
    accept: async () => {
      try {
        await apiFetch("/examiners/delete", {
          method: "POST",
          body: { ids: selectedExaminers.value.map(s => s.id) },
        });
        toast.add({
          severity: "success",
          summary: "Success",
          detail: "Examiners deleted successfully",
          life: 3000,
        });
        selectedExaminers.value = [];
        loadPeople();
      } catch (error) {
        toast.add({
          severity: "error",
          summary: "Error",
          detail: "Failed to delete examiners",
          life: 3000,
        });
      }
    },
  });
}

const navigateTo = (path) => {
  router.push(path);
};

onMounted(() => {
  refreshSessionID();
  loadPeople();
});

watch(
  sessionId,
  (newSessionId) => {
    refreshSessionID();
  }
);
</script>

<style scoped>
.session-actions{
  justify-content: space-between;
  margin: 1rem;
}
</style>