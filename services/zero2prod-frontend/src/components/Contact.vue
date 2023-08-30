<script setup lang="ts">
  import {RouterLink} from 'vue-router'
</script>

<template>
  <div class="sm:w-1/2 lg:w-2/6 flex flex-col items-start py-24 sm:py-0">
    <h1 class="text-3xl xl:text-10xl font-text text-slate-900 font-bold leading-none mb-8"><RouterLink to="/">Area 403</RouterLink><span class="text-2xl xl:text-10xl">: Contact</span></h1>
    <p class="xl:text-xl tracking-wider text-slate-700 font-text font-medium"><i class="icon-regular-envelope mr-2" aria-hidden="true"></i>info@area403.io</p>
    <p class="xl:text-xl tracking-wider text-slate-700 font-text font-medium mr-2"><i class="icon-brand-github mr-2" aria-hidden="true"></i>crocme10</p>
  </div>
</template>
