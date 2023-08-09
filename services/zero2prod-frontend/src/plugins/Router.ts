import router from '../router'
import { useAuthStore } from '../stores/Auth'

const RouterInterceptor = {
  install() {
    router.beforeEach(async (to) => {
      const authStore = useAuthStore()
      const loggedIn = authStore.isLoggedIn
      const publicPages = ['Login', 'Register', 'Home']
      const authRequired = !publicPages.includes(to.name as string)
      // FIXME Should probably use authStore

      // trying to access a restricted page + not logged in
      // redirect to login page
      if (authRequired && !loggedIn) {
        return { name: 'Login' }
      }
    })
  }
}

export default RouterInterceptor
