<script setup lang="ts">
  import {RouterLink} from 'vue-router'
</script>

<template>
  <div class="sm:w-1/2 lg:w-2/6 flex flex-col items-start py-24 sm:py-0">
    <h1 class="text-3xl xl:text-10xl font-text text-slate-900 font-bold leading-none mb-8"><RouterLink to="/">Area 403</RouterLink><span class="text-2xl xl:text-10xl">: About</span></h1>
    <p class="xl:text-xl tracking-wider text-slate-700 font-text font-medium">Area 403 is a one-person business</p>
    <p class="xl:text-xl tracking-wider text-slate-700 font-text font-medium">I am the founder, Matthieu Paindavoine</p>
    <p class="xl:text-xl tracking-wider text-slate-700 font-text font-medium">My goal is to help customers with fullstack development: Together, we'll analyze your requirements and find solutions.</p>
    <p class="xl:text-xl tracking-wider text-slate-700 font-text font-medium">My experience revolves around the following technological stack:</p>
    <ul class="xl:text-xl tracking-wider text-slate-700 font-text font-medium">
      <li>Database: Postgresql</li>
      <li>Backend: Rust (tokio, axum, sqlx, &hellip;)</li>
      <li>Frontend: Vue 3 (typescript) and Tailwindcss</li>
      <li>Deployment: Docker, Kubernetes</li>
      <li>Monitoring: Opentelemetry</li>
    </ul>
  </div>
</template>
