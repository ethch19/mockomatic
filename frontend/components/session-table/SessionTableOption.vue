<template>
    <DropdownMenu>
        <DropdownMenuTrigger as-child>
        <Button variant="ghost" class="w-8 h-8 p-0">
            <span class="sr-only">Open menu</span>
            <iconify-icon icon="lucide:more-horizontal" width="24" height="24"></iconify-icon>
        </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
            <DropdownMenuItem @click="navigateTo('/sessions/'+session.id)" class="flex-row justify-left gap-2">
                <iconify-icon icon="lucide:pencil-line" width="20" height="20"></iconify-icon>
                Edit
            </DropdownMenuItem>
            <DropdownMenuItem @click="delete_session" class="flex-row justify-left gap-2">
                <iconify-icon icon="lucide:trash-2" width="20" height="20"></iconify-icon>
                Delete
            </DropdownMenuItem>
            <DropdownMenuSeparator/>
            <DropdownMenuItem @click="copy_id(session.id)" class="flex-row justify-left gap-2">
                <iconify-icon icon="lucide:copy" width="20" height="20"></iconify-icon>
                Copy ID
            </DropdownMenuItem>
        </DropdownMenuContent>
    </DropdownMenu>
</template>

<script setup lang="ts">
import { useSessionBrowserStore } from '~/stores/sessionBrowser';

const props = defineProps<{
    session: ISession
}>();

const sessionBrowserStore = useSessionBrowserStore();

const copy_id = (id: string) => {
    navigator.clipboard.writeText(id)
}

const delete_session = async () => {
    await sessionBrowserStore.deleteSession(props.session.id);
}
</script>