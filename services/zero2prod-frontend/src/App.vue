<script setup lang="ts">
  import {RouterView } from 'vue-router'
</script>

<template>
  <main class="bg-white font-text relative h-screen">
    <div class="container h-screen mx-auto px-6 sm:px-12 flex flex-col sm:flex-row items-center relative z-10">
      <RouterView class="sm:w-1/2 lg:w-2/6"/>
      <div class="hidden sm:flex items-center justify-center xl:justify-end w-1/2 lg:w-4/6 py-8">
        <!-- Image based on Photo by <a href="https://unsplash.com/@davidclode?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText">David Clode</a> on <a href="https://unsplash.com/photos/8KbzBHA3ENc?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText">Unsplash</a> -->
        <img src="/tree-1.png" class="h-full w-4/5"/>
      </div>
    </div>
</main>
</template>
