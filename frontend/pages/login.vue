<template>
    <div class="flex-column justify-center content-center">
        <ColorScheme placeholder="..." tag="span">
            <img v-if="$colorMode.value === 'light'" src="/public/img/long_logo.png" class="long-logo" alt="Logo">
            <img v-if="$colorMode.value === 'dark'" src="/public/img/long_logo_dark.png" class="long-logo" alt="Logo">
        </ColorScheme>
        <form class="login-form flex-column text p-fluid" @submit="handleLogin">
            <h2 class="subtitle">Sign In</h2>
            <FormField v-slot="{ componentField }" name="username">
                <FormItem>
                    <FormLabel>Username</FormLabel>
                    <FormControl>
                        <Input type="text" v-bind="componentField" ></Input>
                    </FormControl>
                    <FormMessage />
                </FormItem>
            </FormField>
            <FormField v-slot="{ componentField }" name="password">
                <FormItem>
                    <FormLabel>Password</FormLabel>
                    <FormControl>
                        <Input type="password" v-bind="componentField" ></Input>
                    </FormControl>
                    <FormMessage />
                </FormItem>
            </FormField>
            <Button type="submit" variant="default" class="login-btn flex-row">
                <iconify-icon v-show="loading" icon="svg-spinners:180-ring" height="1rem" ></iconify-icon>
                <iconify-icon v-show="!loading" icon="lucide:log-in" height="1rem" ></iconify-icon>
                Login
            </Button>
        </form>
        <Button variant="ghost" size="icon" class="flex-column colour-mode-btn" @click="changeColourMode" :aria-label="`Switch to ${$colorMode.value === 'light' ? 'dark' : 'light'} mode`">
            <iconify-icon v-if="$colorMode.value === 'light'" icon="lucide:sun" height="1.75rem" width="1.75rem" style="color: var(--foreground)"></iconify-icon>
            <iconify-icon v-if="$colorMode.value === 'dark'" icon="lucide:moon-star" height="1.75rem" width="1.75rem" style="color: var(--foreground)"></iconify-icon>
        </Button>
    </div>
</template>

<script lang="ts" setup>
definePageMeta({
    layout: false,
})

import type  { AuthBody } from "~/stores/auth";
import * as z from "zod";
import { useForm } from 'vee-validate';
import { toTypedSchema } from '@vee-validate/zod';
import { toast } from "vue-sonner";
import { useAuthStore } from "~/stores/auth";
import "iconify-icon";

const loading = ref(false)
const router = useRouter()
const colorMode = useColorMode()

const { values, handleSubmit} = useForm({
    validationSchema: toTypedSchema(
        z.object({
            username: z.string().nonempty("Username cannot be empty"),
            password: z.string().nonempty("Password cannot be empty"),
        }),
    ),
});

const changeColourMode = () => {
    if (colorMode.value === "light") {
        colorMode.value = "dark"
    } else {
        colorMode.value = "light"
    }
    console.log("Colour mode value: {}", colorMode.value);
    console.log("Colour mode preference: {}", colorMode.preference);
}

const handleLogin = handleSubmit(async values => {
    loading.value = true
    try {
        const authStore = useAuthStore();
        const data: AuthBody = await apiFetch("/users/login", {
            method: "POST",
            body: JSON.stringify({ username: values.username, password: values.password }),
            headers: {
                "origin": "http://localhost:3000",
            }
        })
        authStore.setAuth(data);
        router.push("/");
    } catch (err) {
        toast.error(err.data);
    } finally {
        loading.value = false
    }
})
</script>

<style scoped>
.long-logo {
    display: inline-flex;
    align-self: center;
    min-width: 15vw;
    width: 25rem;
    max-height: 100%;
}

.login-form {
    min-height: 30vh;
    min-width: 20vw;
    width: 25rem;
    background-color: var(--background-2);
    border-radius: 1rem;
    justify-content: space-between;
    align-self: center;
    align-items: stretch;
    gap: 1.5rem;
    padding: 3rem;
}

.subtitle {
    margin: 0 0 1rem 0;
}

.login-btn {
    width: 40%;
    min-width: 8rem;
    margin-top: 1.5rem;
    align-self: center;
}

.colour-mode-btn {
    position: absolute;
    top: 2rem;
    right: 2rem;
}

.colour-mode-btn:hover {
    background-color: var(--border);
}
</style>