<template>
  <header id="header" class="bg-gray-700">
    <nav class="container mx-auto flex justify-start items-center py-5 px-4">
      <!-- App Name -->
      <router-link class="text-white font-bold uppercase text-2xl mr-4"
        :to="{ name: 'Home' }"
        exact-active-class="no-active">Music</router-link>

      <div class="flex flex-grow items-center">
        <!-- Primary Navigation -->
        <ul class="flex flex-row mt-1">
          <!-- Navigation Links -->
          <li v-if="!isLoggedIn">
            <a class="px-2 text-white" href="#" @click.prevent="toggleHidden">Login / Register</a>
          </li>
          <template v-else>
            <li>
              <a class="px-2 text-white" href="#" @click.prevent="logout">Logout</a>
            </li>
            <li>
              <router-link class="px-2 text-white"
                :to="{ name: 'Manage' }">Manage</router-link>
            </li>
          </template>
          <li>
            <router-link class="px-2 text-white" 
              :to="{ name: 'About'}">About</router-link>
          </li>
        </ul>
      </div>
    </nav>
  </header>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useModalStore } from '../stores/modal'
import { useAuthStore } from '../stores/Auth'
import { storeToRefs } from 'pinia'
import { useRouter, useRoute } from 'vue-router'

export default defineComponent({
  setup() {
    const modalStore = useModalStore()
    const toggleHidden = modalStore.toggleHidden

    const authStore = useAuthStore()
    const { isLoggedIn } = storeToRefs(authStore)
    const authLogout = authStore.logout

    const router = useRouter()
    const route = useRoute()

    const logout = () => {
      console.log('Logging out')
      authLogout()
      if (route.meta?.requiresAuth) {
         router.push({name: 'Home'})
      }
    }

    return {
      toggleHidden,
      isLoggedIn,
      logout
    }
  }
})
</script>
