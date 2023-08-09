import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import About from '../views/About.vue'
import Manage from '../views/Manage.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    meta: {
      title: 'Home'
    }
  },
  {
    path: '/about',
    name: 'About',
    component: About,
    meta: {
      title: 'About'
    }
  },
  {
    path: '/manage',
    name: 'Manage',
    component: Manage,
    meta: {
      requiresAuth: true,
      title: 'About'
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

export default router
