<template>
    <div class="flex-column justify-start py-[1rem] px-[3rem] text h-full">
        <div class="flex-column justify-start items-start">
            <h1 class="subtitle">Account Settings</h1>
            <div class="flex-column gap-[1rem] py-[1rem]">
                <Avatar class="self-start h-[5rem] w-auto">
                    <AvatarImage src="https://github.com/unovue.png" :alt="`@${authStore.username}`" />
                    <AvatarFallback>MK</AvatarFallback>
                </Avatar>
                <p class="subhead">Organisation: <span class="text">{{ authStore.organisation }}</span></p>
                <div class="flex-row gap-[1rem]">
                    <Label class="subhead" for="username">New Username:</Label>
                    <Input class="w-[15rem]" id="username">{{ authStore.username }}</Input>
                    <Button @click="changeUsername">Save</Button>
                </div>
                <Separator />
                <div class="flex-column gap-[1rem]">
                    <div class="flex-row gap-[1rem] justify-between">
                        <Label class="subhead" for="old_password">Old Password:</Label>
                        <Input class="w-[15rem]" id="old_password" type="password"></Input>
                    </div>
                    <div class="flex-row gap-[1rem] justify-between">
                        <Label class="subhead" for="new_password">New Password:</Label>
                        <Input class="w-[15rem]" id="new_password" type="password"></Input>
                    </div>
                    <div class="flex-row gap-[1rem] justify-between">
                        <Label class="subhead" for="repeat_password">Repeat Password:</Label>
                        <Input class="w-[15rem]" id="repeat_password" type="password"></Input>
                    </div>
                    <Button @click="changePassword">Save</Button>
                </div>
            </div>
        </div>
    </div>
</template>

<script lang="ts" setup>
import { apiFetch } from "~~/composables/apiFetch"
import { useAuthStore } from "~/stores/auth";
import { Separator } from "~/components/ui/separator";
import { toast } from "vue-sonner";

const authStore = useAuthStore();
const router = useRouter();

const new_password = ref("");
const repeat_password = ref("");
const old_password = ref("");
const new_username = ref("");


const changePassword = async () => {
    if (new_password.value !== repeat_password.value) {
        toast.error("Passwords do not match");
        return;
    }
    try {
        await apiFetch("/users/change-password", {
            method: "POST",
            body: {
                old_password: old_password.value,
                new_password: new_password.value,
            }
        });
        toast.success("Password changed successfully to " + new_password.value);
        old_password.value = "";
        new_password.value = "";
        repeat_password.value = "";
        // -- FEATURE --
        // requires new access token/claims, no need to log in again
        // response should include new AuthBody
    } catch (err) {
        toast.error("Failed to change password: " + err.data);
    }
};

const changeUsername = async () => {
    try {
        await apiFetch("/users/change-username", {
            method: "POST",
            body: {
                new_username: new_username.value,
            }
        });
        toast.success("Username changed successfully to " + new_username.value);
        new_username.value = "";
        // -- FEATURE --
        // requires new access token/claims, no need to log in again
        // response should include new AuthBody
    } catch (err) {
        toast.error("Failed to change username: " + err.data);
    }
};
</script>