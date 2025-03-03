<template>
  <Form class="login-form flex-column text p-fluid" v-slot="$form" :resolver="resolver" :initialValues="initialValues" @submit="handleLogin">
    <h2 class="subtitle">Sign In</h2>
    <div>
      <FloatLabel variant="on">
        <InputText :class="{ 'p-invalid': error }" :style="{ width: '100%' }" :inputStyle="{ width: '100%', 'padding': '1rem' }" id="username" v-model="username" type="text"/>
        <label for="username">Username</label>
      </FloatLabel>
    </div>
    <div>
      <FloatLabel variant="on">
        <Password :class="{ 'p-invalid': error }" :style="{ width: '100%' }" :inputStyle="{ width: '100%', 'padding': '1rem' }" id="password" v-model="password" :feedback="false"/>
        <label for="password">Password</label>
      </FloatLabel>
    </div>
    <Button type="submit" severity="secondary" label="Login" />
    <p v-if="error" class="error-message">{{ error }}</p>
  </Form>
</template>

<script lang="ts" setup>
definePageMeta({
  layout: "default",
})

import { ref } from "vue"
import { useRouter } from "vue-router"
import { useAuthStore } from "~/stores/auth"
import { apiFetch } from "~/composables/apiFetch"

const username = ref("")
const password = ref("")
const loading = ref(false)
const error = ref(null)
const router = useRouter()
const authStore = useAuthStore()

async function handleLogin() {
  loading.value = true
  error.value = null
  try {
    const data = await apiFetch("/users/login", {
      method: "POST",
      body: JSON.stringify({ username: username.value, password: password.value }),
      headers: {
        "origin": "http://localhost:3000",
      }
    })
    if (data.access_token && data.token_type === "Bearer ") {
      authStore.setAccessToken(data.access_token)
      router.push("/")
    } else {
      error.value = "Login failed: Invalid response from server"
      console.error("Unexpected response:", data)
    }
  } catch (err) {
    console.error("Login error:", err)
    console.log("Error response:", err.response)
    if (err.response) {
      const errorMessage = typeof err.response._data === "string" ? err.response._data : err.response._data?.detail || "Unknown server error"
      error.value = `Login failed: ${errorMessage}`
    } else {
      error.value = `Login failed: ${err.message || "Network error"}`
    }
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-form {
  height:fit-content;
  min-width: 20vw;
  max-width: 30vw;
  background-color: var(--p-surface-50);
  box-shadow: 0 4px 8px var(--p-surface-300);
  border-radius: 1rem;
  align-items: stretch;
  margin: auto;
  gap: 1rem;
  padding: 2rem;
}

.subtitle {
  margin: 0 0 1rem 0;
}
</style>