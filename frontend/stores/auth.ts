import { defineStore } from "pinia"
import { apiFetch } from "~/composables/apiFetch"

export const useAuthStore = defineStore("auth", {
  state: () => ({
    accessToken: null,
  }),
  actions: {
    setAccessToken(token) {
      this.accessToken = token
    },
    clearAccessToken() {
      this.accessToken = null
    },
    async refreshTokens() {
      try {
        const response = await apiFetch("/users/refresh", {
          method: "GET",
        });
        this.setAccessToken(response.access_token);
        return true;
      } catch (error) {
        console.error("Token refresh failed:", error);
        this.clearAccessToken();
        return false;
      }
    },
  },
})
