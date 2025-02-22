<template>
  <div class="upload-container flex-column">
      <FileUpload
        ref="fileUpload"
        name="file"
        accept=".xlsx"
        :maxFileSize="10000000"
        :customUpload="true"
        :multiple="false"
        @uploader="onUpload"
        chooseLabel="Select XLSX File"
        uploadLabel="Upload"
        cancelLabel="Cancel"
      >
        <template #empty>
          <p>Drag and drop an XLSX file here to upload.</p>
        </template>
      </FileUpload>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useToast } from 'primevue/usetoast';
import { apiFetch } from '~/composables/apiFetch'
import { useSessionStore } from '~/stores/session';

const fileUpload = ref(null);
const toast = useToast();
const sessionId = route.params.sessionID;
const sessionStore = useSessionStore();

const onUpload = async (event) => {
  const file = event.files[0];
  if (!file) {
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: 'No file selected',
      life: 3000
    });
    return;
  }

  const formData = new FormData();
  formData.append('file', file);

  try {
    const response = await apiFetch('/people/examiners/upload-xlsx?id=${sessionId}', {
      method: 'POST',
      body: formData,
      headers: {
        'Content-Type': 'multipart/form-data'
      },
    });

    toast.add({
      severity: 'success',
      summary: 'Success',
      detail: 'File uploaded successfully',
      life: 3000
    });
    
    fileUpload.value.clear();
  } catch (error) {
    toast.add({
      severity: 'error',
      summary: 'Upload Failed',
      detail: error.message || 'An error occurred during upload',
      life: 3000
    });
  }
};

onMounted(() => {
  const sessionId = route.params.sessionID;
  if (sessionId && (!sessionStore.session || sessionStore.session.id !== sessionId)) {
    sessionStore.fetchSession(sessionId);
  }
});
</script>

<style scoped>
.upload-container {
  justify-content: center;
  align-items: center;
}
</style>