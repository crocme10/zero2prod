import { defineStore } from 'pinia'
import LocalStorage from '../utils/LocalStorage'
import AuthService from '../api/AuthService'

export const useAuthStore = defineStore('auth', {
  state: () => ({}),
  actions: {
    async register(data: Map<string, any>) {
      try {
        console.log('AuthStore::register ' + data)
        // FIXME Note: I'm not storing anything!
        const res = await AuthService.register(data)
        return res
      } catch (error) {
        console.log(error)
        return error
      }
    },
    async login(data: Map<string, any>) {
      try {
        const res = await AuthService.login(data)
        LocalStorage.setToken(res.data?.access_token)
        return res
      } catch (error) {
        console.log(error)
      }
    },
    logout() {
      LocalStorage.removeToken()
    }
  }
})
