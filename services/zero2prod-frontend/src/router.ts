import { createRouter, createWebHistory } from 'vue-router'
import Home from './views/Home.vue'
import About from './views/About.vue'
import Manage from './views/Manage.vue'
import { useAuthStore } from './stores/Auth'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    meta: {
      requiresAuth: false,
    }
  },
  {
    path: '/about',
    name: 'About',
    component: About,
    meta: {
      requiresAuth: false,
    }
  },
  {
    path: '/manage',
    name: 'Manage',
    component: Manage,
    meta: {
      requiresAuth: true,
    }
  },
  {
    // A catch all, could redirect to a 404 page
    path: '/:catchAll(.*)*',
    redirect: { name: 'Home' },
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  linkExactActiveClass: 'text-yellow-500'
})

router.beforeEach( to => {
  if (to.meta.requiresAuth) {
    console.log('requires auth')
    const authStore = useAuthStore()
    const loggedIn = authStore.isLoggedIn
    if (!loggedIn && to.name !== 'Login') {
      // FIXME Send to login instead of home
      return { name: 'Home' }
    }
  }
})

export default router
