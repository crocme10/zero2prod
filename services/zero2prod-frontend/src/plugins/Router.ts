import router from '../router'
import { useAuthStore } from '../stores/Auth'

const RouterInterceptor = {
  install() {
    router.beforeEach(async (to, from, next) => {
      console.log(from)
      console.log(to)
      next()
      // if (to.matched.some(record => record.meta.requiresAuth)) {
      //   console.log('requires auth')
      //   const authStore = useAuthStore()
      //   const loggedIn = authStore.isLoggedIn
      //   if (!loggedIn && to.name !== 'Login') {
      //     return { name: 'Login' }
      //   }
      // } else {
      //   console.log('does not requires auth')
      //   next()
      // }
    })
  }
}

export default RouterInterceptor
