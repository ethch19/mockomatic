<template>
  <div class="session-layout text">
    <Toolbar class="header">
      <template #start>
        <img src="/img/long_logo.png" alt="Logo" class="long-logo" />
      </template>
      <template #end>
        <div class="header-right">
          <Button
            label="Back to Home"
            icon="pi pi-arrow-left"
            class="p-button-text"
            @click="navigateTo('/')"
          />
          <span class="session-info">
            Session ID: {{ sessionStore.session?.id || "Loading..." }}
          </span>
          <span class="session-info">
            Scheduled: {{ formatDate(sessionStore.session?.scheduled_date) }}
          </span>
        </div>
      </template>
    </Toolbar>
    
    <div class="content">
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from "vue-router";
import { ref, onMounted } from "vue";
import { useSessionStore } from "~/stores/session";

const router = useRouter();
const route = useRoute();
const sessionStore = useSessionStore();

const navigateTo = (path) => {
  router.push(path);
};

const formatDate = (dateString) => {
  return dateString ? new Date(dateString).toLocaleDateString() : "N/A";
};

onMounted(() => {
  const sessionId = route.params.sessionID;
  console.log("Session Layout")
  console.log(sessionId)
  if (sessionId && (!sessionStore.session || sessionStore.session.id !== sessionId)) {
    sessionStore.fetchSession(sessionId);
  }
});
</script>

<style scoped>
.session-layout {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.header {
  background-color: #f8f9fa;
  padding: 1rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.long-logo {
  display: inline-flex;
  align-self: center;
  height: 2rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 1.5rem; /* Space between elements */
}

.session-info {
  font-size: 1rem;
  color: #333;
}

.content {
  flex: 1;
  padding: 2rem;
}
</style>