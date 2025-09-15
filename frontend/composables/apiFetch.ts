import { $fetch } from "ofetch"
import { useAuthStore } from "~/stores/auth"
import { useRuntimeConfig } from "#app"
import { useRouter } from "vue-router";

export const apiFetch = (url: string, options = {}) => {
    const nuxtApp = useNuxtApp();
    const authStore = useAuthStore()
    const router = useRouter();
    const csrf_cookie = useCookie("csrf_token");
    const config = useRuntimeConfig();
    const apiBase = config.public.apiBase;
    const headers = {
        ...options.headers,
    }

    if (authStore.accessToken) {
        headers["Authorization"] = `Bearer ${authStore.accessToken}`
    }

    if (csrf_cookie.value) {
        headers["x-csrf-token"] = csrf_cookie.value;
    }

    return $fetch(`${apiBase}${url}`, {
        ...options,
        headers,
        credentials: "include",
    }).catch(err => {
        throw err;
    });
}