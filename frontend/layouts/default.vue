<template>
  <div class="header-container flex-column">
    <header class="flex-column">
        <img class="long-logo" src="/img/long_logo.png" alt="Logo">
    </header>
    <div class="flex-column slot-container">
      <slot />
    </div>
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

.slot-container {
  width: 100%;
  height: 100%;
}

.title {
  text-align: center;
}
</style>