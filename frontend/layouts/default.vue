<template>
  <div class="header-container flex-column">
    <header class="flex-column">
        <img class="long-logo" src="/img/long_logo.svg" alt="Logo">
    </header>
    <slot class="slot" />
  </div>
</template>

<script lang="ts" setup>

</script>

<style>
#__nuxt {
  height: 100vh;
}
</style>

<style scoped>
.header-container {
  height: 100%;
  width: 100%;
  align-items: center;
}

header {
  height: 6rem;
  width: 100%;
  margin-top: 3rem;
}

.long-logo {
  display: inline-flex;
  align-self: center;
  height: 6rem;
}

.title {
  text-align: center;
}
</style>