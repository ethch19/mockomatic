export const useAuthStore = defineStore("auth", {
  state: () => ({
    accessToken: null as string | null,
    token_type: null as string | null,
    username: null as string | null,
    role: null as string | null,
    organisation: null as string | null
  }),
  actions: {
    setAuth(authBody: AuthBody) {
      this.accessToken = authBody.access_token
      this.token_type = authBody.token_type
      this.username = authBody.username
      this.role = authBody.role.charAt(0).toUpperCase() + authBody.role.slice(1);
      this.organisation = authBody.organisation;
    },
    clearAuth() {
      this.accessToken = null
      this.token_type = null
      this.username = null
      this.role = null
      this.organisation = null
    },
    async logout() {
        try {
            const response = await apiFetch("/users/logout", { method: "GET"});
            this.clearAuth();
            return true;
        } catch (error) {
            this.clearAuth();
            return false;
        }
    },
    async refreshTokens() {
      try {
        const response: AuthBody = await apiFetch("/users/refresh", { method: "GET" });
        this.setAuth(response);
        return true;
      } catch (error) {
        this.clearAuth();
        return false;
      }
    },
  },
})
