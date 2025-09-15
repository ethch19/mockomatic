import { useAuthStore } from "~/stores/auth";
import { useNuxtApp } from "#app";
import { routeLocationKey } from "vue-router";

export default defineNuxtRouteMiddleware(async (to, from) => {
    if (import.meta.server) return;

    const { $pinia } = useNuxtApp();
    const authStore = useAuthStore($pinia);

    console.log(to.path, from.path);
    if (authStore.accessToken == null) {
        const result = await authStore.refreshTokens();
        console.log("Token refresh result:", result);
        if (!result && to.path !== "/login") {
            console.log("Redirecting to login");
            return navigateTo("/login");
        }
        return;
    }
    
    if (to.path === "/login") {
        return navigateTo("/");
    }
});