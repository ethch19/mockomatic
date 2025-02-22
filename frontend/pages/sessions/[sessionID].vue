<template>
  <div class="session-page text">
    <h1>Session Details</h1>
    <Card>
      <template #title>
        {{ sessionStore.session?.name || 'Loading...' }}
      </template>
      <template #content>
        <div v-if="sessionStore.loading" class="loading">
          <ProgressSpinner />
        </div>
        <div v-else-if="sessionStore.error" class="error">
          <Message severity="error" :closable="false">
            {{ sessionStore.error }}
          </Message>
        </div>
        <div v-else class="session-details">
          <p><strong>ID:</strong> {{ sessionStore.session?.id }}</p>
          <p><strong>Scheduled Date:</strong> {{ formatDate(sessionStore.session?.scheduled_date) }}</p>
          <p><strong>Created At:</strong> {{ formatDate(sessionStore.session?.created_at) }}</p>
        </div>
      </template>
    </Card>

    <div class="actions" v-if="!sessionStore.loading && !sessionStore.error">
      <Button
        label="Edit Session"
        icon="pi pi-pencil"
        class="p-button-primary"
        @click="navigateTo(`/sessions/${sessionId}`)"
      />
      <Button
        label="Upload Examiners"
        icon="pi pi-upload"
        class="p-button-secondary"
        @click="navigateTo(`/spreadsheet`)"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from 'vue-router';
import { useSessionStore } from '~/stores/session';

definePageMeta({
  layout: 'session',
});

// Router and route
const router = useRouter();
const route = useRoute();
const sessionId = route.params.sessionID;
const sessionStore = useSessionStore();

const navigateTo = (path) => {
  router.push(path);
};

const formatDate = (dateString) => {
  return dateString ? new Date(dateString).toLocaleString() : 'N/A';
};
</script>

<style scoped>
.session-page {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
}

.session-details p {
  margin: 0.5rem 0;
}

.actions {
  margin-top: 2rem;
  display: flex;
  gap: 1rem;
}

.loading {
  display: flex;
  justify-content: center;
  padding: 2rem;
}

.error {
  padding: 1rem;
}
</style>