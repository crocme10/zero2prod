import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import About from '../views/About.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    //hidden: true,
    meta: {
      title: 'Home'
    }
  },
  {
    path: '/about',
    name: 'About',
    component: About,
    //hidden: true,
    meta: {
      title: 'About'
    }
  }
]
//   {
//     path: '/',
//     name: 'Home',
//     component: MainLayout,
//     redirect: { name: 'Dashboard' },
//     meta: {
//       requiresAuth: true,
//       title: 'Homepage'
//     },
//     children: [
//       {
//         path: '',
//         name: 'Dashboard',
//         component: DashboardPage,
//         meta: {
//           requiresAuth: true,
//           title: 'Dashboard'
//         }
//       },
//       {
//         path: 'project',
//         name: 'Project',
//         component: ProjectPage,
//         meta: {
//           requiresAuth: true,
//           title: 'Project'
//         }
//       },
//       {
//         path: 'language',
//         name: 'language',
//         component: LanguagePage,
//         meta: {
//           requiresAuth: true,
//           title: 'Language'
//         }
//       }
//     ]
//   },
// ]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes
})

export default router
