<template>
  <div class="session-container flex-column text">
    <h1>Auto Allocation</h1>
    <div class="flex-row session-actions">
      <Button label="Allocate" icon="pi pi-code" severity="primary" @click="allocate" />
      <Button label="Back to Session" icon="pi pi-arrow-left" severity="primary" @click="navigateTo(`/sessions/${sessionId}`)" />
    </div>
    <ConfirmPopup class="text" /> 
  </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~/composables/apiFetch"
import { useToast } from "primevue/usetoast";
import { useConfirm } from "primevue/useconfirm";

definePageMeta({
  layout: "session",
});

const router = useRouter();
const toast = useToast();
const confirm = useConfirm();
const route = useRoute();
const sessionStore = useSessionStore();
const sessionId = computed(() => route.params.sessionID);

const refreshSessionID = () => {
  if (sessionId.value && (!sessionStore.session || sessionStore.session.id !== sessionId.value)) {
    sessionStore.fetchSession(sessionId.value);
  }
};

const navigateTo = (path) => {
  router.push(path);
};

const allocate = async () => {
  if (loading.value) return;
  loading.value = true;
  try {
    const response = await apiFetch(`/allocations/generate?id=${sessionId.value}`, {
        method: "GET",
    });
    toast.add({
      severity: "success",
      summary: "Success",
      detail: "Allocations generated successfully",
      life: 3000,
    });
  } catch (error) {
    toast.add({
      severity: "error",
      summary: "Error",
      detail: error.message || "Failed to allocate",
      life: 3000,
    });
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  refreshSessionID();
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