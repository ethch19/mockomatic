<template>
  <div>
    <h1>Test Page</h1>
    <p v-if="loading">Loading...</p>
    <p v-else-if="error">{{ error }}</p>
    <pre v-else>{{ data }}</pre>
  </div>
</template>

<script lang="ts" setup>
import { ref, inject } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '~/stores/auth'

const data = ref(null)
const loading = ref(true)
const error = ref(null)
const router = useRouter()
const authStore = useAuthStore()

async function fetchData() {
  try {
    data.value = await apiFetch('/sessions/test')
  } catch (err) {
    error.value = 'Failed to load data: ' + (err.response?._data?.detail || 'Unknown error')
    if (err.response?.status === 401) {
      // Token expired or invalid, redirect to login
      authStore.clearAccessToken()
      router.push('/login')
    }
  } finally {
    loading.value = false
  }
}

async function logout() {
  authStore.clearAccessToken()
  router.push('/login')
}

fetchData()
</script>

<style>

</style>