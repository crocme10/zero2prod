<script setup lang="ts">
  import {RouterLink} from 'vue-router'
</script>

<template>
  <div class="flex flex-col sm:flex-row items-center relative">
    <div class="w-1/2 flex flex-col items-center py-24 sm:py-0">
      <h1 class="text-6xl xl:text-10xl font-text text-slate-900 font-bold leading-none mb-8">Area 403</h1>
      <p class="xl:text-xl tracking-wider text-slate-700 font-text font-bold">Software Consulting Company</p>
      <!--<a href="#" class="font-text text-white sm:font-xl uppercase py-3 px-6 sm:py-4 sm:px-8 rounded-full shadow-lg bg-slate-900 hover:bg-slate-800 mt-8 sm:mt-16">Coming Soon</a>-->
    </div>
    <div class="w-1/2 flex flex-col items-start pl-24 py-24 sm:py-0">
      <RouterLink to="/about" class="hidden xl:block font-text font-bold uppercase text-gray-800 tracking-wider px-4 xl:px-8 py-6 text-lg hover:underline">About</RouterLink>
      <RouterLink to="/about" class="hidden xl:block font-text font-bold uppercase text-gray-800 tracking-wider px-4 xl:px-8 py-6 text-lg hover:underline">Projects</RouterLink>
      <RouterLink to="/contact" class="hidden xl:block font-text font-bold uppercase text-gray-800 tracking-wider px-4 xl:px-8 py-6 text-lg hover:underline">Contact</RouterLink>
    </div>
  </div>
</template>
