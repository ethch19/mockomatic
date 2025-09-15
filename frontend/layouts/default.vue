<template>
    <div class="session-layout text">
        <header class="header flex-row">
            <NuxtLink to="/">
                <ColorScheme placeholder="..." tag="span">
                    <img v-if="$colorMode.value === 'light'" src="/public/img/long_logo.png" class="long-logo" alt="Logo">
                    <img v-if="$colorMode.value === 'dark'" src="/public/img/long_logo_dark.png" class="long-logo" alt="Logo">
                </ColorScheme>
            </NuxtLink>
            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <Button class="justify-center content-center h-full" variant="ghost">
                        <div class="flex-row justify-center content-center gap-1 h-full">
                            <span class="justify-center content-center flex-column">
                                <Label class="head text-end self-end">{{ authStore.username }}</Label>
                                <Label class="subhead text-(--text-2) capitalize text-end self-end">{{ authStore.role }}</Label>
                            </span>
                            <Avatar class="self-center h-full w-auto">
                                <AvatarImage src="https://github.com/unovue.png" :alt="`@${authStore.username}`" />
                                <AvatarFallback>MK</AvatarFallback>
                            </Avatar>
                        </div>
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                    <DropdownMenuItem class="text" @click="navigate('/account')">
                        <iconify-icon class="text-(--foreground)" icon="lucide:settings" width="24" height="24"></iconify-icon>
                        Account Settings
                    </DropdownMenuItem>
                    <DropdownMenuItem class="text" @click="logout()">
                        <iconify-icon class="text-(--foreground)" icon="lucide:log-out" width="24" height="24"></iconify-icon>
                        Logout
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </header>
        <div class="content">
            <slot />
        </div>
    </div>
</template>

<script lang="ts" setup>
import { useAuthStore } from "~/stores/auth";

const authStore = useAuthStore();
const router = useRouter();

const logout = async () => {
    await authStore.logout();
    return navigate('/login')
};

const navigate = (path: string) => {
    return navigateTo(path)
};
</script>

<style scoped>
.session-layout {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    padding: 0;
    margin: 0;
}

.header {
    background-color: var(--background-2);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    justify-content: space-between;
    height: 5rem;
    padding: 0.5rem 4rem;
    border-bottom: 1px solid var(--border);
}

.long-logo {
    display: inline-flex;
    align-self: center;
    height: 100%;
}

.content {
    flex: 1;
    padding: 1rem;
}
</style>