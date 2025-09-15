<template>
    <div class="h-full flex-column justify-center content-center">
        <NuxtLayout>
            <NuxtPage />
        </NuxtLayout>
        <Toaster position="bottom-right" :expand="true"/>
    </div>
</template>

<style>
#__nuxt {
    height: 100%;
}
</style>