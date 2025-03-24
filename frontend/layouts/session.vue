<template>
  <div class="session-layout text">
    <Toast position="bottom-right" class="text" />
    <header class="header flex-row">
      <img src="/img/long_logo.png" alt="Logo" class="long-logo" />
      <div class="header-right">
        <Button
          label="Back to Home"
          icon="pi pi-arrow-left"
          variant="text"
          @click="navigateTo('/')"
        />
      </div>
    </header>
    <div class="content">
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
const router = useRouter();

const navigateTo = (path) => {
  router.push(path);
};

const formatDate = (dateString) => {
  return dateString ? new Date(dateString).toLocaleDateString() : "N/A";
};
</script>

<style>
.p-toast {
  z-index: 10001 !important;
}
</style>

<style scoped>
.session-layout {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  padding: 0;
  margin: 0;
}

.header {
  background-color: #f8f9fa;
  padding: 1rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  justify-content: space-between;
}

.long-logo {
  display: inline-flex;
  align-self: center;
  height: 2rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.session-info {
  font-size: 1rem;
  color: #333;
}

.content {
  flex: 1;
  padding: 1rem;
}
</style>