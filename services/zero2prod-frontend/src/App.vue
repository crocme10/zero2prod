<template>
  <AppHeader />

  <router-view></router-view>

  <AppAuth />
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import AppHeader from './components/AppHeader.vue'
import AppAuth from './components/AppAuth.vue'
import { storeToRefs } from 'pinia'
import { useAuthStore } from './stores/Auth'

export default defineComponent({
  components: {
    AppHeader,
    AppAuth
  },
  setup() {
    const store = useAuthStore()
    const { isLoggedIn } = storeToRefs(store)

    onMounted(async () => {
      await store.authenticate()
    })

    return {
      isLoggedIn
    }
  }
})
</script>
