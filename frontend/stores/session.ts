import { defineStore } from "pinia"
import { apiFetch } from "~/composables/apiFetch"

export const useSessionStore = defineStore("session", {
  state: () => ({
    session: null,
    loading: false,
    error: null,
  }),
  actions: {
    async fetchSession(sessionId) {
      this.loading = true;
      this.error = null;
      try {
        const response = await apiFetch("/sessions/get", {
            method: "POST",
            body: JSON.stringify({ id: sessionId })
        });
        this.session = response;
        console.log(response)
      } catch (error) {
        this.error = err.message || "Failed to fetch session";
        this.session = null;
      } finally {
        this.loading = false;
      }
    },
    clearSession() {
      this.session = null;
      this.loading = false;
      this.error = null;
    },
  },
});