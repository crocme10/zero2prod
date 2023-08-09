import './style.css'
import './main.css'
import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import RouterInterceptor from './plugins/Router'

const app = createApp(App)
  .use(createPinia())
  .use(router)
  //.use(RouterInterceptor)

app.mount('#app')
